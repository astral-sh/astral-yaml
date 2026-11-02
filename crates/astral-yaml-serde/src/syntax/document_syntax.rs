use super::source::{self, Source};
use super::splice::{self, OuterComments};
use super::{Comment, CommentPosition, Node, emit};
use astral_yaml::comments::CommentIndex;
use astral_yaml::{Error, NodeId, ParseOptions};
use std::borrow::Cow;

/// An owned YAML document, including its source and comments.
///
/// Inspect this document through [`Editable::syntax`](crate::Editable::syntax).
/// Node IDs and spans belong to the document state that produced them.
///
/// Serialized values and new comment text share the transaction’s
/// [`ParseOptions::max_input_bytes`] limit.
#[derive(Clone, Debug)]
pub struct DocumentSyntax {
    pub(super) source: Source,
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
            normalized_source(source)
        };
        let stream = astral_yaml::parse_with_options(&source, options)?;
        let document = super::single_document(&stream)?;
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
            source: Source {
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

    /// JSON-compatible quoted input can contain characters that a YAML writer
    /// must escape. Rewrite just those scalars before retaining the source.
    pub(crate) fn escape_non_printable(self) -> Result<Self, Error> {
        if !self.source.text.contains(crate::ser::non_printable) {
            return Ok(self);
        }

        let scalars = self
            .source
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(i, node)| {
                let span = self.source.contents[i];
                (matches!(node.kind, super::NodeKind::Scalar { .. })
                    && self.source.text[span.start()..span.end()]
                        .contains(crate::ser::non_printable))
                .then(|| NodeId::from_index(i))
            });
        super::Editor::new(&self).finish_rewriting(scalars)
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
        Ok(
            emit::parse_rendered(self.into(), &output, self.source.options, region, &[])?
                .into_yaml(),
        )
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

/// Normalize physical line breaks and mark UTF-8 when its first character requires it.
fn normalized_source(source: &str) -> Cow<'_, str> {
    let prefix = if source.starts_with(|c: char| !c.is_ascii() && c != '\u{feff}') {
        "\u{feff}"
    } else {
        ""
    };
    let mut segments = source.split('\r');
    let first = segments
        .next()
        .expect("split always yields a first segment");
    if prefix.is_empty() && first.len() == source.len() {
        return Cow::Borrowed(source);
    }

    let mut normalized = String::with_capacity(source.len() + prefix.len());
    normalized.push_str(prefix);
    normalized.push_str(first);
    for segment in segments {
        normalized.push('\n');
        normalized.push_str(segment.strip_prefix('\n').unwrap_or(segment));
    }
    Cow::Owned(normalized)
}
