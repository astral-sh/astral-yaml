//! Shared development checks and YAML Test Suite event formatting.
use std::fmt::Write;

use astral_yaml::{
    CollectionStyle, Comment, Document, Node, NodeId, NodeKind, ScalarStyle, Span, Stream,
};

/// Formats semantic events using the YAML Test Suite notation.
///
/// # Panics
/// Panics if passed an internally inconsistent stream.
#[must_use]
pub fn events(stream: &Stream<'_>) -> String {
    enum Event {
        Node(NodeId),
        End(&'static str),
    }
    let mut out = String::from("+STR\n");
    for doc in stream.documents() {
        out.push_str(if doc.start_marker.is_some() {
            "+DOC ---\n"
        } else {
            "+DOC\n"
        });
        let mut stack = vec![Event::Node(doc.root)];
        while let Some(event) = stack.pop() {
            match event {
                Event::End(s) => {
                    out.push_str(s);
                    out.push('\n');
                }
                Event::Node(id) => {
                    let node = stream.node(id).expect("valid node id");
                    let header = |out: &mut String, prefix: &str| {
                        out.push_str(prefix);
                        if let Some(anchor) = &node.anchor {
                            write!(out, " &{}", anchor.value).unwrap();
                        }
                        if let Some(tag) = &node.tag {
                            write!(out, " <{}>", event_tag(&tag.value)).unwrap();
                        }
                    };
                    match &node.kind {
                        NodeKind::Scalar(scalar) => {
                            header(&mut out, "=VAL");
                            out.push(' ');
                            out.push(match scalar.style {
                                ScalarStyle::Plain => ':',
                                ScalarStyle::SingleQuoted => '\'',
                                ScalarStyle::DoubleQuoted => '"',
                                ScalarStyle::Literal => '|',
                                ScalarStyle::Folded => '>',
                            });
                            for c in scalar.value.chars() {
                                match c {
                                    '\\' => out.push_str("\\\\"),
                                    '\n' => out.push_str("\\n"),
                                    '\r' => out.push_str("\\r"),
                                    '\t' => out.push_str("\\t"),
                                    '\0' => out.push_str("\\0"),
                                    '\u{8}' => out.push_str("\\b"),
                                    c => out.push(c),
                                }
                            }
                        }
                        NodeKind::Sequence { items, style } => {
                            header(
                                &mut out,
                                if *style == CollectionStyle::Flow {
                                    "+SEQ []"
                                } else {
                                    "+SEQ"
                                },
                            );
                            stack.push(Event::End("-SEQ"));
                            stack.extend(items.iter().rev().map(|&id| Event::Node(id)));
                        }
                        NodeKind::Mapping { entries, style } => {
                            header(
                                &mut out,
                                if *style == CollectionStyle::Flow {
                                    "+MAP {}"
                                } else {
                                    "+MAP"
                                },
                            );
                            stack.push(Event::End("-MAP"));
                            for entry in entries.iter().rev() {
                                stack.push(Event::Node(entry.value));
                                stack.push(Event::Node(entry.key));
                            }
                        }
                        NodeKind::Alias { name, .. } => write!(out, "=ALI *{name}").unwrap(),
                    }
                    out.push('\n');
                }
            }
        }
        out.push_str(if doc.end_marker.is_some() {
            "-DOC ...\n"
        } else {
            "-DOC\n"
        });
    }
    out.push_str("-STR\n");
    out
}

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

// The suite's event convention decodes URI escapes (e.g. 6CK3). The parser's
// public tag value preserves them as required by YAML 1.2.2 section 5.6.
fn event_tag(tag: &str) -> std::borrow::Cow<'_, str> {
    let mut bytes = tag.bytes();
    let mut decoded = Vec::new();
    while let Some(b) = bytes.next() {
        if b == b'%' {
            // Tags only come from the parser, which validates percent escapes.
            let high = bytes
                .next()
                .and_then(|b| char::from(b).to_digit(16))
                .expect("validated hex digit");
            let low = bytes
                .next()
                .and_then(|b| char::from(b).to_digit(16))
                .expect("validated hex digit");
            decoded.push(u8::try_from(high * 16 + low).expect("hex pair fits in u8"));
        } else {
            decoded.push(b);
        }
    }
    String::from_utf8(decoded).map_or_else(|_| tag.into(), std::borrow::Cow::Owned)
}

#[cfg(test)]
mod tests {
    use super::{event_tag, events, validate_parts, validate_spans};
    use astral_yaml::{Span, parse};

    #[test]
    fn event_escaping() {
        let stream = parse("\"\\0\\b\\t\\r\\n\\\\\"").unwrap();
        assert!(events(&stream).contains("=VAL \"\\0\\b\\t\\r\\n\\\\\n"));
        assert_eq!(event_tag("!%FF"), "!%FF");
        assert_eq!(event_tag("!%C3%A9"), "!é");
    }

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
