//! Source span and arena invariant checks shared with fuzz targets.

use astral_yaml::{Comment, Document, Node, NodeId, NodeKind, Span, Stream};

/// Checks source boundaries, parent ranges, comments, and arena references.
///
/// # Errors
/// Returns a description of the first broken invariant.
pub(crate) fn validate_spans(stream: &Stream<'_>) -> Result<(), String> {
    validate_parts(
        stream.source(),
        stream.nodes(),
        stream.documents(),
        stream.byte_order_marks(),
        stream.document_end_markers(),
        stream.comments(),
        &|id| stream.node(id),
    )
}

pub(crate) fn validate_parts<'a: 's, 's>(
    source: &str,
    nodes: &[Node<'a>],
    documents: &[Document<'a>],
    byte_order_marks: &[Span],
    document_end_markers: &[Span],
    comments: &[Comment<'a>],
    node: &dyn Fn(NodeId) -> Option<&'s Node<'a>>,
) -> Result<(), String> {
    let valid = |span: Span| -> Result<(), String> {
        if span.slice(source).is_some() {
            Ok(())
        } else {
            Err(format!("invalid span: {span:?}"))
        }
    };

    let contains = |outer: Span, inner: Span| -> Result<(), String> {
        valid(inner)?;
        if outer.start() <= inner.start() && inner.end() <= outer.end() {
            Ok(())
        } else {
            Err(format!("span {inner:?} outside parent {outer:?}"))
        }
    };

    for current in nodes {
        valid(current.span)?;

        for p in current.anchor.iter().chain(current.tag.iter()) {
            contains(current.span, p.span)?;
        }

        match &current.kind {
            NodeKind::Scalar(s) => contains(current.span, s.span)?,
            NodeKind::Sequence { items, .. } => {
                for &id in items {
                    contains(current.span, node(id).ok_or("invalid child ID")?.span)?;
                }
            }
            NodeKind::Mapping { entries, .. } => {
                for e in entries {
                    contains(current.span, e.span)?;
                    contains(e.span, node(e.key).ok_or("invalid key ID")?.span)?;
                    contains(e.span, node(e.value).ok_or("invalid value ID")?.span)?;
                }
            }
            NodeKind::Alias { target, .. } => {
                node(*target).ok_or("invalid alias ID")?;
            }
        }
    }

    for doc in documents {
        valid(doc.span)?;
        contains(doc.span, node(doc.root).ok_or("invalid root ID")?.span)?;

        for &marker in doc.start_marker.iter().chain(doc.end_marker.iter()) {
            contains(doc.span, marker)?;
        }

        for directive in &doc.directives {
            contains(doc.span, directive.span)?;
        }
    }

    for (spans, expected) in [
        (byte_order_marks, "\u{feff}"),
        (document_end_markers, "..."),
    ] {
        let mut end = 0;

        for &span in spans {
            valid(span)?;
            if span.start() < end || span.slice(source) != Some(expected) {
                return Err(format!("invalid {expected:?} marker span"));
            }

            end = span.end();
        }
    }

    let mut end = 0;

    for comment in comments {
        valid(comment.span)?;
        if comment.span.start() < end
            || comment.span.slice(source).and_then(|s| s.strip_prefix('#')) != Some(comment.text)
        {
            return Err(format!("invalid comment: {comment:?}"));
        }

        end = comment.span.end();
    }

    Ok(())
}
