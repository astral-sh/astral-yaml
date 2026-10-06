//! Owned YAML syntax for source-preserving editing.

mod emit;
mod framing;
mod layout;
mod transaction;
mod view;

pub use transaction::{Editor, Handle};

use std::borrow::Cow;

use crate::{
    CollectionStyle, Error, NodeId, ParseOptions, ScalarStyle, Span,
    comments::{CommentIndex, CommentPlacement},
    parse_with_options,
};

/// An owned source document with LF line endings. Node IDs and spans refer only
/// to this document's current, normalized view.
#[derive(Clone, Debug)]
pub struct EditableDocument {
    pub(crate) source: String,
    pub(crate) nodes: Vec<Node>,
    pub(crate) root: NodeId,
    pub(crate) options: ParseOptions,
    pub(crate) start_marker: Option<Span>,
    pub(crate) end_marker: Option<Span>,
    pub(crate) extra_end_markers: Vec<Span>,
}

/// An owned syntax node. Properties and comments belong to the occurrence,
/// including when the occurrence is an alias.
///
/// `Location` separates syntax from its source provenance. Committed nodes use
/// [`Span`]; unfinished syntax can use a location type that represents absence.
#[derive(Clone, Debug)]
pub struct Node<Location = Span> {
    pub(crate) span: Location,
    pub(crate) kind: NodeKind,
    pub(crate) anchor: Option<String>,
    pub(crate) tag: Option<String>,
    pub(crate) comments: Vec<Comment<Location>>,
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
    /// An edge to its anchored definition. The spelling comes from the target's
    /// anchor, so a name cannot disagree with the referenced node.
    Alias {
        target: NodeId,
    },
}

/// A comment with its direct owner retained by the containing node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Comment<Location = Span> {
    pub(crate) text: String,
    pub(crate) placement: CommentPlacement,
    pub(crate) position: CommentPosition,
    pub(crate) span: Location,
}

/// Source position and attachment are distinct: comments between a node's
/// properties and content can be attached as trailing comments to that node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CommentPosition {
    Before,
    Header,
    /// An inline comment after a flow collection's opening delimiter.
    Opening,
    /// A collection-owned comment before an item or mapping key.
    BeforeChild(NodeId),
    After,
}

impl CommentPosition {
    fn for_comment(
        stream: &crate::Stream<'_>,
        owner: NodeId,
        attached: &crate::comments::AttachedComment<'_>,
        content_start: usize,
    ) -> Self {
        let node = &stream.nodes()[owner.0];
        let at = attached.comment.span.start();
        if at < node.span.start() {
            CommentPosition::Before
        } else if at < content_start {
            CommentPosition::Header
        } else if attached.placement == CommentPlacement::Inline
            && at < node.span.end()
            && matches!(
                node.kind,
                crate::NodeKind::Sequence {
                    style: CollectionStyle::Flow,
                    ..
                } | crate::NodeKind::Mapping {
                    style: CollectionStyle::Flow,
                    ..
                }
            )
        {
            CommentPosition::Opening
        } else if attached.placement == CommentPlacement::Detached {
            let following = match &node.kind {
                crate::NodeKind::Sequence { items, .. } => {
                    let index = items.partition_point(|id| stream.nodes()[id.0].span.start() <= at);
                    items.get(index).copied()
                }
                crate::NodeKind::Mapping { entries, .. } => {
                    let index = entries
                        .partition_point(|entry| stream.nodes()[entry.key.0].span.start() <= at);
                    entries.get(index).map(|entry| entry.key)
                }
                _ => None,
            };
            following.map_or(CommentPosition::After, CommentPosition::BeforeChild)
        } else {
            CommentPosition::After
        }
    }
}

impl EditableDocument {
    /// Retains exactly one YAML document, normalizing CR and CRLF line endings
    /// to LF. All source spans and diagnostics refer to the normalized source.
    ///
    /// # Errors
    /// Returns a syntax, document-count, or resource-limit error.
    pub fn parse(source: &str) -> Result<Self, Error> {
        Self::parse_with_options(source, ParseOptions::default())
    }

