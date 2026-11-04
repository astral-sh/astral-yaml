use std::borrow::Cow;

/// A half-open byte range.
///
/// Parser spans use offsets in the decoded UTF-8 input. [`crate::Source::original_span`]
/// converts them to offsets in the original encoded bytes.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Span {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl Span {
    /// Creates an ordered range. Source boundaries are checked by [`Self::slice`].
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Option<Self> {
        if start <= end {
            Some(Self { start, end })
        } else {
            None
        }
    }

    /// The inclusive start offset in bytes.
    #[must_use]
    pub const fn start(self) -> usize {
        self.start
    }

    /// The exclusive end offset in bytes.
    #[must_use]
    pub const fn end(self) -> usize {
        self.end
    }

    /// The number of bytes covered by this span.
    #[must_use]
    pub const fn len(self) -> usize {
        self.end - self.start
    }

    /// Returns whether the start and end offsets are equal.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }

    /// Returns the covered text if both offsets are in bounds and on UTF-8 boundaries.
    #[must_use]
    pub fn slice(self, source: &str) -> Option<&str> {
        source.get(self.start..self.end)
    }

    /// Slices encoded input using a span returned by [`crate::Source::original_span`].
    #[must_use]
    pub fn slice_bytes(self, source: &[u8]) -> Option<&[u8]> {
        source.get(self.start..self.end)
    }
}

/// An index into one parsed stream's node arena.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NodeId(pub(crate) usize);

impl NodeId {
    /// Construct an index into a node arena.
    #[must_use]
    pub const fn from_index(index: usize) -> Self {
        Self(index)
    }

    /// Return the node's zero-based arena index.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

/// The presentation style of a scalar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScalarStyle {
    /// Unquoted text whose implicit type can depend on its spelling.
    Plain,
    /// Single-quoted text, where two adjacent quotes encode one quote.
    SingleQuoted,
    /// Double-quoted text, which supports backslash escapes.
    DoubleQuoted,
    /// A block introduced by `|` that preserves interior line breaks.
    Literal,
    /// A block introduced by `>` that folds ordinary adjacent lines together.
    Folded,
}

/// The presentation style of a collection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CollectionStyle {
    /// Entries are separated by indentation and line breaks.
    Block,
    /// Entries appear inside `[]` or `{}`, separated by commas.
    Flow,
}

/// A decoded scalar and its source range.
#[derive(Clone, Debug, PartialEq)]
pub struct Scalar<'source> {
    /// Text after YAML quoting, escaping, and line folding have been applied.
    pub value: Cow<'source, str>,
    /// The presentation used to spell the value in the source.
    pub style: ScalarStyle,
    /// The scalar token, including its delimiters and excluding node properties.
    /// Omitted content has a zero-width span before its trailing separation and comments.
    /// For example, in `key: # comment` the span is immediately after the colon.
    pub span: Span,
}

impl Scalar<'_> {
    pub(crate) fn empty(offset: usize) -> Self {
        Self {
            value: Cow::Borrowed(""),
            style: ScalarStyle::Plain,
            span: Span {
                start: offset,
                end: offset,
            },
        }
    }
}

/// An anchor name or expanded tag, with its source range.
/// Percent escapes in tag URIs retain their original spelling.
#[derive(Clone, Debug, PartialEq)]
pub struct Property<'source> {
    /// The name after `&`, or the tag after expanding its handle.
    pub value: Cow<'source, str>,
    /// The original spelling, including the leading `&` or `!`.
    pub span: Span,
}

/// A mapping entry. YAML keys can be collections.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Entry {
    /// The key node in the stream's arena.
    pub key: NodeId,
    /// The value node, including an empty scalar if the value was omitted.
    pub value: NodeId,
    /// The source range covering the key and value.
    pub span: Span,
}

/// Node content. Aliases retain identity without expanding their targets.
#[derive(Clone, Debug, PartialEq)]
pub enum NodeKind<'source> {
    /// A scalar's text and presentation before schema resolution.
    Scalar(Scalar<'source>),
    /// An ordered collection of nodes.
    Sequence {
        /// Item IDs in source order.
        items: Vec<NodeId>,
        /// The enclosing sequence's presentation.
        style: CollectionStyle,
    },
    /// Key-value pairs in source order.
    Mapping {
        /// Entries as parsed, including repeated keys.
        entries: Vec<Entry>,
        /// The enclosing mapping's presentation.
        style: CollectionStyle,
    },
    /// A reference to the most recent preceding anchor with this name.
    Alias {
        /// The name after `*`.
        name: &'source str,
        /// The anchored node, which can be an ancestor of the alias.
        target: NodeId,
    },
}

