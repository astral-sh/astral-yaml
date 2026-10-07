//! Source planning reads a frozen draft and a separate source index. It emits
//! replacement text; it cannot publish a document or alter the semantic draft.

use super::{
    Comment, CommentPosition, Node, NodeKind,
    draft::Candidate,
    emit,
    framing::{RootComments, RootLayout},
    layout::Layout,
    source::Source,
    splice::{self, Boundary, PatchSet},
    view::Syntax,
};
use crate::{CollectionStyle, Error, NodeId, ScalarStyle, Span, comments::CommentPlacement};
use std::{cell::Cell, collections::HashSet};

pub(super) struct Plan<'a> {
    pub(super) original: &'a Source,
    pub(super) candidate: Syntax<'a, Option<Span>>,
    pub(super) parents: &'a [Option<NodeId>],
    pub(super) changes: Changes,
    pub(super) work: &'a Cell<usize>,
}

/// A selected subtree and the exact source reservation used to check overlap.
/// Rendering consumes this same boundary rather than deriving another range.
pub(super) struct Rewrite<'a> {
    pub(super) id: NodeId,
    pub(super) boundary: Boundary<'a>,
}

/// Differences between the original syntax and the final intended tree. This
/// has no mutation history: canceled edits cannot leave stale rewrite marks.
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
                    _ => false,
                };
            if !content && !comments && !resolved {
                continue;
            }
            if id == syntax.root && !content && !resolved && only_outer_comments(old, new) {
                changes.root_comments = true;
                continue;
            }
            let attachments = old.comments.len() != new.comments.len()
                || old
                    .comments
                    .iter()
                    .zip(&new.comments)
                    .any(|(old, new)| !same_attachment(old, new));
            let collection = matches!(
                (&old.kind, &new.kind),
                (NodeKind::Sequence { .. }, NodeKind::Sequence { .. })
                    | (NodeKind::Mapping { .. }, NodeKind::Mapping { .. })
            );
            // A collection owns its child list and delimiters. Token changes
            // can also remain local when their properties/comments have no
            // header. Other shape or attachment changes need the parent header.
            let local = !attachments && (collection || (inline_node(old) && inline_node(new)));
            changes
                .roots
                .insert(if local { id } else { parent.unwrap_or(id) });
        }
        changes
    }

    pub(super) fn is_empty(&self) -> bool {
        self.roots.is_empty() && !self.root_comments
    }
}

fn same_attachment(old: &Comment, new: &Comment<Option<Span>>) -> bool {
    new.span == Some(old.span) && old.text == new.text && old.placement == new.placement
}

