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
#[cfg(test)]
#[path = "../tests/syntax/mod.rs"]
mod tests;
mod transaction;
mod view;

pub(crate) use transaction::{Editor, ValueRef};

use std::borrow::Cow;
use std::cell::Cell;

use self::splice::OuterComments;
use astral_yaml::comments::{CommentIndex, CommentPlacement};
use astral_yaml::{CollectionStyle, Error, NodeId, ParseOptions, ScalarStyle, Span};

/// The state of a selected collection.
#[derive(Debug)]
pub struct Collection<Kind> {
    pub(crate) len: usize,
    pub(crate) kind: std::marker::PhantomData<Kind>,
}

/// Mapping cursor state.
#[derive(Debug)]
pub enum Mapping {}

/// Sequence cursor state.
#[derive(Debug)]
pub enum Sequence {}

/// An owned YAML document, including its source and comments.
///
/// Inspect this document through [`Editable::syntax`](crate::Editable::syntax).
/// Node IDs and spans belong to the document state that produced them.
///
/// Serialized values and new comment text share the transaction’s
/// [`ParseOptions::max_input_bytes`] limit.
#[derive(Clone, Debug)]
pub struct DocumentSyntax {
    source: source::Source,
}

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
    fn for_comment(
        nodes: &[Node],
        owner: NodeId,
        attached: &astral_yaml::comments::AttachedComment<'_>,
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

fn span(start: usize, end: usize) -> Span {
    Span::new(start, end).expect("ordered source range")
}

fn contains(outer: Span, inner: Span) -> bool {
    outer.start() <= inner.start() && inner.end() <= outer.end()
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

fn single_document<'stream, 'source>(
    stream: &'stream astral_yaml::Stream<'source>,
) -> Result<&'stream astral_yaml::Document<'source>, Error> {
    match stream.documents() {
        [document] => Ok(document),
        _ => Err(Error {
            span: stream.span(),
            message: "expected exactly one document",
        }),
    }
}

impl DocumentSyntax {
    #[cfg(test)]
    pub(crate) fn parse(source: &str) -> Result<Self, Error> {
        Self::parse_with_options(source, ParseOptions::default())
    }

    /// Parse one document with LF line endings. Spans refer to the normalized source.
    pub(crate) fn parse_with_options(source: &str, options: ParseOptions) -> Result<Self, Error> {
        // Let the parser reject oversized input before allocating a normalized copy.
        let source = if source.len() > options.max_input_bytes {
            Cow::Borrowed(source)
        } else {
            lf_source(source)
        };
        let stream = astral_yaml::parse_with_options(&source, options)?;
        let document = single_document(&stream)?;
        let contents = source::contents(&source, stream.nodes());
        let comments = CommentIndex::new(&stream, 0)?;
        let root = document.root;
        let start_marker = document.start_marker;
        let end_marker = document.end_marker;
        let byte_order_marks = stream.byte_order_marks().to_vec();
        let extra_end_markers = stream
            .document_end_markers()
            .iter()
            .copied()
            .filter(|marker| end_marker.is_some_and(|first| marker.start() > first.start()))
            .collect();
        let mut nodes: Vec<_> = stream
            .into_nodes()
            .into_iter()
            .map(Node::from_parsed)
            .collect();
        for (owner, attached) in comments.direct() {
            let position = CommentPosition::for_comment(
                &nodes,
                owner,
                attached,
                contents[owner.index()].start(),
            );
            nodes[owner.index()].comments.push(Comment {
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
                root,
                options,
                byte_order_marks,
                start_marker,
                end_marker,
                extra_end_markers,
                text: source.into_owned(),
            },
        })
    }

    /// Return a copy of the YAML source.
    #[must_use]
    pub(crate) fn to_yaml(&self) -> String {
        self.source.text.clone()
    }

    /// Consume the document and return its YAML source without copying it.
    #[must_use]
    pub(crate) fn into_yaml(self) -> String {
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
        let region = splice::region(self, self.source.root, OuterComments::Exclude);
        let rendered = splice::render(self, self.source.root, region)?;
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
        self.source.nodes.get(id.index())
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
    fn from_parsed(node: astral_yaml::Node<'_>) -> Self {
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