/// A syntax node, including any anchor and tag properties.
#[derive(Clone, Debug, PartialEq)]
pub struct Node<'source> {
    /// The source range covering both the node's properties and its content.
    pub span: Span,
    /// The anchor introduced on this node, if present.
    pub anchor: Option<Property<'source>>,
    /// The explicit tag on this node, if present.
    pub tag: Option<Property<'source>>,
    /// Scalar or collection contents, or a reference to an anchored node.
    pub kind: NodeKind<'source>,
}

/// A comment's text and source range.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Comment<'source> {
    /// Includes `#` and excludes the line break.
    pub span: Span,
    /// Excludes `#` and the line break. Preserves whitespace after `#`.
    pub text: &'source str,
}

/// A directive line, excluding its trailing comment and line break.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Directive<'source> {
    /// The source range from `%` through the name or last argument.
    pub span: Span,
    /// The directive name without its leading `%`.
    pub name: &'source str,
    /// The arguments with their internal whitespace and without any trailing comment.
    pub arguments: &'source str,
}

/// One document in a YAML stream.
#[derive(Clone, Debug, PartialEq)]
pub struct Document<'source> {
    /// Begins at the first directive, start marker, or root. An explicit end marker is included.
    pub span: Span,
    /// The root's arena ID. An empty document has an empty scalar as its root.
    pub root: NodeId,
    /// The three bytes of an explicit `---`, if present.
    pub start_marker: Option<Span>,
    /// The three bytes of the `...` that ends this document, if present.
    pub end_marker: Option<Span>,
    /// Directives preceding this document's start marker, in source order.
    pub directives: Vec<Directive<'source>>,
}

/// A parsed stream with flat ownership of all nodes, including cyclic aliases.
#[derive(Clone, Debug, PartialEq)]
pub struct Stream<'source> {
    pub(crate) source: &'source str,
    pub(crate) nodes: Vec<Node<'source>>,
    pub(crate) documents: Vec<Document<'source>>,
    pub(crate) comments: Vec<Comment<'source>>,
    pub(crate) byte_order_marks: Vec<Span>,
    pub(crate) document_end_markers: Vec<Span>,
}

impl<'source> Stream<'source> {
    /// The complete UTF-8 input against which parser spans are measured.
    #[must_use]
    pub fn source(&self) -> &'source str {
        self.source
    }

    /// The range of the entire input, including leading and trailing trivia.
    #[must_use]
    pub fn span(&self) -> Span {
        Span {
            start: 0,
            end: self.source.len(),
        }
    }

    /// Documents in source order. An empty or comment-only stream has none.
    #[must_use]
    pub fn documents(&self) -> &[Document<'source>] {
        &self.documents
    }

    /// Comments in source order, including those outside any document.
    #[must_use]
    pub fn comments(&self) -> &[Comment<'source>] {
        &self.comments
    }

    /// All nodes across all documents. Use [`Self::node`] to follow a node ID.
    #[must_use]
    pub fn nodes(&self) -> &[Node<'source>] {
        &self.nodes
    }

    /// Consume the stream and return its node arena in ID order.
    #[must_use]
    pub fn into_nodes(self) -> Vec<Node<'source>> {
        self.nodes
    }

    /// Byte order marks at document boundaries, excluding any inside scalar content.
    #[must_use]
    pub fn byte_order_marks(&self) -> &[Span] {
        &self.byte_order_marks
    }

    /// Every explicit document end marker, including standalone stream suffixes.
    #[must_use]
    pub fn document_end_markers(&self) -> &[Span] {
        &self.document_end_markers
    }

    /// Returns a node when the ID lies within this stream's arena.
    #[must_use]
    pub fn node(&self, id: NodeId) -> Option<&Node<'source>> {
        self.nodes.get(id.0)
    }

    /// Resolves a source offset to a one-based line and Unicode column.
    #[must_use]
    pub fn location(&self, offset: usize) -> Option<(usize, usize)> {
        let prefix = self.source.get(..offset)?;
        let mut line = 1;
        let mut column = 1;
        let mut cr = false;

        for c in prefix.chars() {
            match c {
                '\n' if cr => {}
                '\r' | '\n' => {
                    line += 1;
                    column = 1;
                }
                _ => column += 1,
            }

            cr = c == '\r';
        }

        Some((line, column))
    }
}
