//! Source planning reads a frozen draft and a separate source index. It emits
//! replacement text; it cannot publish a document or alter the semantic draft.

use super::{
    NodeKind,
    draft::Draft,
    emit,
    framing::{RootComments, RootLayout},
    layout::Layout,
    source::Source,
};
use crate::{CollectionStyle, Error, NodeId, ScalarStyle, Span, comments::CommentPlacement};
use std::{cell::Cell, collections::HashSet};

pub(super) struct Plan<'a> {
    pub(super) original: &'a Source,
    pub(super) candidate: &'a Draft,
    pub(super) parents: &'a [Option<NodeId>],
    pub(super) rewrites: &'a HashSet<NodeId>,
    pub(super) root_comments_changed: bool,
    pub(super) work: &'a Cell<usize>,
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
            if id.0 < self.original.nodes.len() {
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

    pub(super) fn rewrite_regions(
        &self,
        layout: &RootLayout,
        syntax: &Layout,
    ) -> Result<Vec<(NodeId, Span)>, Error> {
        let mut ids: Vec<_> = self.rewrites.iter().copied().collect();
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
            let mut regions: Vec<_> = ids
                .iter()
                .map(|&id| {
                    let mut region = emit::region(self.original, id, false);
                    if id == self.candidate.root
                        && emit::has_inline_header(&self.candidate.nodes[id.0], false)
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
            regions.sort_by_key(|(_, span)| span.start());
            let Some(pair) = regions
                .windows(2)
                .find(|pair| pair[0].1.end() > pair[1].1.start())
            else {
                return Ok(regions);
            };
            let (left, right) = (pair[0].0, pair[1].0);
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
        let root_region = regions.iter().find(|&&(id, _)| id == self.candidate.root);
        let mut root_comments = if root_region.is_some() || self.root_comments_changed {
            let region = root_region.map(|&(_, region)| region);
            Some(self.root_comments(region, &layout)?)
        } else {
            None
        };
        let mut patches = root_comments
            .as_mut()
            .map_or_else(Vec::new, |comments| std::mem::take(&mut comments.patches));
        for &(id, region) in &regions {
            let flow = self.flow_context(id);
            let indent = emit::indent(&self.original.text, region.start());
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
                emit::render_with_comments(
                    self.original.emission(self.candidate),
                    id,
                    region,
                    context,
                    &comments.rendered,
                    syntax,
                )?
            } else {
                emit::render_with_comments(
                    self.original.emission(self.candidate),
                    id,
                    region,
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
        // Prefix insertions at a replacement's start precede that replacement.
        // Suffix insertions at an empty root's position follow its syntax.
        // Stable sorting retains candidate order for insertions at the same point.
        patches.sort_by_key(|(span, _)| (span.start(), span.end()));
        let source = apply_patches(
            &self.original.text,
            patches,
            self.original.options.max_input_bytes,
        )
        .map_err(|message| self.error(message))?;
        Ok(source)
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

/// Apply ordered, disjoint byte patches. Validate each span's UTF-8 boundaries
/// before slicing, and include the retained suffix in the output limit.
pub(super) fn apply_patches(
    source: &str,
    patches: Vec<(Span, String)>,
    limit: usize,
) -> Result<String, &'static str> {
    let mut output = String::new();
    let mut position = 0;
    for (span, replacement) in patches {
        let prefix = source
            .get(position..span.start())
            .ok_or("overlapping or invalid source patches")?;
        source
            .get(span.start()..span.end())
            .ok_or("overlapping or invalid source patches")?;
        output
            .len()
            .checked_add(prefix.len())
            .and_then(|size| size.checked_add(replacement.len()))
            .filter(|&size| size <= limit)
            .ok_or("emitted YAML exceeds input byte limit")?;
        output.push_str(prefix);
        output.push_str(&replacement);
        position = span.end();
    }
    let suffix = &source[position..];
    output
        .len()
        .checked_add(suffix.len())
        .filter(|&size| size <= limit)
        .ok_or("emitted YAML exceeds input byte limit")?;
    output.push_str(suffix);
    Ok(output)
}
