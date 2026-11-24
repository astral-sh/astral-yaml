//! Choose source regions to rewrite by comparing the original and edited syntax.

use super::draft::Candidate;
use super::framing::{RootComments, RootLayout};
use super::layout::Layout;
use super::lines::LineIndex;
use super::scope::Scopes;
use super::source::Source;
use super::splice::{self, Boundary};
use super::view::Syntax;
use super::{Comment, Node, NodeKind, emit};
use astral_yaml::{Budget, Error, NodeId, Span};
use std::collections::HashSet;

pub(super) struct Plan<'plan> {
    pub(super) original: &'plan Source,
    pub(super) candidate: Syntax<'plan, Option<Span>>,
    pub(super) parents: &'plan [Option<NodeId>],
    pub(super) changes: Changes,
    pub(super) work: &'plan Budget,
}

/// A subtree and the source range reserved for its replacement.
pub(super) struct Rewrite<'source> {
    pub(super) id: NodeId,
    pub(super) boundary: Boundary<'source>,
    start: splice::Start,
}

/// Changes between the original and final syntax, excluding canceled edits.
pub(super) struct Changes {
    pub(super) roots: HashSet<NodeId>,
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
            let content = old.kind != new.kind || old.tag != new.tag;
            let comments = !same_comments(&old.comments, &new.comments);
            let resolved = original_layout.flow(id) != layout.flow(id)
                || match (&old.kind, &new.kind) {
                    (NodeKind::Scalar { style: old, .. }, NodeKind::Scalar { style: new, .. }) => {
                        original_layout.scalar_style(id, *old) != layout.scalar_style(id, *new)
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
                    let mut span = scopes.body(id);
                    if id == self.candidate.root {
                        span = layout.rewrite_region(self.original, span);
                    }
                    let line = lines.at(span.start());
                    Rewrite {
                        id,
                        start: scopes.start(id, line, span.start()),
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
            self.root_comments(region, &layout)?
        } else {
            RootComments::default()
        };
        let mut patches = std::mem::take(&mut root_comments.patches);
        // All replacement text remains live until the source patches are joined.
        // Include comment patches on either side of the document body.
        let comment_bytes = patches
            .iter()
            .chain(&root_comments.after_body)
            .map(|(_, text)| text.len())
            .sum();
        let retained_bytes = Budget::new(
            self.original
                .options
                .max_input_bytes
                .saturating_sub(comment_bytes),
        );
        let emission = self.original.emission(self.candidate);
        for rewrite in &regions {
            let id = rewrite.id;
            let comments = if id == self.candidate.root {
                &root_comments.rendered
            } else {
                &self.candidate.nodes[id.index()].comments
            };
            let rendered = splice::render_with_comments(
                emission,
                id,
                &rewrite.boundary,
                rewrite.start,
                comments,
                syntax,
            )?;
            retained_bytes
                .try_charge(rendered.text.len())
                .ok_or(Error {
                    span: rewrite.boundary.body(),
                    message: "emitted YAML exceeds input byte limit",
                })?;
            patches.push((rendered.span, rendered.text));
        }
        patches.append(&mut root_comments.after_body);
        root_comments.finish_separation(self.original, &mut patches);
        splice::apply(
            self.original,
            patches,
            self.original.nodes[self.original.root.index()].span,
        )
    }
}
