//! Root comments share source positions with document markers and directives.
//! Plan their final positions separately from the root's syntax, so deleting a
//! comment does not delete the framing boundary that can host its replacement.

use super::{Comment, CommentPosition, EditableDocument, emit};
use crate::{Error, Span, comments::CommentPlacement};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub(super) struct RootComments {
    pub(super) rendered: Vec<Comment>,
    pub(super) patches: Vec<(Span, String)>,
}

/// A source boundary with explicit separation on either side of a comment.
struct Anchor {
    at: usize,
    before: String,
    after: String,
}

impl RootComments {
    pub(super) fn plan(
        original: &EditableDocument,
        candidate: &EditableDocument,
        region: Option<Span>,
    ) -> Result<Self, Error> {
        let comments = &candidate.nodes[candidate.root.0].comments;
        let Some(region) = region else {
            // No body rewrite means its existing layout remains authoritative.
            // Text-only framing edits must not relocate unchanged body comments.
            let mut plan = Self {
                rendered: comments.clone(),
                patches: Vec::new(),
            };
            plan.changed_comments(original, None);
            return Ok(plan);
        };
        let layout = RootLayout::new(original, candidate);
        let mut following = vec![None; comments.len()];
        let mut next = None;
        for (index, comment) in comments.iter().enumerate().rev() {
            following[index] = next;
            if layout.fixed(comment) {
                next = Some(index);
            }
        }

        let mut plan = Self::default();
        let mut previous = None;
        let mut bytes = 0usize;
        for (index, comment) in comments.iter().enumerate() {
            if layout.fixed(comment) {
                previous = Some(index);
                plan.rendered.push(comment.clone());
                continue;
            }
            let next = following[index].map(|index| &comments[index]);
            let previous_comment = previous.map(|index| &comments[index]);
            let anchor = next
                .and_then(|next| comment_anchor(original, region, comment, next, true))
                .or_else(|| {
                    previous_comment.and_then(|previous| {
                        comment_anchor(original, region, comment, previous, false)
                    })
                })
                .or_else(|| marker_anchor(original, &layout, comment, index, region, next))
                .filter(|anchor| {
                    previous_comment.is_none_or(|previous| previous.span.end() <= anchor.at)
                        && next.is_none_or(|next| anchor.at <= next.span.start())
                });
            let Some(anchor) = anchor else {
                plan.rendered.push(comment.clone());
                continue;
            };
            bytes = bytes
                .checked_add(anchor.before.len() + 1 + comment.text.len() + anchor.after.len())
                .filter(|&size| size <= original.options.max_input_bytes)
                .ok_or(Error {
                    span: region,
                    message: "emitted YAML exceeds input byte limit",
                })?;
            let text = format!("{}#{}{}", anchor.before, comment.text, anchor.after);
            plan.patches.push((
                Span {
                    start: anchor.at,
                    end: anchor.at,
                },
                text,
            ));
        }
        plan.changed_comments(original, Some(region));
        Ok(plan)
    }

    /// Compare final comment identities with the original source. Intermediate
    /// replacements and removals do not create independent, overlapping patches.
    fn changed_comments(&mut self, original: &EditableDocument, region: Option<Span>) {
        let retained: HashMap<_, _> = self
            .rendered
            .iter()
            .filter(|comment| !comment.span.is_empty())
            .map(|comment| (comment.span.start(), comment))
            .collect();
        for comment in &original.nodes[original.root.0].comments {
            if region.is_some_and(|region| contains(region, comment.span)) {
                continue;
            }
            match retained.get(&comment.span.start()) {
                Some(current) if current.text != comment.text => {
                    self.patches
                        .push((comment.span, format!("#{}", current.text)));
                }
                None => self
                    .patches
                    .push((removal_span(&original.source, comment.span), String::new())),
                _ => {}
            }
        }
    }
}

/// Root syntax determines which body comments need an explicit boundary.
#[derive(Clone, Copy)]
enum RootKind {
    Token,
    Omitted,
    BlockScalar,
    FlowCollection,
    BlockCollection { header: bool },
}

