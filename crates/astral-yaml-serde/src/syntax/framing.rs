//! Place root comments around document markers and directives.
//! Plan markers separately so removing a comment cannot remove a marker needed
//! by another comment.

use super::layout::Layout;
use super::lines::Line;
use super::source::{Frame, Prepared, Source};
use super::splice::{self, OuterComments};
use super::view::Syntax;
use super::{Comment, CommentPosition, emit};
use astral_yaml::comments::CommentPlacement;
use astral_yaml::{Error, Span};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub(super) struct RootComments {
    pub(super) rendered: Vec<Comment<Option<Span>>>,
    pub(super) patches: Vec<(Span, String)>,
    /// Patches inserted after an empty root's replacement at the same source offset.
    pub(super) after_body: Vec<(Span, String)>,
    /// End of the root's trailing comments, for removing intervening blank lines.
    trailing_end: Option<usize>,
    leading_start: Option<usize>,
}

/// A comment insertion point and the text needed on either side.
struct Anchor {
    at: usize,
    before: String,
    after: String,
}

/// A retained comment with a known source span.
#[derive(Clone, Copy)]
struct LocatedComment<'comments> {
    comment: &'comments Comment<Option<Span>>,
    span: Span,
}

/// Insert document markers required by the edited root and adjust source spans.
/// Return `None` if no new framing is needed.
pub(super) fn prepare(
    original: &Source,
    candidate: Syntax<'_, Option<Span>>,
    syntax: &Layout,
) -> Option<Prepared> {
    let layout = RootLayout::new(original, candidate, syntax);
    let root = &candidate.nodes[candidate.root.index()];
    let needs_prefix_boundary =
        matches!(layout.kind, RootKind::BlockCollection { header: false, .. });
    let start = original.start_marker.is_none()
        && (matches!(layout.kind, RootKind::Omitted)
            || (needs_prefix_boundary
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
                None => Some(removed_boundary.unwrap_or(original.text.len())),
                Some(marker) => root
                    .comments
                    .iter()
                    .position(|comment| {
                        comment.placement == CommentPlacement::Inline
                            && comment.span.is_some_and(|span| {
                                span.start() >= marker.end()
                                    && span.end() <= line_end(&original.text, marker.end())
                            })
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
    let root_start = splice::region(original, original.root, OuterComments::Exclude).start();
    let root_line = (layout.inline.start.is_some()
        || syntax.separate_root()
        || (original.nodes[original.root.index()].span.is_empty() && !insertions.is_empty()))
        && original
            .start_marker
            .is_some_and(|marker| line_end(&original.text, marker.end()) >= root_start);
    if !start && !root_line && insertions.is_empty() {
        return None;
    }

    let mut frame = Frame::new(original);
    frame.end_markers(insertions);
    if root_line {
        frame.separate_root(root_start);
    }
    if start {
        // Prefix comments and blank lines keep their relation to the boundary.
        let at = root_start - Line::at(&original.text, root_start).column(root_start);
        frame.start_marker(at);
    }
    Some(frame.finish(candidate))
}

impl RootComments {
    pub(super) fn plan(
        original: &Source,
        candidate: Syntax<'_, Option<Span>>,
        region: Option<Span>,
        layout: &RootLayout,
    ) -> Result<Self, Error> {
        let comments = &candidate.nodes[candidate.root.index()].comments;
        let Some(region) = region else {
            return Ok(Self::retained_body(original, comments));
        };
        let mut following = vec![None; comments.len()];
        let mut next = None;
        for (index, comment) in comments.iter().enumerate().rev() {
            following[index] = next;
            if let Some(span) = layout.fixed(comment) {
                next = Some(LocatedComment { comment, span });
            }
        }

        let mut plan = Self::default();
        let mut previous = None;
        let mut previous_position = 0;
        let mut bytes = 0usize;
        for (index, comment) in comments.iter().enumerate() {
            if let Some(span) = layout.fixed(comment) {
                if comment.placement == CommentPlacement::Trailing {
                    plan.trailing_end = Some(span.start());
                } else if comment.placement == CommentPlacement::Leading {
                    plan.leading_start = Some(span.end());
                }
                previous = Some(LocatedComment { comment, span });
                previous_position = span.end();
                plan.rendered.push(comment.clone());
                continue;
            }
            let next = following[index];
            let previous_comment = previous;
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
            // by finish_separation when headers move past them.
            if comment.placement == CommentPlacement::Detached
                && comment.position == CommentPosition::After
                && anchor.before.is_empty()
                && plan
                    .trailing_end
                    .is_some_and(|end| !blank_between(&original.text, end, anchor.at))
            {
                anchor.before.push('\n');
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
            } else if comment.placement == CommentPlacement::Leading {
                plan.leading_start = Some(anchor.at);
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
            patches.push((super::span(anchor.at, anchor.at), text));
        }
        plan.changed_comments(original, Some(region));
        plan.retain_prefix_separation(original, layout, region);
        if let Some(marker) = layout.suffix_boundary {
            plan.retain_suffix_separation(original, marker);
        }
        Ok(plan)
    }

    fn retained_body(original: &Source, comments: &[Comment<Option<Span>>]) -> Self {
        // No body rewrite means its existing layout remains authoritative.
        // Text-only framing edits must not relocate unchanged body comments.
        let mut plan = Self {
            rendered: comments.to_vec(),
            // Descendant rewrites can introduce an end marker even when
            // the root body stays in place. Its retained trailing comments
            // still require adjacency through the final suffix patches.
            trailing_end: comments
                .iter()
                .filter(|comment| comment.placement == CommentPlacement::Trailing)
                .filter_map(|comment| comment.span.map(Span::start))
                .max(),
            ..Self::default()
        };
        plan.changed_comments(original, None);
        plan
    }

    /// Remove blank lines between the root and its leading or trailing comments.
    /// Skip regions already covered by other patches.
    pub(super) fn finish_separation(&self, original: &Source, patches: &mut Vec<(Span, String)>) {
        let root = original.nodes[original.root.index()].span;
        if let Some(start) = self.leading_start {
            Self::remove_blank_lines(original, start, root.start(), patches);
        }
        if let Some(end) = self.trailing_end {
            Self::remove_blank_lines(original, root.end(), end, patches);
        }
    }

    fn remove_blank_lines(
        original: &Source,
        mut at: usize,
        end: usize,
        patches: &mut Vec<(Span, String)>,
    ) {
        let source = &original.text;
        let mut covered: Vec<_> = patches.iter().map(|(span, _)| *span).collect();
        covered.sort_by_key(|span| (span.start(), span.end()));
        while at < end {
            let line_end = line_end(source, at);
            let next = line_end + usize::from(line_end < source.len());
            let whole_line = at == 0 || source[..at].ends_with('\n');
            let index = covered.partition_point(|span| span.end() <= at);
            let owned = covered.get(index).is_some_and(|span| span.start() < next);
            if whole_line && next <= end && !owned && blank(&source[at..line_end]) {
                patches.push((super::span(at, next), String::new()));
            }
            at = next;
        }
    }

    /// A retained prefix can be detached by indentation above a block collection.
    /// Give it a blank line when the new root uses a token or flow delimiter.
    fn retain_prefix_separation(&mut self, original: &Source, layout: &RootLayout, region: Span) {
        if !matches!(layout.kind, RootKind::Token | RootKind::FlowCollection) {
            return;
        }
        let last = self.rendered.iter().rev().find_map(|comment| {
            comment.span.filter(|span| {
                comment.placement == CommentPlacement::Detached
                    && comment.position == CommentPosition::Before
                    && span.end() <= region.start()
            })
        });
        if let Some(span) = last
            && !blank_between(&original.text, span.end(), region.start())
        {
            let at = span.end();
            self.patches.push((super::span(at, at), "\n".to_owned()));
        }
    }

    /// Keep a detached comment separated from a rewritten body.
    /// If rewriting would consume its blank line before the end marker, add one on
    /// the retained side of the marker.
    fn retain_suffix_separation(&mut self, original: &Source, marker: Span) {
        let Some(span) = self.rendered.iter().find_map(|comment| {
            comment.span.filter(|span| {
                comment.placement == CommentPlacement::Detached && span.start() >= marker.end()
            })
        }) else {
            return;
        };
        let after = self.trailing_end.unwrap_or(0).max(marker.end());
        if !blank_between(&original.text, after, span.start()) {
            let start = original.text[..span.start()]
                .rfind('\n')
                .map_or(0, |index| index + 1);
            self.patches
                .push((super::span(start, start), "\n".to_owned()));
        }
    }

    /// Create patches for comments that changed or disappeared outside `region`.
    /// Compare the final comments to the original source, regardless of intervening edits.
    fn changed_comments(&mut self, original: &Source, region: Option<Span>) {
        let retained: HashMap<_, _> = self
            .rendered
            .iter()
            .filter_map(|comment| comment.span.map(|span| (span.start(), comment)))
            .collect();
        for comment in &original.nodes[original.root.index()].comments {
            if region.is_some_and(|region| super::contains(region, comment.span)) {
                continue;
            }
            match retained.get(&comment.span.start()) {
                Some(current) if current.text != comment.text => {
                    self.patches
                        .push((comment.span, format!("#{}", current.text)));
                }
                None => self
                    .patches
                    .push((removal_span(&original.text, comment.span), String::new())),
                _ => {}
            }
        }
    }
}

/// Root syntax that determines whether comments need a document marker.
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

    fn needs_suffix_boundary(self, comment: &Comment<Option<Span>>) -> bool {
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

/// Headers used by the root's inline comments.
/// Use the root's own headers first, then document markers, preserving comment order.
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
        original: &Source,
        root: &super::Node<Option<Span>>,
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
                    && comment.span.is_none_or(|span| super::contains(body, span)))
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
                comment.span.is_some_and(|span| span.start() >= body.end())
                    || comment.placement == CommentPlacement::Trailing
                    || (comment.placement == CommentPlacement::Detached
                        && comment.position == CommentPosition::After)
            })
            .unwrap_or(root.comments.len());
        // An existing opening-delimiter comment remains at that delimiter. Properties
        // before it therefore cannot host later inline comments.
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
        let mut moved = removed_inline_comments(original, root, body);
        moved.extend(
            start
                .iter()
                .chain(&end)
                .filter_map(|&index| root.comments[index].span.map(Span::start)),
        );
        if shared_headers {
            // Reassign body headers together: a retained closing comment must
            // not remain outside the rewrite when it becomes an opening one.
            moved.extend(root.comments.iter().filter_map(|comment| {
                let span = comment.span?;
                (comment.placement == CommentPlacement::Inline && super::contains(body, span))
                    .then_some(span.start())
            }));
        }
        if let Some(&first) = end.first() {
            // Comments following a relocated suffix header must follow it to
            // the same boundary, rather than remain earlier in the old body.
            moved.extend(root.comments[first..].iter().filter_map(|comment| {
                comment
                    .span
                    .filter(|&span| super::contains(body, span))
                    .map(Span::start)
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

/// Include removed inline comments in the body rewrite.
/// A separate removal patch would leave an extra blank line before retained comments.
fn removed_inline_comments(
    original: &Source,
    root: &super::Node<Option<Span>>,
    body: Span,
) -> HashSet<usize> {
    let retained: HashSet<_> = root
        .comments
        .iter()
        .filter_map(|comment| comment.span)
        .collect();
    original.nodes[original.root.index()]
        .comments
        .iter()
        .filter_map(|comment| {
            (comment.placement == CommentPlacement::Inline
                && super::contains(body, comment.span)
                && !retained.contains(&comment.span))
            .then_some(comment.span.start())
        })
        .collect()
}

/// Assign end markers to suffix comments in their edited order.
/// Insert missing markers before the next retained comment, or at EOF.
fn end_slots(
    original: &Source,
    root: &super::Node<Option<Span>>,
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
        if let Some(span) = comment.span.filter(|&span| !super::contains(body, span)) {
            next = Some(span);
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
            let next = following[index].map_or(original.text.len(), Span::start);
            if free.peek().is_some_and(|marker| marker.end() <= next) {
                slots.push(EndSlot::Retained(free.next().expect("available marker")));
            } else {
                let at = if following[index].is_some() {
                    original.text[..next]
                        .rfind('\n')
                        .map_or(0, |index| index + 1)
                } else {
                    original.text.len()
                };
                slots.push(EndSlot::Missing(at));
            }
        }
        if let Some(span) = comment.span.filter(|&span| !super::contains(body, span)) {
            previous = span.end();
        }
    }
    slots
}

/// Find document markers that already have an inline comment.
/// Search the ordered marker list to avoid rescanning comments for each marker.
fn occupied_markers(
    original: &Source,
    root: &super::Node<Option<Span>>,
    body: Span,
) -> HashSet<usize> {
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
            let span = comment.span?;
            if comment.placement != CommentPlacement::Inline || super::contains(body, span) {
                return None;
            }
            let index = markers.partition_point(|marker| marker.end() <= span.start());
            let marker = markers.get(index.checked_sub(1)?)?;
            (!original.text[marker.end()..span.start()].contains('\n')).then_some(marker.start())
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
        original: &Source,
        candidate: Syntax<'_, Option<Span>>,
        syntax: &Layout,
    ) -> Self {
        let root_start = original.nodes[original.root.index()].span.start();
        let start = original.start_marker.map_or(0, |marker| {
            line_end(&original.text, marker.end()).min(root_start)
        });
        let end = original.end_marker.map_or(original.text.len(), Span::start);
        // Framing can insert `...` before `---` to host an inline comment.
        // In that case the interval between the markers contains no body comments.
        let body = super::span(start.min(end), end);
        let root = &candidate.nodes[candidate.root.index()];
        let kind = if emit::omitted(root) {
            RootKind::Omitted
        } else {
            match root.kind {
                super::NodeKind::Scalar {
                    style: astral_yaml::ScalarStyle::Literal | astral_yaml::ScalarStyle::Folded,
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
        let indent = splice::indent(&original.text, root_start);
        let indented_suffixes = root
            .comments
            .iter()
            .filter_map(|comment| {
                let span = comment.span?;
                (matches!(kind, RootKind::BlockCollection { .. })
                    && comment.placement == CommentPlacement::Detached
                    && comment.position == CommentPosition::After
                    && super::contains(body, span)
                    && splice::indent(&original.text, span.start()) > indent)
                    .then_some(span.start())
            })
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

    pub(super) fn needs_scalar_boundary(&self, candidate: Syntax<'_, Option<Span>>) -> bool {
        self.kind.ends_with_block_scalar()
            && candidate.nodes[candidate.root.index()]
                .comments
                .iter()
                .any(|comment| self.kind.needs_suffix_boundary(comment))
    }

    pub(super) fn needs_suffix_context(&self) -> bool {
        !self.indented_suffixes.is_empty()
    }

    /// Find where an end marker is needed after removing a suffix comment.
    /// Its whitespace must not become block-scalar content. Use the final layout
    /// because the same edit can introduce the block scalar. An existing end marker
    /// bounds the search.
    fn removed_scalar_boundary(
        &self,
        original: &Source,
        root: &super::Node<Option<Span>>,
    ) -> Option<usize> {
        if !self.kind.ends_with_block_scalar() {
            return None;
        }
        // Descendant suffix comments can extend past the root's token span.
        // Keep a generated boundary outside the entire subtree rewrite so
        // removing those comments cannot erase the marker with them.
        let end = splice::region(original, original.root, OuterComments::Exclude).end();
        let boundary = original.end_marker.map_or(original.text.len(), Span::start);
        let retained: HashSet<_> = root
            .comments
            .iter()
            .filter_map(|comment| comment.span.map(Span::start))
            .collect();
        original.nodes[original.root.index()]
            .comments
            .iter()
            .any(|comment| {
                (end..boundary).contains(&comment.span.start())
                    && !retained.contains(&comment.span.start())
            })
            // Keep any closing inline header on the body side of the marker. A block
            // scalar's span can already end at the following line's start. That line
            // belongs to the suffix and must stay outside.
            .then(|| {
                if original.text[..end].ends_with('\n') {
                    end
                } else {
                    line_end(&original.text, end)
                }
            })
    }

    /// Return the original span if a comment can stay in place after the edit.
    /// Comments tied to replaced syntax may need to be rendered with the new body.
    fn fixed<Location: Copy + Into<Option<Span>>>(
        &self,
        comment: &Comment<Location>,
    ) -> Option<Span> {
        let span = comment.span.into()?;
        if self.inline.moved.contains(&span.start())
            || self.indented_suffixes.contains(&span.start())
        {
            return None;
        }
        if !super::contains(self.body, span) {
            return Some(span);
        }
        if comment.placement == CommentPlacement::Leading && self.inline.start.is_some() {
            return None;
        }
        let fixed = match self.kind {
            RootKind::Token => true,
            RootKind::Omitted => {
                span.start() < self.root_start
                    && (comment.placement != CommentPlacement::Leading
                        || self.inline.end.is_empty())
            }
            // Indented comments can become block-scalar text. Rendering them together
            // supplies the indentation boundary after the scalar.
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
        };
        fixed.then_some(span)
    }
}

pub(super) fn root_region(original: &Source, layout: &RootLayout, mut region: Span) -> Span {
    for comment in &original.nodes[original.root.index()].comments {
        if layout.fixed(comment).is_none() {
            region = super::span(
                region.start().min(comment.span.start()),
                region.end().max(comment.span.end()),
            );
        }
    }
    if let Some(marker) = layout.suffix_boundary {
        // Trailing comments require adjacency to the root even when the end
        // marker hosts them. Omitted roots and block scalars also own these
        // separators. Render the complete body to establish that boundary.
        region = super::span(region.start(), region.end().max(marker.start()));
    }
    region
}

fn comment_anchor(
    original: &Source,
    region: Span,
    comment: &Comment<Option<Span>>,
    anchor: LocatedComment<'_>,
    before: bool,
) -> Option<Anchor> {
    if super::contains(region, anchor.span) {
        return None;
    }
    let breaks = outer_comment_breaks(comment, anchor.comment, before)?;
    let separator = format!(
        "{}{}",
        "\n".repeat(breaks),
        " ".repeat(splice::indent(&original.text, anchor.span.start()))
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
    original: &Source,
    layout: &RootLayout,
    comment: &Comment<Option<Span>>,
    index: usize,
    region: Span,
    next: Option<LocatedComment<'_>>,
) -> Option<Anchor> {
    match (comment.placement, comment.position) {
        (CommentPlacement::Leading, _)
        | (CommentPlacement::Detached, CommentPosition::Before | CommentPosition::Header) => {
            // Prefix comments share a boundary even when none retain source
            // spans. Moving only the leading comments would reverse their order.
            let marker = original.start_marker?;
            let collection = matches!(
                layout.kind,
                RootKind::FlowCollection | RootKind::BlockCollection { .. }
            );
            let precedes_header = next.is_some_and(|next| next.span.end() <= region.start())
                || layout.inline.start.is_some_and(|header| index < header);
            if !collection
                && !precedes_header
                && line_end(&original.text, marker.end()) < region.start()
            {
                return None;
            }
            Some(Anchor {
                at: marker.start(),
                before: String::new(),
                after: marker_separation(comment.placement),
            })
        }
        (CommentPlacement::Trailing, _) | (CommentPlacement::Detached, CommentPosition::After) => {
            if let Some(next) = next.filter(|next| {
                next.comment.placement == CommentPlacement::Inline
                    && next.span.start() >= layout.body.end()
            }) {
                let at = original.text[..next.span.start()]
                    .rfind('\n')
                    .map_or(0, |index| index + 1);
                return Some(before_marker(at, comment.placement));
            }
            let slot = layout.inline.end.partition_point(|&inline| inline <= index);
            if let Some(slot) = layout.inline.slots.get(slot).filter(|slot| {
                slot.marker().is_some_and(|marker| {
                    next.is_none_or(|next| marker.start() <= next.span.start())
                })
            }) {
                let marker = slot.marker()?;
                return Some(before_marker(marker.start(), comment.placement));
            }
            let marker = original
                .end_marker
                .iter()
                .chain(&original.extra_end_markers)
                .rfind(|marker| next.is_none_or(|next| marker.end() <= next.span.start()))?;
            let at = line_end(&original.text, marker.end());
            Some(Anchor {
                at,
                before: marker_separation(comment.placement),
                after: String::new(),
            })
        }
        _ => {
            // Only inline comments have slots on document markers.
            let marker = if layout.inline.start == Some(index) {
                original.start_marker?
            } else {
                let slot = layout.inline.end.binary_search(&index).ok()?;
                layout.inline.slots.get(slot)?.marker()?
            };
            Some(Anchor {
                at: line_end(&original.text, marker.end()),
                before: " ".to_owned(),
                after: String::new(),
            })
        }
    }
}

fn marker_separation(placement: CommentPlacement) -> String {
    if placement == CommentPlacement::Detached {
        "\n\n".to_owned()
    } else {
        "\n".to_owned()
    }
}

fn before_marker(at: usize, placement: CommentPlacement) -> Anchor {
    Anchor {
        at,
        before: if placement == CommentPlacement::Detached {
            "\n".to_owned()
        } else {
            String::new()
        },
        after: "\n".to_owned(),
    }
}

fn line_end(source: &str, start: usize) -> usize {
    source[start..]
        .find('\n')
        .map_or(source.len(), |offset| start + offset)
}

/// A stream mark occupies no indentation column on a framing line.
fn blank(line: &str) -> bool {
    line.trim_start_matches('\u{feff}')
        .bytes()
        .all(|byte| matches!(byte, b' ' | b'\t'))
}

fn blank_between(source: &str, before: usize, after: usize) -> bool {
    let mut end = line_end(source, before);
    while end < after {
        let start = end + 1;
        end = line_end(source, start);
        if end < after && blank(&source[start..end]) {
            return true;
        }
    }
    false
}

fn removal_span(source: &str, span: Span) -> Span {
    // Removing a standalone comment includes its line break, so adjacent
    // leading/trailing comments do not become detached by an empty line.
    let start = source[..span.start()]
        .rfind('\n')
        .map_or(0, |index| index + 1);
    if blank(&source[start..span.start()]) {
        let suffix = &source[span.end()..];
        let terminator = usize::from(suffix.starts_with('\n'));
        super::span(start, span.end() + terminator)
    } else {
        span
    }
}
/// Return the line breaks needed to insert `comment` beside `anchor`.
/// Return `None` if it must be rendered with the node instead to preserve placement.
fn outer_comment_breaks(
    comment: &Comment<Option<Span>>,
    anchor: &Comment<Option<Span>>,
    before: bool,
) -> Option<usize> {
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
    use crate::syntax::draft::Draft;
    use astral_yaml_dev::{edit_diff, indoc};

    #[test]
    fn framing_materializes_headers_without_changing_source_span_meaning() {
        let source = indoc! {"
            &a [ # open
              *a
            ] # close
        "};
        let document = super::super::DocumentSyntax::parse(source).unwrap();
        let original = document.source.clone();
        let mut candidate = Draft::from(&document);
        for text in [" added 1", " added 2", " added 3", " added 4"] {
            candidate
                .insert_comment(
                    candidate.root(),
                    candidate.node(candidate.root()).comments.len(),
                    text,
                    CommentPlacement::Inline,
                )
                .unwrap();
        }
        let candidate = candidate.freeze().unwrap();
        let candidate = Syntax::from(&candidate);
        let before = RootComments::plan(
            &original,
            candidate,
            Some(splice::region(
                &original,
                original.root,
                OuterComments::Include,
            )),
            &RootLayout::new(&original, candidate, &Layout::new(candidate)),
        )
        .unwrap();
        let syntax = Layout::new(candidate);
        let prepared = prepare(&original, candidate, &syntax).unwrap();
        let framed = &prepared.source;
        for (old, new) in original.nodes.iter().zip(&framed.nodes) {
            assert_eq!(old.span.slice(&original.text), new.span.slice(&framed.text));
            for (old, new) in old.comments.iter().zip(&new.comments) {
                assert_eq!(old.span.slice(&original.text), new.span.slice(&framed.text));
            }
        }
        for (old, new) in original.contents.iter().zip(&framed.contents) {
            assert_eq!(old.slice(&original.text), new.slice(&framed.text));
        }
        // The same mapping relocates rendering provenance. Generated syntax
        // stays source-less, and every retained location still names its text.
        for (old, new) in candidate.nodes.iter().zip(prepared.syntax().nodes) {
            assert_eq!(
                old.span.map(|span| span.slice(&original.text)),
                new.span.map(|span| span.slice(&framed.text)),
            );
            for (old, new) in old.comments.iter().zip(&new.comments) {
                assert_eq!(
                    old.span.map(|span| span.slice(&original.text)),
                    new.span.map(|span| span.slice(&framed.text)),
                );
            }
        }
        let after = RootComments::plan(
            framed,
            prepared.syntax(),
            Some(splice::region(framed, framed.root, OuterComments::Include)),
            &RootLayout::new(framed, prepared.syntax(), &syntax),
        )
        .unwrap();
        insta::assert_snapshot!(format!(
            "Retained headers: {} -> {}\n{}",
            before.rendered.len(), after.rendered.len(), edit_diff(source, &framed.text),
        ), @"
        Retained headers: 6 -> 3
        @@ -1,3 +1,6 @@
        +---
         &a [ # open
           *a
         ] # close
        +...
        +...
        ");
    }
}
