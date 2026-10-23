//! Owned YAML syntax for source-preserving editing.

mod draft;
mod emit;
mod framing;
mod layout;
mod lines;
mod plan;
mod scope;
mod source;
mod splice;
mod transaction;
mod view;

pub use transaction::{
    Collection, Cursor, Editor, Entry, Handle, Mapping, MappingMut, PathSegment, Sequence,
    SequenceMut, ValueRef,
};

pub use crate::edit_path;

use std::{borrow::Cow, cell::Cell};

use crate::{
    CollectionStyle, Error, NodeId, ParseOptions, ScalarStyle, Span,
    comments::{CommentIndex, CommentPlacement},
    parser::parse_with_tag_budget,
};

/// An owned YAML document, including its source and comments.
///
/// Use [`Self::edit`] to change the document. Successful edits update its source,
/// node IDs, and spans together. Failed edits leave it unchanged. IDs and spans
/// belong to the document state that produced them.
///
/// Within each transaction, imported YAML and comment text share the
/// [`ParseOptions::max_input_bytes`] limit. The same limit applies separately to
/// the total length of expanded shorthand tags in the imported fragments.
#[derive(Clone, Debug)]
pub struct EditableDocument {
    source: source::Source,
}

/// An owned YAML node.
///
/// For an alias, comments and source ranges describe the `*name` expression.
/// Committed nodes use [`Span`] for `Location`. Nodes returned by [`Editor`] use
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
    /// An alias to the node that defines its anchor.
    /// The alias name comes from that node's anchor.
    Alias {
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
/// [`Editor::comment_position`] uses [`Handle`] so a position can be reused in
/// an edit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommentPosition<Child = NodeId> {
    /// Before the node's properties and content.
    Before,
    /// Between the node's properties and its content.
    Header,
    /// An inline comment after a flow collection's opening delimiter.
    Opening,
    /// A collection-owned comment before an item or mapping key.
    BeforeChild(Child),
    /// After the node's content.
    After,
}

impl<Child> CommentPosition<Child> {
    fn try_map<T>(
        self,
        map: impl FnOnce(Child) -> Result<T, Error>,
    ) -> Result<CommentPosition<T>, Error> {
        Ok(match self {
            Self::Before => CommentPosition::Before,
            Self::Header => CommentPosition::Header,
            Self::Opening => CommentPosition::Opening,
            Self::BeforeChild(child) => CommentPosition::BeforeChild(map(child)?),
            Self::After => CommentPosition::After,
        })
    }
}

