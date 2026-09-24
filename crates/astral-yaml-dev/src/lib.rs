//! Shared development checks and YAML Test Suite event formatting.
use std::fmt::Write;

use astral_yaml::{CollectionStyle, NodeId, NodeKind, ScalarStyle, Span, Stream};

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
                    match &node.kind {
                        NodeKind::Scalar(_) => out.push_str("=VAL"),
                        NodeKind::Sequence { style, .. } => {
                            out.push_str(if *style == CollectionStyle::Flow {
                                "+SEQ []"
                            } else {
                                "+SEQ"
                            });
                        }
                        NodeKind::Mapping { style, .. } => {
                            out.push_str(if *style == CollectionStyle::Flow {
                                "+MAP {}"
                            } else {
                                "+MAP"
                            });
                        }
                        NodeKind::Alias { name, .. } => {
                            writeln!(out, "=ALI *{name}").unwrap();
                            continue;
                        }
                    }
                    if let Some(anchor) = &node.anchor {
                        write!(out, " &{}", anchor.value).unwrap();
                    }
                    if let Some(tag) = &node.tag {
                        write!(out, " <{}>", tag.value).unwrap();
                    }
                    match &node.kind {
                        NodeKind::Scalar(scalar) => {
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
                        NodeKind::Sequence { items, .. } => {
                            stack.push(Event::End("-SEQ"));
                            stack.extend(items.iter().rev().map(|&id| Event::Node(id)));
                        }
                        NodeKind::Mapping { entries, .. } => {
                            stack.push(Event::End("-MAP"));
                            for entry in entries.iter().rev() {
                                stack.push(Event::Node(entry.value));
                                stack.push(Event::Node(entry.key));
                            }
                        }
                        NodeKind::Alias { .. } => unreachable!(),
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
    let valid = |span: Span| -> Result<(), String> {
        if span.slice(stream.source()).is_some() {
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
    for node in stream.nodes() {
        valid(node.span)?;
        for p in node.anchor.iter().chain(node.tag.iter()) {
            contains(node.span, p.span)?;
        }
        match &node.kind {
            NodeKind::Scalar(s) => contains(node.span, s.span)?,
            NodeKind::Sequence { items, .. } => {
                for &id in items {
                    contains(node.span, stream.node(id).ok_or("invalid child ID")?.span)?;
                }
            }
            NodeKind::Mapping { entries, .. } => {
                for e in entries {
                    contains(node.span, e.span)?;
                    contains(e.span, stream.node(e.key).ok_or("invalid key ID")?.span)?;
                    contains(e.span, stream.node(e.value).ok_or("invalid value ID")?.span)?;
                }
            }
            NodeKind::Alias { target, .. } => {
                stream.node(*target).ok_or("invalid alias ID")?;
            }
        }
    }
    for doc in stream.documents() {
        valid(doc.span)?;
        contains(
            doc.span,
            stream.node(doc.root).ok_or("invalid root ID")?.span,
        )?;
        for &marker in doc.start_marker.iter().chain(doc.end_marker.iter()) {
            contains(doc.span, marker)?;
        }
        for directive in &doc.directives {
            contains(doc.span, directive.span)?;
        }
    }
    let mut end = 0;
    for comment in stream.comments() {
        valid(comment.span)?;
        if comment.span.start() < end
            || comment
                .span
                .slice(stream.source())
                .and_then(|s| s.strip_prefix('#'))
                != Some(comment.text)
        {
            return Err(format!("invalid comment: {comment:?}"));
        }
        end = comment.span.end();
    }
    Ok(())
}
