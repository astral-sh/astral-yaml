//! Root comments share source positions with document markers and directives.
//! Plan their final positions separately from the root's syntax, so deleting a
//! comment does not delete the framing boundary that can host its replacement.

use super::{Comment, CommentPosition, EditableDocument, emit, layout::Layout};
use crate::{Error, Span, comments::CommentPlacement};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub(super) struct RootComments {
    pub(super) rendered: Vec<Comment>,
    pub(super) patches: Vec<(Span, String)>,
    /// At an empty root's insertion point, suffix comments follow its syntax.
    pub(super) after_body: Vec<(Span, String)>,
    /// Trailing attachments require adjacency through document suffix markers.
    trailing_end: Option<usize>,
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
    syntax: &Layout,
) -> Option<EditableDocument> {
    let layout = RootLayout::new(original, candidate, syntax);
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
    let removed_boundary = layout.removed_scalar_boundary(original, root);
    let suffix = if matches!(layout.kind, RootKind::Omitted) || layout.kind.ends_with_block_scalar()
    {
        root.comments
            .iter()
            .position(|comment| layout.kind.needs_suffix_boundary(comment))
            .and_then(|index| match original.end_marker {
                None => Some(removed_boundary.unwrap_or(original.source.len())),
                Some(marker) => root
                    .comments
                    .iter()
                    .position(|comment| {
                        comment.placement == CommentPlacement::Inline
                            && !comment.span.is_empty()
                            && comment.span.start() >= marker.end()
                            && comment.span.end() <= line_end(&original.source, marker.end())
                    })
                    .filter(|&inline| index < inline)
                    .map(|_| marker.start()),
            })
    } else {
        None
    };
    let mut insertions: Vec<_> = layout
        .inline
        .slots
        .iter()
        .filter_map(|slot| match slot {
            EndSlot::Retained(_) => None,
            EndSlot::Missing(at) => Some(*at),
        })
        .collect();
    insertions.extend(removed_boundary.or(suffix));
    // A descendant's prefix may begin before the root token (for example a
    // leading comment on the first item of a flow collection). Keep generated
    // framing outside the complete subtree rewrite, not just the root token.
    let root_start = emit::region(original, original.root, false).start();
    let root_line = (layout.inline.start.is_some()
        || (original.nodes[original.root.0].span.is_empty() && !insertions.is_empty()))
        && original
            .start_marker
            .is_some_and(|marker| line_end(&original.source, marker.end()) >= root_start);
    if !start && !root_line && insertions.is_empty() {
        return None;
    }

    let mut framed = original.clone();
    let newline = emit::newline(&original.source);
    insert_end_markers(&mut framed, candidate, insertions, newline);
    if root_line {
        framed.source.insert_str(root_start, newline);
        shift_spans(&mut framed, root_start, newline.len());
        shift_spans(candidate, root_start, newline.len());
        // Empty roots have no token to move automatically with their suffix.
        for document in [&mut framed, &mut *candidate] {
            let span = &mut document.nodes[document.root.0].span;
            if span.is_empty() && span.start() == root_start {
                span.start += newline.len();
                span.end += newline.len();
            }
        }
    }
    if start {
        // Prefix comments and blank lines keep their relation to the boundary.
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
    candidate.source.clone_from(&framed.source);
    candidate.start_marker = framed.start_marker;
    candidate.end_marker = framed.end_marker;
    candidate
        .extra_end_markers
        .clone_from(&framed.extra_end_markers);
    Some(framed)
}

fn insert_end_markers(
    framed: &mut EditableDocument,
    candidate: &mut EditableDocument,
    mut insertions: Vec<usize>,
    newline: &str,
) {
    // Work backwards so every requested location still uses original source
    // coordinates. Existing markers and comments move with each insertion.
    insertions.sort_unstable();
    while let Some(at) = insertions.pop() {
        let mut count = 1;
        while insertions.last() == Some(&at) {
            insertions.pop();
            count += 1;
        }
        let separator = if at > 0 && !framed.source[..at].ends_with(['\r', '\n']) {
            newline
        } else {
            ""
        };
        let text = format!("{separator}{}", format!("...{newline}").repeat(count));
        framed.source.insert_str(at, &text);
        shift_spans(framed, at, text.len());
        shift_spans(candidate, at, text.len());
        let added = (0..count).map(|index| {
            let start = at + separator.len() + index * (3 + newline.len());
            Span {
                start,
                end: start + 3,
            }
        });
        let mut markers: Vec<_> = framed
            .end_marker
            .iter()
            .chain(&framed.extra_end_markers)
            .copied()
            .collect();
        markers.extend(added);
        markers.sort_unstable_by_key(|marker| marker.start());
        framed.end_marker = Some(markers.remove(0));
        framed.extra_end_markers = markers;
    }
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
        layout: &RootLayout,
    ) -> Result<Self, Error> {
        let comments = &candidate.nodes[candidate.root.0].comments;
        let Some(region) = region else {
            // No body rewrite means its existing layout remains authoritative.
            // Text-only framing edits must not relocate unchanged body comments.
            let mut plan = Self {
                rendered: comments.clone(),
                ..Self::default()
            };
            plan.changed_comments(original, None);
            return Ok(plan);
        };
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
        let mut previous_position = 0;
        let mut bytes = 0usize;
        for (index, comment) in comments.iter().enumerate() {
            if layout.fixed(comment) {
                if comment.placement == CommentPlacement::Trailing {
                    plan.trailing_end = Some(comment.span.start());
                }
                previous = Some(index);
                previous_position = comment.span.end();
                plan.rendered.push(comment.clone());
                continue;
            }
            let next = following[index].map(|index| &comments[index]);
            let previous_comment = previous.map(|index| &comments[index]);
            let ordered = |anchor: &Anchor| {
                previous_position <= anchor.at
                    && next.is_none_or(|next| anchor.at <= next.span.start())
            };
            let anchor = next
                .and_then(|next| comment_anchor(original, region, comment, next, true))
                .filter(ordered)
                .or_else(|| {
                    previous_comment
                        .and_then(|previous| {
                            comment_anchor(original, region, comment, previous, false)
                        })
                        .filter(ordered)
                })
                .or_else(|| {
                    marker_anchor(original, layout, comment, index, region, next).filter(ordered)
                });
            let Some(mut anchor) = anchor else {
                plan.rendered.push(comment.clone());
                continue;
            };
            // A neighboring detached comment can supply separation only after
            // the final trailing attachment. Earlier blank lines are reclaimed
            // by finish_suffix when headers move past them.
            if comment.placement == CommentPlacement::Detached
                && comment.position == CommentPosition::After
                && anchor.before.is_empty()
                && plan
                    .trailing_end
                    .is_some_and(|end| !blank_between(&original.source, end, anchor.at))
            {
                anchor.before.push_str(emit::newline(&original.source));
            }
            bytes = bytes
                .checked_add(anchor.before.len() + 1 + comment.text.len() + anchor.after.len())
                .filter(|&size| size <= original.options.max_input_bytes)
                .ok_or(Error {
                    span: region,
                    message: "emitted YAML exceeds input byte limit",
                })?;
            previous_position = anchor.at;
            if comment.placement == CommentPlacement::Trailing {
                plan.trailing_end = Some(anchor.at);
            }
            let text = format!("{}#{}{}", anchor.before, comment.text, anchor.after);
            let patches = if region.is_empty()
                && anchor.at == region.end()
                && (comment.placement == CommentPlacement::Trailing
                    || (comment.placement == CommentPlacement::Detached
                        && comment.position == CommentPosition::After))
            {
                &mut plan.after_body
            } else {
                &mut plan.patches
            };
            patches.push((
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

    /// Whitespace between the body and suffix markers still determines comment
    /// attachment. Discard orphan blank lines before a trailing comment, while
    /// leaving bytes already owned by a rendered or removed region to its patch.
    pub(super) fn finish_suffix(
        &self,
        original: &EditableDocument,
        patches: &mut Vec<(Span, String)>,
    ) {
        let Some(end) = self.trailing_end else {
            return;
        };
        let source = &original.source;
        let mut at = original.nodes[original.root.0].span.end();
        let mut covered: Vec<_> = patches.iter().map(|(span, _)| *span).collect();
        covered.sort_by_key(|span| (span.start(), span.end()));
        while at < end {
            let line_end = line_end(source, at);
            let next = line_end
                + if source[line_end..].starts_with("\r\n") {
                    2
                } else {
                    usize::from(line_end < source.len())
                };
            let whole_line = at == 0 || source[..at].ends_with(['\r', '\n']);
            let index = covered.partition_point(|span| span.end() <= at);
            let owned = covered.get(index).is_some_and(|span| span.start() < next);
            if whole_line
                && next <= end
                && !owned
                && source[at..line_end]
                    .bytes()
                    .all(|byte| matches!(byte, b' ' | b'\t'))
            {
                patches.push((
                    Span {
                        start: at,
                        end: next,
                    },
                    String::new(),
                ));
            }
            at = next;
        }
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
        let after = self.trailing_end.unwrap_or(0).max(marker.end());
        if !blank_between(&original.source, after, comment.span.start()) {
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
    BlockCollection { header: bool, scalar_tail: bool },
}

impl RootKind {
    fn ends_with_block_scalar(self) -> bool {
        matches!(
            self,
            Self::BlockScalar
                | Self::BlockCollection {
                    scalar_tail: true,
                    ..
                }
        )
    }

    fn needs_suffix_boundary(self, comment: &Comment) -> bool {
        match self {
            Self::Omitted => {
                comment.placement == CommentPlacement::Trailing
                    || (comment.placement == CommentPlacement::Detached
                        && comment.position == CommentPosition::After)
            }
            kind if kind.ends_with_block_scalar() => {
                comment.placement == CommentPlacement::Detached
                    && comment.position == CommentPosition::After
            }
            _ => false,
        }
    }
}

/// Inline comments need syntax on their line. Reserve the root's own headers
/// first, then assign overflow to document markers in owner-local order.
struct InlineLayout {
    start: Option<usize>,
    end: Vec<usize>,
    slots: Vec<EndSlot>,
    moved: HashSet<usize>,
}

#[derive(Clone, Copy)]
enum EndSlot {
    Retained(Span),
    Missing(usize),
}

impl EndSlot {
    fn marker(self) -> Option<Span> {
        match self {
            Self::Retained(marker) => Some(marker),
            Self::Missing(_) => None,
        }
    }
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
        let shared_headers = body_comments.len() > 1;
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
        let suffix_start = root
            .comments
            .iter()
            .position(|comment| {
                (!comment.span.is_empty() && comment.span.start() >= body.end())
                    || comment.placement == CommentPlacement::Trailing
                    || (comment.placement == CommentPlacement::Detached
                        && comment.position == CommentPosition::After)
            })
            .unwrap_or(root.comments.len());
        // An existing opening-delimiter comment remains at that delimiter;
        // properties before it therefore cannot host later inline comments.
        let mut available = if matches!(kind, RootKind::FlowCollection)
            && body_comments
                .first()
                .is_some_and(|&index| root.comments[index].position == CommentPosition::Opening)
        {
            2
        } else {
            headers
        };
        let end = body_comments
            .into_iter()
            .filter(|&index| {
                if index < suffix_start && available > 0 {
                    available -= 1;
                    false
                } else {
                    true
                }
            })
            .collect::<Vec<_>>();
        let mut moved: HashSet<_> = start
            .iter()
            .chain(&end)
            .filter_map(|&index| {
                let span = root.comments[index].span;
                (!span.is_empty()).then_some(span.start())
            })
            .collect();
        if shared_headers {
            // Reassign body headers together: a retained closing comment must
            // not remain outside the rewrite when it becomes an opening one.
            moved.extend(root.comments.iter().filter_map(|comment| {
                (comment.placement == CommentPlacement::Inline
                    && !comment.span.is_empty()
                    && contains(body, comment.span))
                .then_some(comment.span.start())
            }));
        }
        if let Some(&first) = end.first() {
            // Comments following a relocated suffix header must follow it to
            // the same boundary, rather than remain earlier in the old body.
            moved.extend(root.comments[first..].iter().filter_map(|comment| {
                (!comment.span.is_empty() && contains(body, comment.span))
                    .then_some(comment.span.start())
            }));
        }
        let reserve_boundary = root
            .comments
            .iter()
            .any(|comment| kind.needs_suffix_boundary(comment))
            && end.first().is_some_and(|&index| suffix_start < index);
        let slots = end_slots(original, root, body, &end, &occupied, reserve_boundary);
        Self {
            start,
            end,
            slots,
            moved,
        }
    }
}

/// Allocate suffix headers between fixed neighbors, preserving candidate order.
/// Missing headers are inserted before the next fixed line, or at EOF.
fn end_slots(
    original: &EditableDocument,
    root: &super::Node,
    body: Span,
    end: &[usize],
    occupied: &HashSet<usize>,
    reserve_boundary: bool,
) -> Vec<EndSlot> {
    let mut free = original
        .end_marker
        .iter()
        .chain(&original.extra_end_markers)
        .copied()
        .filter(|marker| {
            !(occupied.contains(&marker.start())
                || reserve_boundary && Some(*marker) == original.end_marker)
        })
        .peekable();
    let mut following = vec![None; root.comments.len()];
    let mut next = None;
    for (index, comment) in root.comments.iter().enumerate().rev() {
        following[index] = next;
        if !comment.span.is_empty() && !contains(body, comment.span) {
            next = Some(comment.span);
        }
    }
    let mut previous = 0;
    let mut suffix = end.iter().copied().peekable();
    let mut slots = Vec::new();
    for (index, comment) in root.comments.iter().enumerate() {
        if suffix.peek() == Some(&index) {
            suffix.next();
            while free.peek().is_some_and(|marker| marker.start() < previous) {
                free.next();
            }
            let next = following[index].map_or(original.source.len(), Span::start);
            if free.peek().is_some_and(|marker| marker.end() <= next) {
                slots.push(EndSlot::Retained(free.next().expect("available marker")));
            } else {
                let at = if following[index].is_some() {
                    original.source[..next]
                        .rfind(['\r', '\n'])
                        .map_or(0, |index| index + 1)
                } else {
                    original.source.len()
                };
                slots.push(EndSlot::Missing(at));
            }
        }
        if !comment.span.is_empty() && !contains(body, comment.span) {
            previous = comment.span.end();
        }
    }
    slots
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

pub(super) struct RootLayout {
    kind: RootKind,
    body: Span,
    root_start: usize,
    suffix_boundary: Option<Span>,
    inline: InlineLayout,
    indented_suffixes: HashSet<usize>,
}

impl RootLayout {
    pub(super) fn new(
        original: &EditableDocument,
        candidate: &EditableDocument,
        syntax: &Layout,
    ) -> Self {
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
                            scalar_tail: syntax.tail(candidate.root).is_some(),
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
        // Flow delimiters can keep an indented suffix attached to the root.
        // Block descendants introduced by an edit can claim the same comment.
        // Render these suffixes at the root's indentation with its final body.
        let indent = emit::indent(&original.source, root_start);
        let indented_suffixes = root
            .comments
            .iter()
            .filter(|comment| {
                matches!(kind, RootKind::BlockCollection { .. })
                    && comment.placement == CommentPlacement::Detached
                    && comment.position == CommentPosition::After
                    && !comment.span.is_empty()
                    && contains(body, comment.span)
                    && emit::indent(&original.source, comment.span.start()) > indent
            })
            .map(|comment| comment.span.start())
            .collect();
        Self {
            kind,
            body,
            root_start,
            suffix_boundary: original.end_marker.filter(|_| {
                matches!(kind, RootKind::Omitted)
                    || kind.ends_with_block_scalar()
                    || root
                        .comments
                        .iter()
                        .any(|comment| comment.placement == CommentPlacement::Trailing)
            }),
            inline,
            indented_suffixes,
        }
    }

    pub(super) fn needs_scalar_boundary(&self, candidate: &EditableDocument) -> bool {
        self.kind.ends_with_block_scalar()
            && candidate.nodes[candidate.root.0]
                .comments
                .iter()
                .any(|comment| self.kind.needs_suffix_boundary(comment))
    }

    pub(super) fn needs_suffix_context(&self) -> bool {
        !self.indented_suffixes.is_empty()
    }

    /// A removed suffix comment can expose retained whitespace to the final
    /// scalar, even when the same transaction introduces that scalar. Preserve
    /// the boundary using the candidate's layout, not the original scalar style.
    /// Comments past an existing end marker cannot expose scalar content.
    fn removed_scalar_boundary(
        &self,
        original: &EditableDocument,
        root: &super::Node,
    ) -> Option<usize> {
        if !self.kind.ends_with_block_scalar() {
            return None;
        }
        let end = original.nodes[original.root.0].span.end();
        let boundary = original
            .end_marker
            .map_or(original.source.len(), Span::start);
        let retained: HashSet<_> = root
            .comments
            .iter()
            .filter(|comment| !comment.span.is_empty())
            .map(|comment| comment.span.start())
            .collect();
        original.nodes[original.root.0]
            .comments
            .iter()
            .any(|comment| {
                (end..boundary).contains(&comment.span.start())
                    && !retained.contains(&comment.span.start())
            })
            // Keep any closing inline header on the body side of the marker.
            // A block scalar's span can already end at the following line's
            // start; that line belongs to the suffix and must stay outside.
            .then(|| {
                if original.source[..end].ends_with(['\r', '\n']) {
                    end
                } else {
                    line_end(&original.source, end)
                }
            })
    }

    /// Comments already in the framing retain their source anchors. Comments
    /// within the body may depend on syntax removed by the final candidate.
    fn fixed(&self, comment: &Comment) -> bool {
        if comment.span.is_empty()
            || self.inline.moved.contains(&comment.span.start())
            || self.indented_suffixes.contains(&comment.span.start())
        {
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
            RootKind::BlockCollection {
                header,
                scalar_tail,
            } => match comment.placement {
                CommentPlacement::Leading | CommentPlacement::Trailing => false,
                CommentPlacement::Inline => header,
                CommentPlacement::Detached => {
                    !(scalar_tail && comment.position == CommentPosition::After)
                }
            },
        }
    }
}

pub(super) fn root_region(
    original: &EditableDocument,
    layout: &RootLayout,
    mut region: Span,
) -> Span {
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
            if let Some(next) = next.filter(|next| {
                next.placement == CommentPlacement::Inline && next.span.start() >= layout.body.end()
            }) {
                let at = original.source[..next.span.start()]
                    .rfind(['\r', '\n'])
                    .map_or(0, |index| index + 1);
                return Some(before_marker(at, comment.placement, newline));
            }
            let slot = layout.inline.end.partition_point(|&inline| inline <= index);
            if let Some(slot) = layout.inline.slots.get(slot).filter(|slot| {
                slot.marker().is_some_and(|marker| {
                    next.is_none_or(|next| marker.start() <= next.span.start())
                })
            }) {
                let marker = slot.marker()?;
                return Some(before_marker(marker.start(), comment.placement, newline));
            }
            let marker = original
                .end_marker
                .iter()
                .chain(&original.extra_end_markers)
                .rfind(|marker| next.is_none_or(|next| marker.end() <= next.span.start()))?;
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
            if let Ok(slot) = layout.inline.end.binary_search(&index) {
                let marker = layout.inline.slots.get(slot)?.marker()?;
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
            Some(Anchor {
                at: line_end(&original.source, marker.end()),
                before: " ".to_owned(),
                after: String::new(),
            })
        }
        _ => None,
    }
}

fn before_marker(at: usize, placement: CommentPlacement, newline: &str) -> Anchor {
    Anchor {
        at,
        before: if placement == CommentPlacement::Detached {
            newline.to_owned()
        } else {
            String::new()
        },
        after: newline.to_owned(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn framing_materializes_headers_without_changing_source_span_meaning() {
        let original = EditableDocument::parse("&a [ # open\n  *a\n] # close\n").unwrap();
        let mut candidate = original.clone();
        for text in [" added 1", " added 2", " added 3", " added 4"] {
            candidate.nodes[candidate.root.0].comments.push(Comment {
                text: text.to_owned(),
                placement: CommentPlacement::Inline,
                position: CommentPosition::After,
                span: Span::new(0, 0).unwrap(),
            });
        }
        let before = RootComments::plan(
            &original,
            &candidate,
            Some(emit::region(&original, original.root, true)),
            &RootLayout::new(&original, &candidate, &Layout::new(&candidate)),
        )
        .unwrap();
        let syntax = Layout::new(&candidate);
        let framed = prepare(&original, &mut candidate, &syntax).unwrap();
        for (old, new) in original.nodes.iter().zip(&framed.nodes) {
            assert_eq!(
                old.span.slice(&original.source),
                new.span.slice(&framed.source)
            );
            for (old, new) in old.comments.iter().zip(&new.comments) {
                assert_eq!(
                    old.span.slice(&original.source),
                    new.span.slice(&framed.source)
                );
            }
        }
        assert_eq!(candidate.source, framed.source);
        let after = RootComments::plan(
            &framed,
            &candidate,
            Some(emit::region(&framed, framed.root, true)),
            &RootLayout::new(&framed, &candidate, &syntax),
        )
        .unwrap();
        insta::assert_debug_snapshot!((before.rendered.len(), after.rendered.len(), framed.source), @r#"
        (
            6,
            3,
            "---\n&a [ # open\n  *a\n] # close\n...\n...\n",
        )
        "#);
    }
}