struct RootLayout {
    kind: RootKind,
    body: Span,
    root_start: usize,
    leading_count: usize,
    marker_header: Option<usize>,
}

impl RootLayout {
    fn new(original: &EditableDocument, candidate: &EditableDocument) -> Self {
        let root_start = original.nodes[original.root.0].span.start();
        let body = Span {
            start: original.start_marker.map_or(0, |marker| {
                line_end(&original.source, marker.end()).min(root_start)
            }),
            end: original
                .end_marker
                .map_or(original.source.len(), Span::start),
        };
        let root = &candidate.nodes[candidate.root.0];
        let kind = if emit::omitted(root) {
            RootKind::Omitted
        } else {
            match root.kind {
                super::NodeKind::Scalar {
                    style: crate::ScalarStyle::Literal | crate::ScalarStyle::Folded,
                    ..
                } => RootKind::BlockScalar,
                super::NodeKind::Sequence { .. } | super::NodeKind::Mapping { .. } => {
                    if emit::block_collection(&root.kind, false) {
                        RootKind::BlockCollection {
                            header: emit::has_inline_header(root, false),
                        }
                    } else {
                        RootKind::FlowCollection
                    }
                }
                _ => RootKind::Token,
            }
        };
        Self {
            kind,
            body,
            root_start,
            leading_count: root
                .comments
                .iter()
                .take_while(|comment| comment.placement == CommentPlacement::Leading)
                .count(),
            marker_header: root.comments.iter().rposition(|comment| {
                comment.placement == CommentPlacement::Inline
                    && (comment.span.is_empty() || contains(body, comment.span))
            }),
        }
    }

    /// Comments already in the framing retain their source anchors. Comments
    /// within the body may depend on syntax removed by the final candidate.
    fn fixed(&self, comment: &Comment) -> bool {
        if comment.span.is_empty() {
            return false;
        }
        if !contains(self.body, comment.span) {
            return true;
        }
        match self.kind {
            RootKind::Token => true,
            RootKind::Omitted => {
                comment.span.start() < self.root_start
                    && !(comment.placement == CommentPlacement::Leading
                        && self.marker_header.is_some())
            }
            // Indented comments can become block-scalar text; rendering them
            // together supplies the indentation boundary after the scalar.
            RootKind::BlockScalar => false,
            RootKind::FlowCollection => comment.placement != CommentPlacement::Leading,
            RootKind::BlockCollection { header } => match comment.placement {
                CommentPlacement::Leading | CommentPlacement::Trailing => false,
                CommentPlacement::Inline => header,
                CommentPlacement::Detached => true,
            },
        }
    }
}

pub(super) fn root_region(
    original: &EditableDocument,
    candidate: &EditableDocument,
    mut region: Span,
) -> Span {
    let layout = RootLayout::new(original, candidate);
    for comment in &original.nodes[original.root.0].comments {
        if !layout.fixed(comment) {
            region.start = region.start().min(comment.span.start());
            region.end = region.end().max(comment.span.end());
        }
    }
    region
}

fn contains(region: Span, span: Span) -> bool {
    span.start() >= region.start() && span.end() <= region.end()
}

fn comment_anchor(
    original: &EditableDocument,
    region: Span,
    comment: &Comment,
    anchor: &Comment,
    before: bool,
) -> Option<Anchor> {
    if contains(region, anchor.span) {
        return None;
    }
    let breaks = outer_comment_breaks(comment, anchor, before)?;
    let separator = format!(
        "{}{}",
        emit::newline(&original.source).repeat(breaks),
        " ".repeat(emit::indent(&original.source, anchor.span.start()))
    );
    Some(if before {
        Anchor {
            at: anchor.span.start(),
            before: String::new(),
            after: separator,
        }
    } else {
        Anchor {
            at: anchor.span.end(),
            before: separator,
            after: String::new(),
        }
    })
}

