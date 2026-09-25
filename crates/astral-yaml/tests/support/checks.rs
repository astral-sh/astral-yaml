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
pub(crate) fn events(stream: &Stream<'_>) -> String {
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

// The suite's event convention decodes URI escapes (e.g. 6CK3). The parser's
// public tag value preserves them as required by YAML 1.2.2 section 5.6.
pub(crate) fn event_tag(tag: &str) -> std::borrow::Cow<'_, str> {
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
