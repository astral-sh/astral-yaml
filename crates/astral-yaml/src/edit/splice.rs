//! Source boundaries are reserved before emission. Syntax output describes its
//! ending; it cannot choose which source bytes to replace. Patch construction
//! checks all ranges and the complete output size before allocating the result.

use super::{
    Comment,
    emit::{self, Context, Ending, Fragment},
    layout::Layout,
    view::{Emission, Syntax},
};
use crate::{Error, NodeId, Span, comments::CommentPlacement};
use std::borrow::Cow;

#[cfg(test)]
#[path = "splice_tests.rs"]
mod tests;

/// A rewritten region includes descendant comments, but leaves the root's outer
/// comments in their original source positions unless explicitly requested.
pub(super) fn region<'a>(document: impl Into<Syntax<'a, Span>>, id: NodeId, outer: bool) -> Span {
    let document = document.into();
    let root = &document.nodes[id.0];
    let mut start = root.span.start();
    let mut end = root.span.end();
    let mut pending = vec![id];
    while let Some(current) = pending.pop() {
        let node = &document.nodes[current.0];
        if current != id || outer {
            for comment in &node.comments {
                start = start.min(comment.span.start());
                end = end.max(comment.span.end());
            }
        }
        emit::children(&node.kind, &mut pending);
    }
    Span::new(start, end).expect("ordered source ranges")
}

pub(super) fn indent(source: &str, at: usize) -> usize {
    source[..at]
        .rsplit('\n')
        .next()
        .unwrap_or("")
        .bytes()
        .take_while(|&byte| byte == b' ')
        .count()
}

pub(super) enum Separator {
    None,
    Space,
    Line(usize),
}

/// Everything the emitter needs to join a node to its retained prefix. These
/// are formatting instructions, not permission to inspect or consume source.
pub(super) struct Placement<'a, Location> {
    pub(super) comments: &'a [Comment<Location>],
    pub(super) context: Context,
    pub(super) before: Separator,
    pub(super) at_column_zero: bool,
    pub(super) comment_follows: bool,
    pub(super) diagnostic_span: Span,
}

enum RootLine {
    Standalone { start: usize },
    Shared,
}

/// A subtree owns its body and may consume adjacent whitespace to terminate
/// its final scalar or comment. Reserve that entire interval before checking
/// overlap, even when the emitted syntax ultimately needs only its body.
pub(super) struct Boundary<'a> {
    source: &'a str,
    body: Span,
    reserved: Span,
    erased: Option<RootLine>,
    end_marker: Option<Span>,
}

impl<'a> Boundary<'a> {
    pub(super) fn new(
        source: &'a str,
        body: Span,
        erase_root: bool,
        end_marker: Option<Span>,
    ) -> Self {
        let erased = (erase_root && !body.is_empty()).then(|| {
            let prefix = source[..body.start()]
                .rsplit('\n')
                .next()
                .unwrap_or("")
                .trim_start_matches('\u{feff}');
            if prefix.bytes().all(|byte| matches!(byte, b' ' | b'\t')) {
                RootLine::Standalone {
                    start: body.start() - prefix.len(),
                }
            } else {
                RootLine::Shared
            }
        });
        let start = match erased {
            Some(RootLine::Standalone { start }) => start,
            _ => body.start(),
        };
        Self {
            source,
            body,
            reserved: Span {
                start,
                end: following_line_breaks(source, body.end()),
            },
            erased,
            end_marker,
        }
    }

    pub(super) fn body(&self) -> Span {
        self.body
    }

    pub(super) fn reserved(&self) -> Span {
        self.reserved
    }

    fn placement<'comments, Location: Copy + Into<Option<Span>>>(
        &self,
        context: Context,
        comments: &'comments [Comment<Location>],
        all_comments: &[Comment<Location>],
        own_line: bool,
    ) -> Placement<'comments, Location> {
        let prefix = self.source[..self.body.start()]
            .rsplit('\n')
            .next()
            .unwrap_or("");
        let line_start = self.body.start() - prefix.len();
        let terminated = self.body.is_empty()
            && all_comments.iter().any(|comment| {
                comment.span.into().is_some_and(|span| {
                    span.start() >= line_start && span.end() <= self.body.start()
                })
            });
        let before = if terminated || (own_line && !prefix.bytes().all(|byte| byte == b' ')) {
            Separator::Line(context.indent)
        } else if self.body.is_empty()
            && matches!(
                self.source.as_bytes()[..self.body.start()].last(),
                Some(b':' | b'-' | b'?')
            )
        {
            Separator::Space
        } else {
            Separator::None
        };
        Placement {
            comments,
            context,
            before,
            at_column_zero: prefix.trim_start_matches('\u{feff}').is_empty(),
            comment_follows: self.body.is_empty()
                && self.source.as_bytes().get(self.body.end()) == Some(&b'#'),
            diagnostic_span: self.body,
        }
    }

    /// Resolve the reserved whitespace using the actual lexical ending. No
    /// replacement can escape the interval used for overlap planning.
    fn complete(&self, mut fragment: Fragment, limit: usize) -> Result<Replacement, Error> {
        let mut span = self.body;
        match fragment.ending {
            Ending::BlockScalar => span.end = self.reserved.end(),
            _ if self.erased.is_some() => {
                span = self.reserved;
                if matches!(self.erased, Some(RootLine::Shared))
                    && self.source[self.body.start()..span.end()].contains('\n')
                {
                    // The root's token disappears, but a retained document
                    // marker still needs its line terminator.
                    let without_indent = fragment.text.trim_end_matches(' ');
                    if without_indent.ends_with('\n') {
                        fragment.text.truncate(without_indent.len());
                    } else {
                        fragment.text.push('\n');
                    }
                    fragment.ending = Ending::Token;
                }
            }
            _ => {}
        }
        let suffix = &self.source[span.end()..];
        let trimmed = suffix.trim_start_matches([' ', '\t']);
        let suffix_has_break = trimmed.starts_with('\n');
        if matches!(fragment.ending, Ending::Comment) && suffix_has_break {
            // Reuse the retained terminator, excluding spaces that would
            // otherwise become part of the emitted comment's text.
            span.end += suffix.len() - trimmed.len();
        }
        let needs_break = matches!(fragment.ending, Ending::Comment)
            || self
                .end_marker
                .is_some_and(|marker| marker.start() == span.end())
            || (!span.is_empty() && self.source[..span.end()].ends_with('\n'));
        if needs_break
            && !fragment.text.is_empty()
            && !fragment.text.ends_with('\n')
            && !suffix_has_break
        {
            fragment.text.push('\n');
        }
        if fragment.text.len() > limit {
            return Err(Error {
                span: self.body,
                message: "emitted YAML exceeds input byte limit",
            });
        }
        Ok(Replacement {
            span,
            text: fragment.text,
        })
    }
}