fn marker_anchor(
    original: &EditableDocument,
    layout: &RootLayout,
    comment: &Comment,
    index: usize,
    region: Span,
    next: Option<&Comment>,
) -> Option<Anchor> {
    let newline = emit::newline(&original.source);
    match comment.placement {
        CommentPlacement::Leading => {
            let marker = original.start_marker?;
            let collection = matches!(
                layout.kind,
                RootKind::FlowCollection | RootKind::BlockCollection { .. }
            );
            let precedes_header = next.is_some_and(|next| next.span.end() <= region.start())
                || (matches!(layout.kind, RootKind::Omitted)
                    && layout.marker_header.is_some_and(|header| index < header));
            if !collection
                && !precedes_header
                && line_end(&original.source, marker.end()) < region.start()
            {
                return None;
            }
            Some(Anchor {
                at: marker.start(),
                before: String::new(),
                after: newline.to_owned(),
            })
        }
        CommentPlacement::Trailing | CommentPlacement::Detached
            if comment.placement == CommentPlacement::Trailing
                || comment.position == CommentPosition::After =>
        {
            let marker = original.end_marker?;
            let at = line_end(&original.source, marker.end());
            Some(Anchor {
                at,
                before: newline.repeat(if comment.placement == CommentPlacement::Detached {
                    2
                } else {
                    1
                }),
                after: String::new(),
            })
        }
        CommentPlacement::Inline
            if matches!(
                layout.kind,
                RootKind::Omitted | RootKind::BlockCollection { header: false }
            ) =>
        {
            if index != layout.leading_count {
                let marker = original.end_marker?;
                return Some(Anchor {
                    at: line_end(&original.source, marker.end()),
                    before: " ".to_owned(),
                    after: String::new(),
                });
            }
            let marker = original.start_marker?;
            let end = line_end(&original.source, marker.end());
            let at = if end >= region.start() && region.start() > marker.end() {
                // The original root shares the marker line, or is omitted at
                // EOF. The root renderer supplies its own initial line break.
                region.start()
            } else {
                end
            };
            Some(Anchor {
                at,
                before: " ".to_owned(),
                after: String::new(),
            })
        }
        _ => None,
    }
}

fn line_end(source: &str, start: usize) -> usize {
    source[start..]
        .find(['\r', '\n'])
        .map_or(source.len(), |offset| start + offset)
}

fn removal_span(source: &str, span: Span) -> Span {
    // Removing a standalone comment includes its line break, so adjacent
    // leading/trailing comments do not become detached by an empty line.
    let start = source[..span.start()]
        .rfind(['\r', '\n'])
        .map_or(0, |index| index + 1);
    if source[start..span.start()]
        .bytes()
        .all(|byte| matches!(byte, b' ' | b'\t'))
    {
        let suffix = &source[span.end()..];
        let terminator = if suffix.starts_with("\r\n") {
            2
        } else {
            usize::from(suffix.starts_with(['\r', '\n']))
        };
        Span {
            start,
            end: span.end() + terminator,
        }
    } else {
        span
    }
}
/// Choose separation only when inserting beside a retained outer comment
/// preserves both attachments. Otherwise emission places the new comment next
/// to the rewritten content. Inline and leading/trailing comments must remain
/// adjacent to their syntax; detached comments require blank separation.
fn outer_comment_breaks(comment: &Comment, anchor: &Comment, before: bool) -> Option<usize> {
    if !matches!(
        comment.position,
        CommentPosition::Before | CommentPosition::After
    ) || anchor.position != comment.position
        || comment.placement == CommentPlacement::Inline
    {
        return None;
    }
    if anchor.placement == comment.placement {
        return Some(1);
    }
    match (
        comment.position,
        before,
        comment.placement,
        anchor.placement,
    ) {
        (CommentPosition::Before, true, CommentPlacement::Detached, CommentPlacement::Leading)
        | (
            CommentPosition::After,
            false,
            CommentPlacement::Detached,
            CommentPlacement::Inline | CommentPlacement::Trailing,
        ) => Some(2),
        (CommentPosition::After, false, CommentPlacement::Trailing, CommentPlacement::Inline) => {
            Some(1)
        }
        _ => None,
    }
}
