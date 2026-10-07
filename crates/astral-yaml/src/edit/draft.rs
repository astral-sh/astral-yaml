//! Mutable syntax has no source text. Its locations describe provenance in the
//! original document; they are not spans of the draft's eventual output.

use super::{EditableDocument, Node, NodeKind, emit, view::Syntax};
use crate::{Error, NodeId, ParseOptions, Span};
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub(super) struct Draft {
    pub(super) nodes: Vec<Node>,
    pub(super) root: NodeId,
    pub(super) requested_styles: HashSet<NodeId>,
}

impl From<&EditableDocument> for Draft {
    fn from(document: &EditableDocument) -> Self {
        Self {
            nodes: document.nodes.clone(),
            root: document.root,
            requested_styles: HashSet::new(),
        }
    }
}

impl<'a> From<&'a Draft> for Syntax<'a, Span> {
    fn from(draft: &'a Draft) -> Self {
        Self {
            nodes: &draft.nodes,
            root: draft.root,
        }
    }
}

impl Draft {
    /// Only reparsed output that agrees with the unchanged draft can commit.
    pub(super) fn commit(
        &self,
        output: &str,
        options: ParseOptions,
    ) -> Result<EditableDocument, Error> {
        let error = |message| Error {
            span: self.nodes[self.root.0].span,
            message,
        };
        let parsed = EditableDocument::parse_with_options(output, options)?;
        let correspondence = emit::correspondence(self, &parsed)
            .ok_or_else(|| error("emission changed YAML content or alias identity"))?;
        for &id in &self.requested_styles {
            // Removing a subtree also removes its style requests.
            let target = correspondence[id.0].expect("styled node is retained");
            if !same_style(&self.nodes[id.0].kind, &parsed.nodes[target.0].kind) {
                return Err(error(
                    "requested style is not representable in this context",
                ));
            }
        }
        if !emit::comments_preserved(self, &parsed, &correspondence) {
            return Err(error("emission changed comment ownership or placement"));
        }
        Ok(parsed)
    }
}

pub(super) fn same_style(left: &NodeKind, right: &NodeKind) -> bool {
    match (left, right) {
        (NodeKind::Scalar { style: a, .. }, NodeKind::Scalar { style: b, .. }) => a == b,
        (NodeKind::Sequence { items, style: a }, NodeKind::Sequence { style: b, .. }) => {
            items.is_empty() || a == b
        }
        (NodeKind::Mapping { entries, style: a }, NodeKind::Mapping { style: b, .. }) => {
            entries.is_empty() || a == b
        }
        _ => false,
    }
}
