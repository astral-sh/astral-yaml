use astral_yaml::comments::{AttachedComment, CommentPlacement};
use astral_yaml::{CollectionStyle, NodeId, ScalarStyle, Span};

/// An owned YAML node.
///
/// For an alias, comments and source ranges describe the `*name` expression.
/// Committed nodes use [`Span`] for `Location`. Nodes returned during an edit use
/// `Option<Span>` because inserted nodes have no original source range.
#[derive(Clone, Debug)]
pub struct Node<Location = Span> {
    pub(crate) span: Location,
    pub(crate) kind: NodeKind,
    pub(crate) anchor: Option<String>,
    pub(crate) tag: Option<String>,
    pub(crate) comments: Vec<Comment<Location>>,
}

/// The contents of a [`Node`]. Mapping entries are ordered, and their keys can
/// be any kind of YAML node.
#[derive(Clone, Debug, PartialEq)]
pub enum NodeKind {
    /// Decoded text with its YAML presentation.
    Scalar {
        /// The text after YAML escapes and line folding have been applied.
        value: String,
        /// The scalar style retained by the editor.
        style: ScalarStyle,
    },
    /// A sequence of child nodes.
    Sequence {
        /// IDs in the current sequence order.
        items: Vec<NodeId>,
        /// The block or flow style of the sequence.
        style: CollectionStyle,
    },
    /// A mapping whose keys can themselves be collections.
    Mapping {
        /// Pairs of key and value IDs in the current entry order.
        entries: Vec<(NodeId, NodeId)>,
        /// The block or flow style of the mapping.
        style: CollectionStyle,
    },
    /// An alias to the node that defines its anchor.
    /// The alias name comes from that node's anchor.
    Alias {
        /// The anchored node, which can be an ancestor of the alias.
        target: NodeId,
    },
}

/// A comment attached to a [`Node`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Comment<Location = Span> {
    pub(crate) text: String,
    pub(crate) placement: CommentPlacement,
    pub(crate) position: CommentPosition,
    pub(crate) span: Location,
}

/// Where a comment appears within its node's syntax.
///
/// This is separate from [`CommentPlacement`]. A comment between a node's
/// properties and content can still have trailing placement. Rendering may move
/// these headers, but detached comments stay before their named child.
///
/// [`Comment::position`] identifies children by [`NodeId`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommentPosition {
    /// Before the node's properties and content.
    Before,
    /// Between the node's properties and its content.
    Header,
    /// An inline comment after a flow collection's opening delimiter.
    Opening,
    /// A collection-owned comment before an item or mapping key.
    BeforeChild(NodeId),
    /// After the node's content.
    After,
}

impl CommentPosition {
    pub(super) fn for_comment(
        nodes: &[Node],
        owner: NodeId,
        attached: &AttachedComment<'_>,
        content_start: usize,
    ) -> Self {
        let node = &nodes[owner.index()];
        let at = attached.comment.span.start();
        if at < node.span.start() {
            CommentPosition::Before
        } else if at < content_start {
            CommentPosition::Header
        } else if attached.placement == CommentPlacement::Inline
            && at < node.span.end()
            && matches!(
                node.kind,
                NodeKind::Sequence {
                    style: CollectionStyle::Flow,
                    ..
                } | NodeKind::Mapping {
                    style: CollectionStyle::Flow,
                    ..
                }
            )
        {
            CommentPosition::Opening
        } else if attached.placement == CommentPlacement::Detached {
            let following = match &node.kind {
                NodeKind::Sequence { items, .. } => {
                    let index = items.partition_point(|id| nodes[id.index()].span.start() <= at);
                    items.get(index).copied()
                }
                NodeKind::Mapping { entries, .. } => {
                    let index =
                        entries.partition_point(|&(key, _)| nodes[key.index()].span.start() <= at);
                    entries.get(index).map(|&(key, _)| key)
                }
                _ => None,
            };
            following.map_or(CommentPosition::After, CommentPosition::BeforeChild)
        } else {
            CommentPosition::After
        }
    }
}

impl Node {
    pub(super) fn from_parsed(node: astral_yaml::Node<'_>) -> Self {
        let kind = match node.kind {
            astral_yaml::NodeKind::Scalar(scalar) => NodeKind::Scalar {
                value: scalar.value.into_owned(),
                style: scalar.style,
            },
            astral_yaml::NodeKind::Sequence { items, style } => NodeKind::Sequence { items, style },
            astral_yaml::NodeKind::Mapping { entries, style } => NodeKind::Mapping {
                entries: entries
                    .into_iter()
                    .map(|entry| (entry.key, entry.value))
                    .collect(),
                style,
            },
            astral_yaml::NodeKind::Alias { target, .. } => NodeKind::Alias { target },
        };
        Self {
            span: node.span,
            kind,
            anchor: node.anchor.map(|property| property.value.into_owned()),
            tag: node.tag.map(|property| property.value.into_owned()),
            comments: Vec::new(),
        }
    }
}

impl<Location> Node<Location> {
    /// Return the node's contents.
    #[must_use]
    pub fn kind(&self) -> &NodeKind {
        &self.kind
    }

    /// Return the node's source range.
    ///
    /// During an edit, this is the original range, even after replacement. Newly
    /// inserted nodes have no source range. The committed document has fresh ranges.
    #[must_use]
    pub fn span(&self) -> Location
    where
        Location: Copy,
    {
        self.span
    }

    /// Return the node's anchor name, if any.
    #[must_use]
    pub fn anchor(&self) -> Option<&str> {
        self.anchor.as_deref()
    }

    /// Return the node's expanded YAML tag, if any.
    #[must_use]
    pub fn tag(&self) -> Option<&str> {
        self.tag.as_deref()
    }

    /// Return this node's own comments in their current order, excluding its descendants'
    /// comments.
    #[must_use]
    pub fn comments(&self) -> &[Comment<Location>] {
        &self.comments
    }
}

impl<Location> Comment<Location> {
    /// The text, placement, and position compared across edits, independent of source spans.
    pub(super) fn attachment(&self) -> (&str, CommentPlacement, CommentPosition) {
        (&self.text, self.placement, self.position)
    }

    /// Return the comment text, excluding `#` and the line break.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Return the comment's placement relative to the node that owns it.
    #[must_use]
    pub fn placement(&self) -> CommentPlacement {
        self.placement
    }

    /// Return the comment's position within its node.
    #[must_use]
    pub fn position(&self) -> CommentPosition {
        self.position
    }

    /// Return the comment's source range.
    /// During an edit, retained comments keep their original ranges and new comments
    /// have no range. The committed document has fresh ranges.
    #[must_use]
    pub fn span(&self) -> Location
    where
        Location: Copy,
    {
        self.span
    }
}
