//! Root comments share source positions with document markers and directives.
//! Plan their final positions separately from the root's syntax, so deleting a
//! comment does not delete the framing boundary that can host its replacement.

use super::{Comment, CommentPosition, EditableDocument, emit};
use crate::{Error, Span, comments::CommentPlacement};
use std::collections::{HashMap, HashSet};

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

/// Supply document boundaries required by the final root layout. The original
/// and candidate keep the same node identities; only source coordinates move.
/// Planning against this framed source lets retained and generated markers use
/// the same comment-placement rules.
pub(super) fn prepare(
    original: &EditableDocument,
    candidate: &mut EditableDocument,
) -> Option<EditableDocument> {
    let layout = RootLayout::new(original, candidate);
    let root = &candidate.nodes[candidate.root.0];
    let collection = matches!(
        layout.kind,
        RootKind::FlowCollection | RootKind::BlockCollection { .. }
    );
    let start = original.start_marker.is_none()
        && (matches!(layout.kind, RootKind::Omitted)
            || (collection
                && root
                    .comments
                    .iter()
                    .any(|comment| comment.placement == CommentPlacement::Leading))
            || layout.inline.start.is_some());
    let suffix = usize::from(
        original.end_marker.is_none()
            && matches!(layout.kind, RootKind::Omitted)
            && root.comments.iter().any(|comment| {
                comment.placement == CommentPlacement::Trailing
                    || (comment.placement == CommentPlacement::Detached
                        && comment.position == CommentPosition::After)
            }),
    );
    let ends = layout
        .inline
        .end
        .len()
        .saturating_sub(layout.inline.slots.len())
        .max(suffix);
    if !start && ends == 0 {
        return None;
    }

    let mut framed = original.clone();
    let newline = emit::newline(&original.source);
    if start {
        // Place the boundary immediately before the root's syntax line. Any
        // prefix comments and blank lines then keep their original relation to
        // that boundary, including a BOM at the beginning of the stream.
        let root_start = original.nodes[original.root.0].span.start();
        let at = original.source[..root_start]
            .rfind(['\r', '\n'])
            .map_or_else(
                || usize::from(original.source.starts_with('\u{feff}')) * '\u{feff}'.len_utf8(),
                |index| index + 1,
            );
        let marker = format!("---{newline}");
        framed.source.insert_str(at, &marker);
        shift_spans(&mut framed, at, marker.len());
        shift_spans(candidate, at, marker.len());
        framed.start_marker = Some(Span {
            start: at,
            end: at + 3,
        });
    }
    if ends > 0 {
        if framed.end_marker.is_none() && !framed.source.ends_with(['\r', '\n']) {
            framed.source.push_str(newline);
        }
        let at = framed.end_marker.map_or(framed.source.len(), Span::start);
        let text = format!("...{newline}").repeat(ends);
        framed.source.insert_str(at, &text);
        shift_spans(&mut framed, at, text.len());
        shift_spans(candidate, at, text.len());
        let mut markers: Vec<_> = (0..ends)
            .map(|index| {
                let start = at + index * (3 + newline.len());
                Span {
                    start,
                    end: start + 3,
                }
            })
            .collect();
        markers.extend(framed.end_marker);
        markers.append(&mut framed.extra_end_markers);
        framed.end_marker = Some(markers.remove(0));
        framed.extra_end_markers = markers;
    }
    candidate.source.clone_from(&framed.source);
    candidate.start_marker = framed.start_marker;
    candidate.end_marker = framed.end_marker;
    candidate
        .extra_end_markers
        .clone_from(&framed.extra_end_markers);
    Some(framed)
}

