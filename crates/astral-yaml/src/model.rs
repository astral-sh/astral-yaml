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
pub struct Scalar<'a> {
    /// Text after YAML quoting, escaping, and line folding have been applied.
    pub value: Cow<'a, str>,
    /// The presentation used to spell the value in the source.
    pub style: ScalarStyle,
    /// The scalar token, including its delimiters and excluding node properties.
    pub span: Span,
}

/// An anchor name or expanded tag, with its source range.
/// Percent escapes in tag URIs retain their original spelling.
#[derive(Clone, Debug, PartialEq)]
pub struct Property<'a> {
    /// The name after `&`, or the tag after expanding its handle.
    pub value: Cow<'a, str>,
    /// The original spelling, including the leading `&` or `!`.
    pub span: Span,
}

/// A mapping entry. YAML keys can be collections.
#[derive(Clone, Debug, PartialEq)]
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
pub enum NodeKind<'a> {
    /// A scalar's text and presentation before schema resolution.
    Scalar(Scalar<'a>),
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
        name: &'a str,
        /// The anchored node, which can be an ancestor of the alias.
        target: NodeId,
    },
}

/// A syntax node, including any anchor and tag properties.
#[derive(Clone, Debug, PartialEq)]
pub struct Node<'a> {
    /// The source range covering both the node's properties and its content.
    pub span: Span,
    /// The anchor introduced on this node, if present.
    pub anchor: Option<Property<'a>>,
    /// The explicit tag on this node, if present.
    pub tag: Option<Property<'a>>,
    /// Scalar or collection contents, or a reference to an anchored node.
    pub kind: NodeKind<'a>,
}

/// A comment's text and source range.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Comment<'a> {
    /// Includes `#` and excludes the line break.
    pub span: Span,
    /// Excludes `#` and the line break. Preserves whitespace after `#`.
    pub text: &'a str,
}

/// A directive line, excluding its trailing comment and line break.
#[derive(Clone, Debug, PartialEq)]
pub struct Directive<'a> {
    /// The directive's source range, starting at `%`.
    pub span: Span,
    /// The directive name without its leading `%`.
    pub name: &'a str,
    /// The arguments with their internal whitespace and without any trailing comment.
    pub arguments: &'a str,
}

/// One document in a YAML stream.
#[derive(Clone, Debug, PartialEq)]
pub struct Document<'a> {
    /// Begins at the first directive, start marker, or root. An explicit end marker is included.
    pub span: Span,
    /// The root's arena ID. An empty document has an empty scalar as its root.
    pub root: NodeId,
    /// The three bytes of an explicit `---`, if present.
    pub start_marker: Option<Span>,
    /// The three bytes of the `...` that ends this document, if present.
    pub end_marker: Option<Span>,
    /// Directives preceding this document's start marker, in source order.
    pub directives: Vec<Directive<'a>>,
}

/// A parsed stream with flat ownership of all nodes, including cyclic aliases.
#[derive(Clone, Debug, PartialEq)]
pub struct Stream<'a> {
    pub(crate) source: &'a str,
    pub(crate) nodes: Vec<Node<'a>>,
    pub(crate) documents: Vec<Document<'a>>,
    pub(crate) comments: Vec<Comment<'a>>,
    pub(crate) byte_order_marks: Vec<Span>,
    pub(crate) document_end_markers: Vec<Span>,
}

impl<'a> Stream<'a> {
    /// The complete UTF-8 input against which parser spans are measured.
    #[must_use]
    pub fn source(&self) -> &'a str {
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
    pub fn documents(&self) -> &[Document<'a>] {
        &self.documents
    }

    /// Comments in source order, including those outside any document.
    #[must_use]
    pub fn comments(&self) -> &[Comment<'a>] {
        &self.comments
    }

    /// All nodes across all documents. Use [`Self::node`] to follow a node ID.
    #[must_use]
    pub fn nodes(&self) -> &[Node<'a>] {
        &self.nodes
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
    pub fn node(&self, id: NodeId) -> Option<&Node<'a>> {
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
