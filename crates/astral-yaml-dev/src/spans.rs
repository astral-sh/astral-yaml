//! Source span and arena invariant checks shared with fuzz targets.

use astral_yaml::{Comment, Document, Node, NodeId, NodeKind, Span, Stream};

/// Checks source boundaries, parent ranges, comments, and arena references.
///
/// # Errors
/// Returns a description of the first broken invariant.
pub fn validate_spans(stream: &Stream<'_>) -> Result<(), String> {
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

fn validate_parts<'a: 's, 's>(
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

#[cfg(test)]
mod tests {
    use super::{validate_parts, validate_spans};
    use astral_yaml::{Span, parse};

    #[test]
    fn validator_rejects_bad_ranges_and_references() {
        for source in ["x", "[a]", "{a: b}", "[&a x, *a]"] {
            let stream = parse(source).unwrap();
            validate_spans(&stream).unwrap();
            let error = validate_parts(
                source,
                stream.nodes(),
                stream.documents(),
                &[],
                &[],
                &[],
                &|_| None,
            )
            .unwrap_err();
            assert!(error.contains("ID"));
        }

        let stream = parse("{a: b}").unwrap();
        let mut value = None;

        for node in stream.nodes() {
            if let astral_yaml::NodeKind::Mapping { entries, .. } = &node.kind {
                value = Some(entries[0].value);
            }
        }

        let value = value.unwrap();
        assert_eq!(
            validate_parts(stream.source(), stream.nodes(), &[], &[], &[], &[], &|id| {
                if id == value { None } else { stream.node(id) }
            })
            .unwrap_err(),
            "invalid value ID"
        );

        let stream = parse("[&a x, *a]").unwrap();
        let alias = stream.nodes().last().unwrap();
        assert_eq!(
            validate_parts(
                stream.source(),
                std::slice::from_ref(alias),
                &[],
                &[],
                &[],
                &[],
                &|_| None
            )
            .unwrap_err(),
            "invalid alias ID"
        );

        let stream = parse("'é'").unwrap();
        let lookup = &|id| stream.node(id);
        validate_parts(
            stream.source(),
            stream.nodes(),
            stream.documents(),
            &[],
            &[],
            &[],
            lookup,
        )
        .unwrap();

        let mut nodes = stream.nodes().to_vec();
        nodes[0].span = Span::new(0, 10).unwrap();
        assert!(
            validate_parts(stream.source(), &nodes, &[], &[], &[], &[], lookup)
                .unwrap_err()
                .contains("invalid span")
        );

        nodes[0].span = Span::new(0, 1).unwrap();
        assert!(
            validate_parts(stream.source(), &nodes, &[], &[], &[], &[], lookup)
                .unwrap_err()
                .contains("outside parent")
        );
    }

    #[test]
    fn validator_checks_comments_and_byte_order_marks() {
        let stream = parse("\u{feff}# one\n# two\n%YAML 1.2\n---\nx\n...\n").unwrap();
        validate_spans(&stream).unwrap();

        let check = |boms: &[Span], comments: &[astral_yaml::Comment<'_>]| {
            validate_parts(
                stream.source(),
                stream.nodes(),
                stream.documents(),
                boms,
                stream.document_end_markers(),
                comments,
                &|id| stream.node(id),
            )
        };

        let bom = stream.byte_order_marks()[0];
        assert!(check(&[bom, bom], &[]).is_err());
        assert!(check(&[Span::new(3, 4).unwrap()], &[]).is_err());

        let mut comments = stream.comments().to_vec();
        comments.reverse();
        assert!(check(&[], &comments).is_err());

        comments[0].text = "different text";
        assert!(check(&[], &comments[..1]).is_err());
    }
}