fn shift_spans(document: &mut EditableDocument, at: usize, offset: usize) {
    let shift = |span: &mut Span| {
        if span.end() > at {
            if span.start() >= at {
                span.start += offset;
            }
            span.end += offset;
        }
    };
    for node in &mut document.nodes {
        shift(&mut node.span);
        for comment in &mut node.comments {
            if !comment.span.is_empty() {
                shift(&mut comment.span);
            }
        }
    }
    for marker in document
        .start_marker
        .iter_mut()
        .chain(document.end_marker.iter_mut())
        .chain(document.extra_end_markers.iter_mut())
    {
        shift(marker);
    }
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
        if let Some(marker) = layout.suffix_boundary {
            plan.retain_suffix_separation(original, marker);
        }
        Ok(plan)
    }

    /// A rewritten body owns its separation from the end marker. If an outer
    /// detached comment relied on that separation, give it a blank line on the
    /// retained side of the marker instead.
    fn retain_suffix_separation(&mut self, original: &EditableDocument, marker: Span) {
        let Some(comment) = self.rendered.iter().find(|comment| {
            comment.placement == CommentPlacement::Detached && comment.span.start() >= marker.end()
        }) else {
            return;
        };
        if !blank_between(&original.source, marker.end(), comment.span.start()) {
            let start = original.source[..comment.span.start()]
                .rfind(['\r', '\n'])
                .map_or(0, |index| index + 1);
            self.patches.push((
                Span { start, end: start },
                emit::newline(&original.source).to_owned(),
            ));
        }
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

/// Inline comments need syntax on their line. Reserve the root's own headers
/// first, then assign overflow to document markers in owner-local order.
struct InlineLayout {
    start: Option<usize>,
    end: Vec<usize>,
    slots: Vec<Span>,
    moved: HashSet<usize>,
}

impl InlineLayout {
    fn new(
        original: &EditableDocument,
        root: &super::Node,
        body: Span,
        kind: RootKind,
        leading: usize,
    ) -> Self {
        let headers = usize::from(root.anchor.is_some())
            + usize::from(root.tag.is_some())
            + match kind {
                RootKind::Token | RootKind::BlockScalar => 1,
                RootKind::FlowCollection => 2,
                RootKind::Omitted | RootKind::BlockCollection { .. } => 0,
            };
        let mut body_comments = root
            .comments
            .iter()
            .enumerate()
            .filter_map(|(index, comment)| {
                (comment.placement == CommentPlacement::Inline
                    && (comment.span.is_empty() || contains(body, comment.span)))
                .then_some(index)
            })
            .collect::<Vec<_>>();
        let occupied = occupied_markers(original, root, body);
        let start = if body_comments.len() > headers
            && body_comments.first() == Some(&leading)
            && original
                .start_marker
                .is_none_or(|marker| !occupied.contains(&marker.start()))
        {
            Some(body_comments.remove(0))
        } else {
            None
        };
        let end = body_comments.into_iter().skip(headers).collect::<Vec<_>>();
        let mut moved: HashSet<_> = start
            .iter()
            .chain(&end)
            .filter_map(|&index| {
                let span = root.comments[index].span;
                (!span.is_empty()).then_some(span.start())
            })
            .collect();
        if let Some(&first) = end.first() {
            // Comments following a relocated suffix header must follow it to
            // the same boundary, rather than remain earlier in the old body.
            moved.extend(root.comments[first..].iter().filter_map(|comment| {
                (!comment.span.is_empty() && contains(body, comment.span))
                    .then_some(comment.span.start())
            }));
        }
        let slots = original
            .end_marker
            .iter()
            .chain(&original.extra_end_markers)
            .copied()
            .filter(|&marker| !occupied.contains(&marker.start()))
            .collect();
        Self {
            start,
            end,
            slots,
            moved,
        }
    }
}

/// Match fixed inline comments to their preceding marker once. Searching the
/// ordered marker list avoids scanning every comment for every document suffix.
fn occupied_markers(original: &EditableDocument, root: &super::Node, body: Span) -> HashSet<usize> {
    let markers: Vec<_> = original
        .start_marker
        .iter()
        .chain(original.end_marker.iter())
        .chain(&original.extra_end_markers)
        .copied()
        .collect();
    root.comments
        .iter()
        .filter_map(|comment| {
            if comment.placement != CommentPlacement::Inline
                || comment.span.is_empty()
                || contains(body, comment.span)
            {
                return None;
            }
            let index = markers.partition_point(|marker| marker.end() <= comment.span.start());
            let marker = markers.get(index.checked_sub(1)?)?;
            (!original.source[marker.end()..comment.span.start()].contains(['\r', '\n']))
                .then_some(marker.start())
        })
        .collect()
}

struct RootLayout {
    kind: RootKind,
    body: Span,
    root_start: usize,
    suffix_boundary: Option<Span>,
    inline: InlineLayout,
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
        let leading_count = root
            .comments
            .iter()
            .take_while(|comment| comment.placement == CommentPlacement::Leading)
            .count();
        let inline = InlineLayout::new(original, root, body, kind, leading_count);
        Self {
            kind,
            body,
            root_start,
            suffix_boundary: original.end_marker.filter(|_| {
                matches!(kind, RootKind::Omitted | RootKind::BlockScalar)
                    || root
                        .comments
                        .iter()
                        .any(|comment| comment.placement == CommentPlacement::Trailing)
            }),
            inline,
        }
    }

    /// Comments already in the framing retain their source anchors. Comments
    /// within the body may depend on syntax removed by the final candidate.
    fn fixed(&self, comment: &Comment) -> bool {
        if comment.span.is_empty() || self.inline.moved.contains(&comment.span.start()) {
            return false;
        }
        if !contains(self.body, comment.span) {
            return true;
        }
        if comment.placement == CommentPlacement::Leading && self.inline.start.is_some() {
            return false;
        }
        match self.kind {
            RootKind::Token => true,
            RootKind::Omitted => {
                comment.span.start() < self.root_start
                    && (comment.placement != CommentPlacement::Leading
                        || self.inline.end.is_empty())
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
    if let Some(marker) = layout.suffix_boundary {
        // Trailing comments require adjacency to the root even when the end
        // marker hosts them. Omitted roots and block scalars also own these
        // separators. Render the complete body to establish that boundary.
        region.end = region.end().max(marker.start());
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
                || layout.inline.start.is_some_and(|header| index < header);
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
            let marker = original
                .extra_end_markers
                .last()
                .copied()
                .or(original.end_marker)?;
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
        CommentPlacement::Inline => {
            if let Some(slot) = layout
                .inline
                .end
                .iter()
                .position(|&comment| comment == index)
            {
                let marker = *layout.inline.slots.get(slot)?;
                return Some(Anchor {
                    at: line_end(&original.source, marker.end()),
                    before: " ".to_owned(),
                    after: String::new(),
                });
            }
            if layout.inline.start != Some(index) {
                return None;
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

fn blank_between(source: &str, before: usize, after: usize) -> bool {
    let mut end = line_end(source, before);
    while end < after {
        let start = end
            + if source[end..].starts_with("\r\n") {
                2
            } else {
                1
            };
        end = line_end(source, start);
        if end < after
            && source[start..end]
                .bytes()
                .all(|byte| matches!(byte, b' ' | b'\t'))
        {
            return true;
        }
    }
    false
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