fn same_comments(old: &[Comment], new: &[Comment<Option<Span>>]) -> bool {
    old.len() == new.len()
        && old
            .iter()
            .zip(new)
            .all(|(old, new)| same_attachment(old, new) && old.position == new.position)
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

fn inline_node<Location>(node: &Node<Location>) -> bool {
    node.comments
        .iter()
        .all(|comment| comment.position != CommentPosition::Header)
        && matches!(
            node.kind,
            NodeKind::Scalar {
                style: ScalarStyle::Plain | ScalarStyle::SingleQuoted | ScalarStyle::DoubleQuoted,
                ..
            } | NodeKind::Alias { .. }
        )
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

    /// Select the syntax context from the final candidate, after all mutations.
    fn rewrite_root(&self, mut id: NodeId, syntax: &Layout) -> NodeId {
        // Changes below a key can make an enclosing implicit key multiline or
        // too long. Include its mapping so emission can use explicit notation.
        let edited = id;
        let mut ancestor = id;
        while let Some(parent) = self.parents[ancestor.0] {
            if ancestor != edited
                && matches!(&self.candidate.nodes[parent.0].kind,
                    NodeKind::Mapping { entries, .. }
                        if entries.iter().any(|&(key, _)| key == ancestor))
            {
                id = parent;
            }
            ancestor = parent;
        }

        while let Some(parent) = self.parents[id.0] {
            // Changes only selects retained occurrences. Their ownership path
            // also belongs to the original source; imported nodes are covered
            // by the retained collection whose child list changed.
            let multiline = syntax.multiline(id, &self.candidate.nodes[id.0].kind);
            let start = self.original.nodes[id.0].span.start();
            let prefix = self.original.text[..start]
                .rsplit('\n')
                .next()
                .unwrap_or("");
            if (!multiline || prefix.bytes().all(|byte| byte == b' '))
                && !self.needs_mapping_rewrite(id, parent)
            {
                break;
            }
            id = parent;
        }
        id
    }

    fn needs_mapping_rewrite(&self, id: NodeId, parent: NodeId) -> bool {
        let NodeKind::Mapping { entries, style } = &self.original.nodes[parent.0].kind else {
            return false;
        };
        let source = &self.original.text;
        let span = self.original.nodes[id.0].span;
        for &(key, value) in entries {
            if key == id {
                // Key notation and separation belong to the mapping, outside
                // the key's span. Reuse the emitter's implicit-key check.
                let node = &self.candidate.nodes[id.0];
                let separator = crate::comments::skip_trivia(source, span.end());
                let plain = matches!(
                    node.kind,
                    NodeKind::Scalar {
                        style: ScalarStyle::Plain,
                        ..
                    }
                );
                return !emit::simple_key(node, *style == CollectionStyle::Flow)
                    || (plain
                        && source[separator..].strip_prefix(':').is_some_and(|suffix| {
                            suffix.starts_with(|c: char| {
                                !c.is_ascii_whitespace() && !matches!(c, ',' | ']' | '}')
                            })
                        }));
            }
            if value != id {
                continue;
            }
            // A block sequence may be indentless. Render its mapping when it
            // becomes flow syntax so the value can follow ':' on the same line.
            if matches!(
                self.original.nodes[id.0].kind,
                NodeKind::Sequence {
                    style: CollectionStyle::Block,
                    ..
                }
            ) && matches!(&self.candidate.nodes[id.0].kind,
                    NodeKind::Sequence { items, style }
                        if items.is_empty() || *style == CollectionStyle::Flow)
            {
                return true;
            }
            if span.is_empty() {
                // A value omitted without ':' has no source range in which
                // a scalar replacement can supply its value indicator.
                let separator =
                    crate::comments::skip_trivia(source, self.original.nodes[key.0].span.end());
                return source.as_bytes().get(separator) != Some(&b':');
            }
        }
        false
    }

    fn body_region(&self, id: NodeId, layout: &RootLayout) -> Span {
        let mut region = splice::region(self.original, id, false);
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
            let roots: HashSet<_> = ids
                .into_iter()
                .map(|id| self.rewrite_root(id, syntax))
                .collect();
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
                    let region = self.body_region(id, layout);
                    (id, region)
                })
                .collect();

            let mut expanded = false;
            for &(id, region) in &regions {
                let Some(parent) = self.parents[id.0] else {
                    continue;
                };
                let original = &self.original.nodes[id.0];
                let collection = matches!(
                    self.candidate.nodes[id.0].kind,
                    NodeKind::Sequence { .. } | NodeKind::Mapping { .. }
                );
                let outer_comments = collection
                    && original.comments.iter().any(|comment| {
                        comment.span.start() < original.span.start()
                            || comment.span.end() > original.span.end()
                    });
                // A descendant prefix can move the region before the node's
                // indentation or field header. A collection's outer comments
                // can also change attachment when its delimiters move. Render
                // the parent to supply both the syntax and comment context.
                // The document root keeps its separate framing-comment plan.
                if region.start() < original.span.start() || outer_comments {
                    ids.push(parent);
                    expanded = true;
                }
            }
            if expanded {
                continue;
            }
            let mut rewrites: Vec<_> = regions
                .into_iter()
                .map(|(id, span)| Rewrite {
                    id,
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
            let region = rewrite.boundary.body();
            let flow = self.flow_context(id);
            let indent = splice::indent(&self.original.text, region.start());
            let context = if self.parents[id.0].is_some_and(|parent| {
                matches!(
                    &self.candidate.nodes[parent.0].kind,
                    NodeKind::Mapping { entries, .. } if entries.iter().any(|&(key, _)| key == id)
                )
            }) {
                emit::Context::key(indent, flow)
            } else {
                emit::Context::value(
                    indent,
                    flow,
                    if id == self.candidate.root { 0 } else { indent },
                )
            };
            let rendered = if id == self.candidate.root {
                let comments = root_comments.as_ref().expect("root comment plan");
                splice::render_with_comments(
                    self.original.emission(self.candidate),
                    id,
                    &rewrite.boundary,
                    context,
                    &comments.rendered,
                    syntax,
                )?
            } else {
                splice::render_with_comments(
                    self.original.emission(self.candidate),
                    id,
                    &rewrite.boundary,
                    context,
                    &self.candidate.nodes[id.0].comments,
                    syntax,
                )?
            };
            patches.push((rendered.span, rendered.text));
        }
        if let Some(comments) = root_comments.as_mut() {
            patches.append(&mut comments.after_body);
            comments.finish_suffix(self.original, &mut patches);
        }
        PatchSet::new(
            &self.original.text,
            patches,
            self.original.options.max_input_bytes,
        )
        .map(PatchSet::apply)
        .map_err(|message| self.error(message))
    }

    fn flow_context(&self, mut id: NodeId) -> bool {
        while let Some(parent) = self.parents[id.0] {
            match &self.candidate.nodes[parent.0].kind {
                NodeKind::Sequence {
                    style: CollectionStyle::Flow,
                    ..
                }
                | NodeKind::Mapping {
                    style: CollectionStyle::Flow,
                    ..
                } => return true,
                _ => {}
            }
            id = parent;
        }
        false
    }
}
