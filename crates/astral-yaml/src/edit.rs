//! Owned YAML syntax for source-preserving editing.

use crate::{
    CollectionStyle, Error, NodeId, ParseOptions, ScalarStyle, Span,
    comments::{CommentIndex, CommentPlacement},
    parse_with_options,
};

/// An owned source document. Node IDs refer only to this document's current view.
#[derive(Clone, Debug)]
pub struct EditableDocument {
    pub(crate) source: String,
    pub(crate) nodes: Vec<Node>,
    pub(crate) root: NodeId,
    pub(crate) options: ParseOptions,
}

/// An owned syntax node. Properties and comments belong to the occurrence,
/// including when the occurrence is an alias.
#[derive(Clone, Debug)]
pub struct Node {
    pub(crate) span: Span,
    pub(crate) kind: NodeKind,
    pub(crate) anchor: Option<String>,
    pub(crate) tag: Option<String>,
    pub(crate) comments: Vec<Comment>,
}

/// Syntax content. Mapping entries retain their order and arbitrary key nodes.
#[derive(Clone, Debug, PartialEq)]
pub enum NodeKind {
    Scalar {
        value: String,
        style: ScalarStyle,
    },
    Sequence {
        items: Vec<NodeId>,
        style: CollectionStyle,
    },
    Mapping {
        entries: Vec<(NodeId, NodeId)>,
        style: CollectionStyle,
    },
    Alias {
        name: String,
        target: NodeId,
    },
}

/// A comment with its direct owner retained by the containing node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Comment {
    pub(crate) text: String,
    pub(crate) placement: CommentPlacement,
    pub(crate) span: Span,
}

impl EditableDocument {
    /// Retains exactly one YAML document and its UTF-8 source.
    ///
    /// # Errors
    /// Returns a syntax, document-count, or resource-limit error.
    pub fn parse(source: &str) -> Result<Self, Error> {
        Self::parse_with_options(source, ParseOptions::default())
    }

    /// Retains one document using explicit parser limits.
    ///
    /// # Errors
    /// Returns a syntax, document-count, or resource-limit error.
    pub fn parse_with_options(source: &str, options: ParseOptions) -> Result<Self, Error> {
        let stream = parse_with_options(source, options)?;
        let [document] = stream.documents() else {
            return Err(Error {
                span: stream.span(),
                message: "expected exactly one document",
            });
        };
        let mut nodes: Vec<_> = stream
            .nodes()
            .iter()
            .map(|node| {
                let kind = match &node.kind {
                    crate::NodeKind::Scalar(scalar) => NodeKind::Scalar {
                        value: scalar.value.to_string(),
                        style: scalar.style,
                    },
                    crate::NodeKind::Sequence { items, style } => NodeKind::Sequence {
                        items: items.clone(),
                        style: *style,
                    },
                    crate::NodeKind::Mapping { entries, style } => NodeKind::Mapping {
                        entries: entries
                            .iter()
                            .map(|entry| (entry.key, entry.value))
                            .collect(),
                        style: *style,
                    },
                    crate::NodeKind::Alias { name, target } => NodeKind::Alias {
                        name: (*name).to_owned(),
                        target: *target,
                    },
                };
                Node {
                    span: node.span,
                    kind,
                    anchor: node.anchor.as_ref().map(|p| p.value.to_string()),
                    tag: node.tag.as_ref().map(|p| p.value.to_string()),
                    comments: Vec::new(),
                }
            })
            .collect();
        for (owner, attached) in CommentIndex::new(&stream, 0)?.direct() {
            nodes[owner.0].comments.push(Comment {
                text: attached.comment.text.to_owned(),
                placement: attached.placement,
                span: attached.comment.span,
            });
        }
        Ok(Self {
            source: source.to_owned(),
            nodes,
            root: document.root,
            options,
        })
    }

    /// The current committed UTF-8 source.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// The root of this document's current syntax view.
    #[must_use]
    pub fn root(&self) -> NodeId {
        self.root
    }

    /// Looks up a node in the current syntax view.
    #[must_use]
    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.0)
    }

    /// Parser limits used when constructing this document.
    #[must_use]
    pub fn options(&self) -> ParseOptions {
        self.options
    }
}

impl Node {
    /// The node's syntax content.
    #[must_use]
    pub fn kind(&self) -> &NodeKind {
        &self.kind
    }

    /// The node's range in the current committed source.
    #[must_use]
    pub fn span(&self) -> Span {
        self.span
    }

    /// The anchor defined by this node, if any.
    #[must_use]
    pub fn anchor(&self) -> Option<&str> {
        self.anchor.as_deref()
    }

    /// The expanded YAML tag, if any.
    #[must_use]
    pub fn tag(&self) -> Option<&str> {
        self.tag.as_deref()
    }

    /// Comments directly owned by this occurrence, in source order.
    #[must_use]
    pub fn comments(&self) -> &[Comment] {
        &self.comments
    }
}

impl Comment {
    /// Comment text, excluding the `#` character and line break.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Placement relative to its directly owning node.
    #[must_use]
    pub fn placement(&self) -> CommentPlacement {
        self.placement
    }

    /// Comment range in the current committed source.
    #[must_use]
    pub fn span(&self) -> Span {
        self.span
    }
}
