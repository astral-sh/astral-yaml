//! Replace source regions with rendered YAML.
//!
//! Reserve each region before rendering so its whitespace cannot overlap another
//! replacement. [`PatchSet`] checks ranges and output size before assembling the result.

use super::{
    Comment,
    emit::{self, Context, Ending, Fragment},
    layout::Layout,
    lines::Line,
    source::Source,
    view::{Emission, Syntax},
};
use astral_yaml::{Error, NodeId, Span, comments::CommentPlacement};
use std::borrow::Cow;

#[cfg(test)]
mod tests;

/// Return the source range covering `id` and its descendants' comments.
/// Set `outer` to include comments outside `id`'s own span.
pub(super) fn region<'a>(document: impl Into<Syntax<'a, Span>>, id: NodeId, outer: bool) -> Span {
    let document = document.into();
    let root = &document.nodes[id.index()];
    let mut start = root.span.start();
    let mut end = root.span.end();
    let mut pending = vec![id];
    while let Some(current) = pending.pop() {
        let node = &document.nodes[current.index()];
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
    Line::at(source, at).indent(at)
}

pub(super) enum Separator {
    None,
    Space,
    Line(usize),
}

/// Formatting needed to join a rendered node to the surrounding source.
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

/// The source range a replacement is allowed to consume.
///
/// This includes adjacent whitespace needed to end a scalar or comment. Reserve
/// it before checking overlap, even if rendering later leaves some of it unused.
pub(super) struct Boundary<'a> {
    source: &'a str,
    body: Span,
    reserved: Span,
    line: Line,
    erased: Option<RootLine>,
    end_marker: Option<Span>,
}