/// Complete replacement text and its source interval, ready for patch checking.
#[derive(Debug)]
pub(super) struct Replacement {
    pub(super) text: String,
    pub(super) span: Span,
}

pub(super) fn render<'a, Location: Copy + Into<Option<Span>> + 'a>(
    document: impl Into<Emission<'a, Location>>,
    id: NodeId,
    region: Span,
    indent: usize,
    flow: bool,
) -> Result<Replacement, Error> {
    let document = document.into();
    let boundary = Boundary::new(
        document.source,
        region,
        id == document.root && emit::omitted(&document.nodes[id.0]),
        document.end_marker,
    );
    render_with_comments(
        document,
        id,
        &boundary,
        Context::value(indent, flow, if id == document.root { 0 } else { indent }),
        &document.nodes[id.0].comments,
        &Layout::new(Syntax {
            nodes: document.nodes,
            root: document.root,
        }),
    )
}

pub(super) fn render_with_comments<'a, Location: Copy + Into<Option<Span>> + 'a>(
    document: impl Into<Emission<'a, Location>>,
    id: NodeId,
    boundary: &Boundary<'_>,
    context: Context,
    root_comments: &'a [Comment<Location>],
    layout: &Layout,
) -> Result<Replacement, Error> {
    let document = document.into();
    let region = boundary.body();
    let visible: Vec<_> = root_comments
        .iter()
        .filter(|comment| {
            comment
                .span
                .into()
                .is_none_or(|span| span.start() >= region.start() && span.end() <= region.end())
        })
        .cloned()
        .collect();
    let own_line = id == document.root
        && (emit::block_collection(&document.nodes[id.0].kind, context.in_flow())
            || visible
                .iter()
                .any(|comment| comment.placement == CommentPlacement::Leading));
    let placement = boundary.placement(context, &visible, root_comments, own_line);
    let fragment = emit::render(
        Syntax {
            nodes: document.nodes,
            root: document.root,
        },
        document.options,
        id,
        &placement,
        layout,
    )?;
    boundary.complete(fragment, document.options.max_input_bytes)
}

/// Consume complete whitespace-only lines, retaining the indentation of any
/// following syntax. Block scalars and erased roots can own these breaks.
fn following_line_breaks(source: &str, start: usize) -> usize {
    let suffix = &source[start..];
    match suffix.bytes().position(|byte| !byte.is_ascii_whitespace()) {
        Some(index) => suffix[..index]
            .rfind('\n')
            .map_or(start, |offset| start + offset + 1),
        None => source.len(),
    }
}

/// Validated borrowed/owned segments. Construction proves source ordering,
/// disjoint UTF-8 intervals, and the exact final size. Application cannot fail
/// or slice source at unchecked offsets.
pub(super) struct PatchSet<'a> {
    segments: Vec<Cow<'a, str>>,
    length: usize,
}

impl<'a> PatchSet<'a> {
    pub(super) fn new(
        source: &'a str,
        mut patches: Vec<(Span, String)>,
        limit: usize,
    ) -> Result<Self, &'static str> {
        // Stable order preserves multiple insertions at the same boundary.
        // Insertions precede nonempty replacements starting at that boundary.
        patches.sort_by_key(|(span, _)| (span.start(), span.end()));
        let mut segments = Vec::new();
        let mut position = 0;
        let mut length = 0usize;
        for (span, replacement) in patches {
            let prefix = source
                .get(position..span.start())
                .ok_or("overlapping or invalid source patches")?;
            source
                .get(span.start()..span.end())
                .ok_or("overlapping or invalid source patches")?;
            length = length
                .checked_add(prefix.len())
                .and_then(|size| size.checked_add(replacement.len()))
                .filter(|&size| size <= limit)
                .ok_or("emitted YAML exceeds input byte limit")?;
            segments.extend([Cow::Borrowed(prefix), Cow::Owned(replacement)]);
            position = span.end();
        }
        let suffix = &source[position..];
        length = length
            .checked_add(suffix.len())
            .filter(|&size| size <= limit)
            .ok_or("emitted YAML exceeds input byte limit")?;
        segments.push(Cow::Borrowed(suffix));
        Ok(Self { segments, length })
    }

    pub(super) fn apply(self) -> String {
        let mut output = String::with_capacity(self.length);
        for segment in self.segments {
            output.push_str(&segment);
        }
        output
    }
}
