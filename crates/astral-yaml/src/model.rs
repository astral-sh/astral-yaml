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

    #[must_use]
    pub const fn start(self) -> usize {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> usize {
        self.end
    }

    #[must_use]
    pub const fn len(self) -> usize {
        self.end - self.start
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }

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
    Plain,
    SingleQuoted,
    DoubleQuoted,
    Literal,
    Folded,
}

/// The presentation style of a collection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CollectionStyle {
    Block,
    Flow,
}

/// A decoded scalar and its source range.
#[derive(Clone, Debug, PartialEq)]
pub struct Scalar<'a> {
    pub value: Cow<'a, str>,
    pub style: ScalarStyle,
    pub span: Span,
}

/// An anchor name or expanded tag, with its source range.
/// Percent escapes in tag URIs retain their original spelling.
#[derive(Clone, Debug, PartialEq)]
pub struct Property<'a> {
    pub value: Cow<'a, str>,
    pub span: Span,
}

/// A mapping entry. YAML keys can be collections.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub key: NodeId,
    pub value: NodeId,
    pub span: Span,
}

/// Node content. Aliases retain identity without expanding their targets.
#[derive(Clone, Debug, PartialEq)]
pub enum NodeKind<'a> {
    Scalar(Scalar<'a>),
    Sequence {
        items: Vec<NodeId>,
        style: CollectionStyle,
    },
    Mapping {
        entries: Vec<Entry>,
        style: CollectionStyle,
    },
    Alias {
        name: &'a str,
        target: NodeId,
    },
}

/// A syntax node, including any anchor and tag properties.
#[derive(Clone, Debug, PartialEq)]
pub struct Node<'a> {
    pub span: Span,
    pub anchor: Option<Property<'a>>,
    pub tag: Option<Property<'a>>,
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
    pub span: Span,
    pub name: &'a str,
    pub arguments: &'a str,
}

/// One document in a YAML stream.
#[derive(Clone, Debug, PartialEq)]
pub struct Document<'a> {
    pub span: Span,
    pub root: NodeId,
    pub start_marker: Option<Span>,
    pub end_marker: Option<Span>,
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
    #[must_use]
    pub fn source(&self) -> &'a str {
        self.source
    }

    #[must_use]
    pub fn span(&self) -> Span {
        Span {
            start: 0,
            end: self.source.len(),
        }
    }

    #[must_use]
    pub fn documents(&self) -> &[Document<'a>] {
        &self.documents
    }

    #[must_use]
    pub fn comments(&self) -> &[Comment<'a>] {
        &self.comments
    }

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