impl<'a> Boundary<'a> {
    pub(super) fn new(
        source: &'a str,
        body: Span,
        line: Line,
        erase_root: bool,
        end_marker: Option<Span>,
    ) -> Self {
        let erased = (erase_root && !body.is_empty()).then(|| {
            let prefix = &source[body.start() - line.column(body.start())..body.start()];
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
            reserved: super::span(start, following_line_breaks(source, body.end())),
            erased,
            end_marker,
            line,
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
        let at = self.body.start();
        let line_start = at - self.line.column(at);
        let terminated = self.body.is_empty()
            && all_comments.iter().any(|comment| {
                comment.span.into().is_some_and(|span| {
                    span.start() >= line_start && span.end() <= self.body.start()
                })
            });
        let before = if terminated || (own_line && !self.line.own_line(at)) {
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
            at_column_zero: self.line.column(at) == 0,
            comment_follows: self.body.is_empty()
                && self.source.as_bytes().get(self.body.end()) == Some(&b'#'),
            diagnostic_span: self.body,
        }
    }

    /// Finish the fragment's line break and choose its replacement range.
    /// The range must stay within the whitespace reserved before rendering.
    fn complete(&self, mut fragment: Fragment, limit: usize) -> Result<Replacement, Error> {
        let mut span = self.body;
        match fragment.ending {
            Ending::BlockScalar => span = super::span(span.start(), self.reserved.end()),
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
            span = super::span(span.start(), span.end() + (suffix.len() - trimmed.len()));
        }
        let needs_break = matches!(fragment.ending, Ending::Comment)
            || suffix.starts_with('\u{feff}')
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

/// Rendered text and the source range it replaces.
#[derive(Debug)]
pub(super) struct Replacement {
    pub(super) text: String,
    pub(super) span: Span,
}

pub(super) fn render<'a, Location: Copy + Into<Option<Span>> + 'a>(
    document: impl Into<Emission<'a, Location>>,
    id: NodeId,
    region: Span,
    flow: bool,
) -> Result<Replacement, Error> {
    let document = document.into();
    let Emission { syntax, source } = document;
    let root = id == syntax.root;
    let node = &syntax.nodes[id.index()];
    let line = Line::at(&source.text, region.start());
    let indent = line.indent(region.start());
    let boundary = Boundary::new(
        &source.text,
        region,
        line,
        root && emit::omitted(node),
        source.end_marker,
    );
    render_with_comments(
        document,
        id,
        &boundary,
        Context::value(indent, flow, if root { 0 } else { indent }),
        &node.comments,
        &Layout::new(syntax),
    )
}

pub(super) fn render_with_comments<'a, Location: Copy + Into<Option<Span>> + 'a>(
    Emission { syntax, source }: Emission<'a, Location>,
    id: NodeId,
    boundary: &Boundary<'_>,
    context: Context,
    root_comments: &'a [Comment<Location>],
    layout: &Layout,
) -> Result<Replacement, Error> {
    let region = boundary.body();
    let visible: Vec<_> = root_comments
        .iter()
        .filter(|comment| {
            comment
                .span
                .into()
                .is_none_or(|span| super::contains(region, span))
        })
        .cloned()
        .collect();
    let own_line = id == syntax.root
        && (emit::block_collection(&syntax.nodes[id.index()].kind, context.in_flow())
            || visible
                .iter()
                .any(|comment| comment.placement == CommentPlacement::Leading));
    let placement = boundary.placement(context, &visible, root_comments, own_line);
    let fragment = emit::render(syntax, source.options, id, &placement, layout)?;
    boundary.complete(fragment, source.options.max_input_bytes)
}

/// Skip whitespace through the last line break before the next token.
/// Leave that token's indentation untouched. At EOF, consume all remaining whitespace.
fn following_line_breaks(source: &str, start: usize) -> usize {
    let suffix = &source[start..];
    match suffix.bytes().position(|byte| !byte.is_ascii_whitespace()) {
        Some(index) => suffix[..index]
            .rfind('\n')
            .map_or(start, |offset| start + offset + 1),
        None => source.len(),
    }
}

/// Preserve stream marks crossed by a rewrite, including those between its comments.
/// Prefix lines are counted from the start of the stream and suffix lines from the end.
/// When an edit removes those lines, the marks stay outside the document content.
pub(super) fn apply(
    source: &Source,
    patches: Vec<(Span, String)>,
    span: Span,
) -> Result<String, Error> {
    let error = |message| Error { span, message };
    let marks = &source.byte_order_marks;
    let mut regions = Vec::new();
    if !marks.is_empty() {
        regions.extend(patches.iter().map(|(span, _)| *span));
    }
    let limit = source.options.max_input_bytes;
    let set = PatchSet::new(&source.text, patches, limit).map_err(error)?;
    let mut covered = Vec::new();
    for region in regions {
        let first = marks.partition_point(|mark| mark.start() < region.start());
        let last = marks.partition_point(|mark| mark.start() < region.end());
        covered.extend_from_slice(&marks[first..last]);
    }
    if covered.is_empty() {
        return Ok(set.apply());
    }
    if covered.len() > (limit - set.length) / '\u{feff}'.len_utf8() {
        return Err(error("emitted YAML exceeds input byte limit"));
    }

    let output = set.apply();
    let stream = astral_yaml::parse_with_options(&output, source.options)?;
    let [document] = stream.documents() else {
        return Err(error("expected exactly one document"));
    };
    let root = &stream.nodes()[document.root.index()];
    let mut insertions = std::collections::BTreeMap::<usize, usize>::new();
    covered.sort_unstable_by_key(|mark| mark.start());
    let split = covered
        .partition_point(|mark| mark.start() < source.nodes[source.root.index()].span.start());
    let (prefix, suffix) = covered.split_at(split);

    let mut lines = memchr::memchr_iter(b'\n', &output.as_bytes()[..document.span.start()]);
    let mut at = 0;
    let mut previous = 0;
    for mark in prefix {
        for _ in memchr::memchr_iter(b'\n', &source.text.as_bytes()[previous..mark.start()]) {
            if let Some(newline) = lines.next() {
                at = newline + 1;
            }
        }
        *insertions.entry(at).or_default() += 1;
        previous = mark.start();
    }

    let end = root.span.end();
    let boundary = if output[..end].ends_with('\n') {
        end
    } else {
        output[end..]
            .find('\n')
            .map_or(output.len(), |newline| end + newline + 1)
    };
    let mut lines = memchr::memrchr_iter(b'\n', &output.as_bytes()[boundary..])
        .map(|newline| boundary + newline + 1);
    let mut at = lines.next().unwrap_or(boundary);
    let mut previous = source.text.len();
    for mark in suffix.iter().rev() {
        for _ in memchr::memchr_iter(b'\n', &source.text.as_bytes()[mark.start()..previous]) {
            at = lines.next().unwrap_or(boundary);
        }
        *insertions.entry(at).or_default() += 1;
        previous = mark.start();
    }

    let patches = insertions
        .into_iter()
        .map(|(at, count)| {
            let mut text = String::new();
            if at == output.len() && !output.ends_with('\n') {
                text.push('\n');
            }
            text.extend(std::iter::repeat_n('\u{feff}', count));
            (super::span(at, at), text)
        })
        .collect();
    PatchSet::new(&output, patches, limit)
        .map(PatchSet::apply)
        .map_err(error)
}

/// Source replacements ready to apply.
///
/// Construction checks for overlapping ranges, invalid UTF-8 boundaries, and
/// excessive output size. Applying the patches only concatenates the checked segments.
struct PatchSet<'a> {
    segments: Vec<Cow<'a, str>>,
    length: usize,
}

impl<'a> PatchSet<'a> {
    fn new(
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

    fn apply(self) -> String {
        let mut output = String::with_capacity(self.length);
        for segment in self.segments {
            output.push_str(&segment);
        }
        output
    }
}
