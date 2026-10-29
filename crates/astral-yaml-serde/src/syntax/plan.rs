//! Choose source regions to rewrite by comparing the original and edited syntax.

use super::{
    Comment, Node, NodeKind,
    draft::Candidate,
    emit,
    framing::{RootComments, RootLayout},
    layout::Layout,
    lines::LineIndex,
    scope::Scopes,
    source::Source,
    splice::{self, Boundary},
    view::Syntax,
};
use astral_yaml::{Error, NodeId, Span, comments::CommentPlacement};
use std::{cell::Cell, collections::HashSet};

pub(super) struct Plan<'a> {
    pub(super) original: &'a Source,
    pub(super) candidate: Syntax<'a, Option<Span>>,
    pub(super) parents: &'a [Option<NodeId>],
    pub(super) changes: Changes,
    pub(super) work: &'a Cell<usize>,
}

/// A subtree and the source range reserved for its replacement.
pub(super) struct Rewrite<'a> {
    pub(super) id: NodeId,
    pub(super) boundary: Boundary<'a>,
    context: emit::Context,
}

/// Changes between the original and final syntax, excluding canceled edits.
pub(super) struct Changes {
    roots: HashSet<NodeId>,
    root_comments: bool,
}

impl Changes {
    pub(super) fn between(original: &Source, candidate: &Candidate, layout: &Layout) -> Self {
        let syntax = Syntax::from(candidate);
        let original_layout = Layout::new(original);
        let mut changes = Self {
            roots: HashSet::new(),
            root_comments: false,
        };
        // New nodes are owned by a retained occurrence whose child list changed.
        // Retired occurrences have no parent and cannot require a source rewrite.
        for (index, old) in original.nodes.iter().enumerate() {
            let id = NodeId::from_index(index);
            let parent = candidate.parents()[index];
            if parent.is_none() && id != syntax.root {
                continue;
            }
            let new = &syntax.nodes[index];
            let content = old.kind != new.kind || old.anchor != new.anchor || old.tag != new.tag;
            let comments = !same_comments(&old.comments, &new.comments);
            let resolved = original_layout.flow(id) != layout.flow(id)
                || match (&old.kind, &new.kind) {
                    (NodeKind::Scalar { style: old, .. }, NodeKind::Scalar { style: new, .. }) => {
                        original_layout.scalar_style(id, *old) != layout.scalar_style(id, *new)
                    }
                    (NodeKind::Alias { target: old }, NodeKind::Alias { target: new }) => {
                        original.nodes[old.index()].anchor != syntax.nodes[new.index()].anchor
                    }
                    _ => false,
                };
            if !content && !comments && !resolved {
                continue;
            }
            if id == syntax.root && !content && !resolved && only_outer_comments(old, new) {
                changes.root_comments = true;
                continue;
            }
            changes.roots.insert(id);
        }
        changes
    }

    pub(super) fn is_empty(&self) -> bool {
        self.roots.is_empty() && !self.root_comments
    }
}

fn same_comments(old: &[Comment], new: &[Comment<Option<Span>>]) -> bool {
    // Provenance controls rewrite boundaries, not whether the final state
    // changed. Removing and reinserting identical attachments is a no-op.
    old.len() == new.len()
        && old
            .iter()
            .map(Comment::attachment)
            .eq(new.iter().map(Comment::attachment))
}

/// Return whether only existing comments outside the node's span have changed.
/// New comments require a layout decision and cannot use this shortcut.
fn only_outer_comments(old: &Node, new: &Node<Option<Span>>) -> bool {
    let body = |span| super::contains(old.span, span);
    if new.comments.iter().any(|comment| comment.span.is_none()) {
        return false;
    }
    let old_body = old
        .comments
        .iter()
        .filter(|comment| body(comment.span))
        .map(|comment| (Some(comment.span), comment.attachment()));
    let new_body = new
        .comments
        .iter()
        .filter(|comment| comment.span.is_some_and(body))
        .map(|comment| (comment.span, comment.attachment()));
    old_body.eq(new_body)
}