    /// Retains one document using explicit parser limits and LF line endings.
    /// The input byte limit applies before normalization. Source spans and
    /// diagnostics refer to the normalized source.
    ///
    /// # Errors
    /// Returns a syntax, document-count, or resource-limit error.
    pub fn parse_with_options(source: &str, options: ParseOptions) -> Result<Self, Error> {
        options.check_input(source.len())?;
        let source = lf_source(source);
        let stream = parse_with_options(&source, options)?;
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
                    crate::NodeKind::Alias { target, .. } => NodeKind::Alias { target: *target },
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
        let mut content_starts = vec![None; nodes.len()];
        for (owner, attached) in CommentIndex::new(&stream, 0)?.direct() {
            let node = &stream.nodes()[owner.0];
            let content_start = *content_starts[owner.0].get_or_insert_with(|| {
                let properties_end = node
                    .anchor
                    .iter()
                    .chain(node.tag.iter())
                    .map(|property| property.span.end())
                    .max()
                    .unwrap_or(node.span.start());
                crate::comments::skip_trivia(&source, properties_end).min(node.span.end())
            });
            let position = CommentPosition::for_comment(&stream, owner, attached, content_start);
            nodes[owner.0].comments.push(Comment {
                text: attached.comment.text.to_owned(),
                placement: attached.placement,
                position,
                span: attached.comment.span,
            });
        }
        Ok(Self {
            nodes,
            root: document.root,
            options,
            start_marker: document.start_marker,
            end_marker: document.end_marker,
            extra_end_markers: stream
                .document_end_markers()
                .iter()
                .copied()
                .filter(|marker| {
                    document
                        .end_marker
                        .is_some_and(|first| marker.start() > first.start())
                })
                .collect(),
            source: source.into_owned(),
        })
    }

    /// Returns the document's current source with LF line endings.
    #[must_use]
    pub fn to_yaml(&self) -> String {
        self.source.clone()
    }

    /// Consumes the document and returns its committed source without cloning it.
    #[must_use]
    pub fn into_yaml(self) -> String {
        self.source
    }

    /// Emits the document with normalized whitespace, preserving comments and
    /// compatible presentation styles. Source outside the root region is retained.
    ///
    /// # Errors
    /// Returns an error if emission exceeds resource limits or cannot preserve
    /// scalar text, alias identity, or comment ownership and placement.
    pub fn normalized_yaml(&self) -> Result<String, Error> {
        let region = emit::region(self, self.root, false);
        let indent = emit::indent(&self.source, region.start());
        let rendered = emit::render(self, self.root, region, indent, false)?;
        let mut output = self.source[..rendered.span.start()].to_owned();
        output.push_str(&rendered.text);
        output.push_str(&self.source[rendered.span.end()..]);
        let parsed = Self::parse_with_options(&output, self.options)?;
        let correspondence = emit::correspondence(self, &parsed).ok_or(Error {
            span: region,
            message: "emission changed YAML content or alias identity",
        })?;
        if !emit::comments_preserved(self, &parsed, &correspondence) {
            return Err(Error {
                span: region,
                message: "emission changed comment ownership or placement",
            });
        }
        Ok(output)
    }

    /// The current committed UTF-8 source, with LF line endings.
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

impl<Location> Node<Location> {
    /// The node's syntax content.
    #[must_use]
    pub fn kind(&self) -> &NodeKind {
        &self.kind
    }

    /// The node's source location. Committed nodes return their current span.
    #[must_use]
    pub fn span(&self) -> Location
    where
        Location: Copy,
    {
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
    pub fn comments(&self) -> &[Comment<Location>] {
        &self.comments
    }
}

impl<Location> Comment<Location> {
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

    /// The comment's source location. Committed comments return their current span.
    #[must_use]
    pub fn span(&self) -> Location
    where
        Location: Copy,
    {
        self.span
    }
}
