//! YAML Test Suite event formatting.

use std::fmt::Write;

use astral_yaml::{CollectionStyle, NodeId, NodeKind, ScalarStyle, Stream};

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
    use super::{event_tag, events};
    use astral_yaml::parse;

    #[test]
    fn event_escaping() {
        let stream = parse("\"\\0\\b\\t\\r\\n\\\\\"").unwrap();
        assert!(events(&stream).contains("=VAL \"\\0\\b\\t\\r\\n\\\\\n"));
        assert_eq!(event_tag("!%FF"), "!%FF");
        assert_eq!(event_tag("!%C3%A9"), "!é");
    }
}