impl Plan<'_> {
    fn charge(&self, amount: usize) -> Result<(), Error> {
        let span = self.original.nodes[self.original.root.index()].span;
        super::charge_work(self.work, amount, span)
    }

    /// Walk from a node to the root, including the node itself.
    fn ancestors(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        std::iter::successors(Some(id), |id| self.parents[id.index()])
    }

    fn body_region(&self, id: NodeId, layout: &RootLayout, scopes: &Scopes<'_>) -> Span {
        let mut region = scopes.body(id);
        if id != self.candidate.root {
            return region;
        }
        if emit::has_inline_header(&self.candidate.nodes[id.index()], false) {
            // A root's inline comment must move with its content
            // when the emitted layout puts it on a header. Framing
            // comments on other lines remain outside the rewrite.
            for comment in &self.original.nodes[id.index()].comments {
                if comment.placement == CommentPlacement::Inline
                    && comment.span.start() >= region.end()
                    && self.original.text[region.end()..comment.span.start()]
                        .bytes()
                        .all(|byte| matches!(byte, b' ' | b'\t'))
                {
                    region = super::span(region.start(), comment.span.end());
                }
            }
        }
        super::framing::root_region(self.original, layout, region)
    }

    pub(super) fn rewrite_regions(
        &self,
        layout: &RootLayout,
        syntax: &Layout,
    ) -> Result<Vec<Rewrite<'_>>, Error> {
        if self.changes.roots.is_empty() {
            return Ok(Vec::new());
        }
        let mut lines = LineIndex::new(&self.original.text);
        let scopes = Scopes::new(self.original, self.candidate, syntax, &mut lines);
        let mut ids: Vec<_> = self.changes.roots.iter().copied().collect();
        if layout.needs_suffix_context()
            || (layout.needs_scalar_boundary(self.candidate)
                && syntax.tail(self.candidate.root).is_some_and(|tail| {
                    self.ancestors(tail)
                        .any(|id| self.changes.roots.contains(&id))
                }))
        {
            ids.push(self.candidate.root);
        }
        loop {
            self.charge(self.original.nodes.len())?;
            let roots: HashSet<_> = ids.into_iter().map(|id| scopes.select(id)).collect();
            ids = roots
                .iter()
                .copied()
                .filter(|&id| !self.ancestors(id).skip(1).any(|id| roots.contains(&id)))
                .collect();
            let mut rewrites: Vec<_> = ids
                .iter()
                .map(|&id| {
                    let span = self.body_region(id, layout, &scopes);
                    let line = lines.at(span.start());
                    Rewrite {
                        id,
                        context: scopes.context(id, line.indent(span.start())),
                        boundary: Boundary::new(
                            &self.original.text,
                            span,
                            line,
                            id == self.candidate.root
                                && emit::omitted(&self.candidate.nodes[id.index()]),
                            self.original.end_marker,
                        ),
                    }
                })
                .collect();
            rewrites.sort_by_key(|rewrite| rewrite.boundary.reserved().start());
            let Some(pair) = rewrites.windows(2).find(|pair| {
                pair[0].boundary.reserved().end() > pair[1].boundary.reserved().start()
            }) else {
                return Ok(rewrites);
            };
            let (left, right) = (pair[0].id, pair[1].id);
            let parent = self
                .ancestors(left)
                .find(|&parent| self.ancestors(right).any(|id| id == parent))
                .expect("common document root");
            ids.push(parent);
        }
    }

    fn root_comments(
        &self,
        region: Option<Span>,
        layout: &RootLayout,
    ) -> Result<RootComments, Error> {
        self.charge(
            self.original.nodes[self.original.root.index()]
                .comments
                .len()
                + self.candidate.nodes[self.candidate.root.index()]
                    .comments
                    .len(),
        )?;
        RootComments::plan(self.original, self.candidate, region, layout)
    }

    pub(super) fn render(self, syntax: &Layout) -> Result<String, Error> {
        let layout = RootLayout::new(self.original, self.candidate, syntax);
        let regions = self.rewrite_regions(&layout, syntax)?;
        let root_region = regions
            .iter()
            .find(|rewrite| rewrite.id == self.candidate.root);
        let mut root_comments = if root_region.is_some() || self.changes.root_comments {
            let region = root_region.map(|rewrite| rewrite.boundary.body());
            Some(self.root_comments(region, &layout)?)
        } else {
            None
        };
        let mut patches = root_comments
            .as_mut()
            .map_or_else(Vec::new, |comments| std::mem::take(&mut comments.patches));
        let emission = self.original.emission(self.candidate);
        for rewrite in &regions {
            let id = rewrite.id;
            let comments = if id == self.candidate.root {
                &root_comments.as_ref().expect("root comment plan").rendered
            } else {
                &self.candidate.nodes[id.index()].comments
            };
            let rendered = splice::render_with_comments(
                emission,
                id,
                &rewrite.boundary,
                rewrite.context,
                comments,
                syntax,
            )?;
            patches.push((rendered.span, rendered.text));
        }
        if let Some(comments) = root_comments.as_mut() {
            patches.append(&mut comments.after_body);
            comments.finish_separation(self.original, &mut patches);
        }
        splice::apply(
            self.original,
            patches,
            self.original.nodes[self.original.root.index()].span,
        )
    }
}