impl CommentPosition {
    fn for_comment(
        nodes: &[Node],
        owner: NodeId,
        attached: &crate::comments::AttachedComment<'_>,
        content_start: usize,
    ) -> Self {
        let node = &nodes[owner.0];
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
                    let index = items.partition_point(|id| nodes[id.0].span.start() <= at);
                    items.get(index).copied()
                }
                NodeKind::Mapping { entries, .. } => {
                    let index =
                        entries.partition_point(|&(key, _)| nodes[key.0].span.start() <= at);
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

impl Span {
    /// Include empty spans at either boundary when checking source containment.
    fn contains(self, other: Self) -> bool {
        self.start <= other.start && other.end <= self.end
    }
}

/// Charge the same work budget while editing the draft and planning its source rewrites.
fn charge_work(work: &Cell<usize>, amount: usize, span: Span) -> Result<(), Error> {
    let remaining = work.get().checked_sub(amount).ok_or(Error {
        span,
        message: "editing resource limit exceeded",
    })?;
    work.set(remaining);
    Ok(())
}

impl EditableDocument {
    /// Parse `source` as an editable YAML document.
    ///
    /// CR and CRLF line endings are converted to LF. Spans and error locations refer
    /// to this normalized source.
    ///
    /// # Errors
    /// Returns an error if `source` is invalid YAML, does not contain exactly one
    /// document, or exceeds a resource limit.
    pub fn parse(source: &str) -> Result<Self, Error> {
        Self::parse_with_options(source, ParseOptions::default())
    }

    /// Like [`Self::parse`], but use the given parser limits.
    /// The input byte limit applies before line endings are normalized.
    ///
    /// # Errors
    /// Returns an error if the options are invalid, `source` is invalid YAML or does
    /// not contain exactly one document, or a resource limit is exceeded.
    pub fn parse_with_options(source: &str, options: ParseOptions) -> Result<Self, Error> {
        let mut remaining_tag_bytes = options.max_input_bytes;
        Self::parse_with_tag_budget(source, options, &mut remaining_tag_bytes)
    }

    fn parse_with_tag_budget(
        source: &str,
        options: ParseOptions,
        remaining_tag_bytes: &mut usize,
    ) -> Result<Self, Error> {
        options.check_input(source.len())?;
        let source = lf_source(source);
        let stream = parse_with_tag_budget(&source, options, remaining_tag_bytes)?;
        let [document] = stream.documents.as_slice() else {
            return Err(Error {
                span: stream.span(),
                message: "expected exactly one document",
            });
        };
        let contents = source::contents(&source, stream.nodes());
        let comments = CommentIndex::new(&stream, 0)?;
        let mut nodes: Vec<_> = stream.nodes.into_iter().map(Node::from_parsed).collect();
        for (owner, attached) in comments.direct() {
            let position =
                CommentPosition::for_comment(&nodes, owner, attached, contents[owner.0].start());
            nodes[owner.0].comments.push(Comment {
                text: attached.comment.text.to_owned(),
                placement: attached.placement,
                position,
                span: attached.comment.span,
            });
        }
        Ok(Self {
            source: source::Source {
                nodes,
                contents,
                root: document.root,
                options,
                byte_order_marks: stream.byte_order_marks,
                start_marker: document.start_marker,
                end_marker: document.end_marker,
                extra_end_markers: stream
                    .document_end_markers
                    .into_iter()
                    .filter(|marker| {
                        document
                            .end_marker
                            .is_some_and(|first| marker.start() > first.start())
                    })
                    .collect(),
                text: source.into_owned(),
            },
        })
    }

    /// Return a copy of the YAML source.
    #[must_use]
    pub fn to_yaml(&self) -> String {
        self.source.text.clone()
    }

    /// Consume the document and return its YAML source without copying it.
    #[must_use]
    pub fn into_yaml(self) -> String {
        self.source.text
    }

    /// Render the document with normalized whitespace, without modifying it.
    ///
    /// Comments and compatible styles are preserved. Source outside the root node's
    /// rewrite region is copied unchanged. Byte order marks within the region
    /// stay on prefix or suffix lines outside the document content.
    ///
    /// # Errors
    /// Returns an error if rendering exceeds a resource limit or changes scalar
    /// content, alias targets, or comment ownership, placement, or order.
    pub fn normalized_yaml(&self) -> Result<String, Error> {
        let region = splice::region(self, self.source.root, false);
        let rendered = splice::render(self, self.source.root, region, false)?;
        let output = splice::apply(&self.source, vec![(rendered.span, rendered.text)], region)?;
        emit::parse_rendered(self.into(), &output, self.source.options, region, &[])?;
        Ok(output)
    }

    /// Return the YAML source, with LF line endings.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source.text
    }

    /// Return the root node's ID.
    #[must_use]
    pub fn root(&self) -> NodeId {
        self.source.root
    }

    /// Return the node with the given ID, or `None` if the ID is out of bounds.
    #[must_use]
    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.source.nodes.get(id.0)
    }

    /// Return the parser limits used to load this document and check its edits.
    #[must_use]
    pub fn options(&self) -> ParseOptions {
        self.source.options
    }
}

/// Normalize physical line breaks without changing escaped scalar content.
fn lf_source(source: &str) -> Cow<'_, str> {
    let Some(first) = source.find('\r') else {
        return Cow::Borrowed(source);
    };
    let mut normalized = String::with_capacity(source.len());
    normalized.push_str(&source[..first]);
    for segment in source[first + 1..].split('\r') {
        normalized.push('\n');
        normalized.push_str(segment.strip_prefix('\n').unwrap_or(segment));
    }
    Cow::Owned(normalized)
}

impl Node {
    fn from_parsed(node: crate::Node<'_>) -> Self {
        let kind = match node.kind {
            crate::NodeKind::Scalar(scalar) => NodeKind::Scalar {
                value: scalar.value.into_owned(),
                style: scalar.style,
            },
            crate::NodeKind::Sequence { items, style } => NodeKind::Sequence { items, style },
            crate::NodeKind::Mapping { entries, style } => NodeKind::Mapping {
                entries: entries
                    .into_iter()
                    .map(|entry| (entry.key, entry.value))
                    .collect(),
                style,
            },
            crate::NodeKind::Alias { target, .. } => NodeKind::Alias { target },
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
    fn attachment(&self) -> (&str, CommentPlacement, CommentPosition) {
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
    /// Use [`Editor::comment_position`] to capture a position for reinsertion during
    /// an edit.
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
