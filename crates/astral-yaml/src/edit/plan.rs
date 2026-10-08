//! Plan source rewrites from the immutable candidate and original source index.

use super::{
    Comment, Node, NodeKind,
    draft::Candidate,
    emit,
    framing::{RootComments, RootLayout},
    layout::Layout,
    scope::Scopes,
    source::Source,
    splice::{self, Boundary, PatchSet},
    view::Syntax,
};
use crate::{Error, NodeId, Span, comments::CommentPlacement};
use std::{cell::Cell, collections::HashSet};

pub(super) struct Plan<'a> {
    pub(super) original: &'a Source,
    pub(super) candidate: Syntax<'a, Option<Span>>,
    pub(super) parents: &'a [Option<NodeId>],
    pub(super) changes: Changes,
    pub(super) work: &'a Cell<usize>,
}

/// A subtree and its source boundary, shared by overlap checks and rendering.
pub(super) struct Rewrite<'a> {
    pub(super) id: NodeId,
    pub(super) boundary: Boundary<'a>,
    context: emit::Context,
}

/// Differences from the final candidate; canceled edits produce no rewrites.
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
            let id = NodeId(index);
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
                        original.nodes[old.0].anchor != syntax.nodes[new.0].anchor
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
        && old.iter().zip(new).all(|(old, new)| {
            old.text == new.text && old.placement == new.placement && old.position == new.position
        })
}

/// Existing comments outside the root token can be edited in the framing while
/// its body retains its source. Generated comments require a body/layout plan.
fn only_outer_comments(old: &Node, new: &Node<Option<Span>>) -> bool {
    let body = |span: Span| span.start() >= old.span.start() && span.end() <= old.span.end();
    if new.comments.iter().any(|comment| comment.span.is_none()) {
        return false;
    }
    let old_body = old
        .comments
        .iter()
        .filter(|comment| body(comment.span))
        .map(|comment| {
            (
                Some(comment.span),
                &comment.text,
                comment.placement,
                comment.position,
            )
        });
    let new_body = new
        .comments
        .iter()
        .filter(|comment| comment.span.is_some_and(body))
        .map(|comment| {
            (
                comment.span,
                &comment.text,
                comment.placement,
                comment.position,
            )
        });
    old_body.eq(new_body)
}

impl Plan<'_> {
    fn error(&self, message: &'static str) -> Error {
        Error {
            span: self.original.nodes[self.original.root.0].span,
            message,
        }
    }

    fn charge(&self, amount: usize) -> Result<(), Error> {
        let remaining = self
            .work
            .get()
            .checked_sub(amount)
            .ok_or_else(|| self.error("editing resource limit exceeded"))?;
        self.work.set(remaining);
        Ok(())
    }

    fn ancestor(&self, ancestor: NodeId, mut node: NodeId) -> bool {
        loop {
            if ancestor == node {
                return true;
            }
            let Some(parent) = self.parents[node.0] else {
                return false;
            };
            node = parent;
        }
    }

    fn body_region(&self, id: NodeId, layout: &RootLayout, scopes: &Scopes<'_>) -> Span {
        let mut region = scopes.body(id);
        if id == self.candidate.root && emit::has_inline_header(&self.candidate.nodes[id.0], false)
        {
            // A root's inline comment must move with its content
            // when the emitted layout puts it on a header. Framing
            // comments on other lines remain outside the rewrite.
            for comment in &self.original.nodes[id.0].comments {
                if comment.placement == CommentPlacement::Inline
                    && comment.span.start() >= region.end()
                    && self.original.text[region.end()..comment.span.start()]
                        .bytes()
                        .all(|byte| matches!(byte, b' ' | b'\t'))
                {
                    region.end = comment.span.end();
                }
            }
        }
        if id == self.candidate.root {
            region = super::framing::root_region(self.original, layout, region);
        }
        region
    }

    pub(super) fn rewrite_regions(
        &self,
        layout: &RootLayout,
        syntax: &Layout,
    ) -> Result<Vec<Rewrite<'_>>, Error> {
        let scopes = Scopes::new(self.original, self.candidate, syntax);
        let mut ids: Vec<_> = self.changes.roots.iter().copied().collect();
        if (!ids.is_empty() && layout.needs_suffix_context())
            || (layout.needs_scalar_boundary(self.candidate)
                && syntax
                    .tail(self.candidate.root)
                    .is_some_and(|tail| ids.iter().any(|&id| self.ancestor(id, tail))))
        {
            ids.push(self.candidate.root);
        }
        loop {
            if !ids.is_empty() {
                self.charge(self.original.nodes.len())?;
            }
            let roots: HashSet<_> = ids.into_iter().map(|id| scopes.select(id)).collect();
            ids = roots
                .iter()
                .copied()
                .filter(|&id| {
                    let mut current = self.parents[id.0];
                    while let Some(parent) = current {
                        if roots.contains(&parent) {
                            return false;
                        }
                        current = self.parents[parent.0];
                    }
                    true
                })
                .collect();
            let regions: Vec<_> = ids
                .iter()
                .map(|&id| {
                    let region = self.body_region(id, layout, &scopes);
                    (id, region)
                })
                .collect();

            let mut rewrites: Vec<_> = regions
                .into_iter()
                .map(|(id, span)| Rewrite {
                    id,
                    context: scopes.context(id, span),
                    boundary: Boundary::new(
                        &self.original.text,
                        span,
                        id == self.candidate.root && emit::omitted(&self.candidate.nodes[id.0]),
                        self.original.end_marker,
                    ),
                })
                .collect();
            rewrites.sort_by_key(|rewrite| rewrite.boundary.reserved().start());
            let Some(pair) = rewrites.windows(2).find(|pair| {
                pair[0].boundary.reserved().end() > pair[1].boundary.reserved().start()
            }) else {
                return Ok(rewrites);
            };
            let (left, right) = (pair[0].id, pair[1].id);
            let mut parent = left;
            while !self.ancestor(parent, right) {
                parent = self.parents[parent.0].expect("common document root");
            }
            ids.push(parent);
        }
    }

    pub(super) fn root_comments(
        &self,
        region: Option<Span>,
        layout: &RootLayout,
    ) -> Result<RootComments, Error> {
        self.charge(
            self.original.nodes[self.original.root.0].comments.len()
                + self.candidate.nodes[self.candidate.root.0].comments.len(),
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
        for rewrite in &regions {
            let id = rewrite.id;
            let rendered = if id == self.candidate.root {
                let comments = root_comments.as_ref().expect("root comment plan");
                splice::render_with_comments(
                    self.original.emission(self.candidate),
                    id,
                    &rewrite.boundary,
                    rewrite.context,
                    &comments.rendered,
                    syntax,
                )?
            } else {
                splice::render_with_comments(
                    self.original.emission(self.candidate),
                    id,
                    &rewrite.boundary,
                    rewrite.context,
                    &self.candidate.nodes[id.0].comments,
                    syntax,
                )?
            };
            patches.push((rendered.span, rendered.text));
        }
        if let Some(comments) = root_comments.as_mut() {
            patches.append(&mut comments.after_body);
            comments.finish_separation(self.original, &mut patches);
        }
        PatchSet::new(
            &self.original.text,
            patches,
            self.original.options.max_input_bytes,
        )
        .map(PatchSet::apply)
        .map_err(|message| self.error(message))
    }
}
