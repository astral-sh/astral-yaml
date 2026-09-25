use std::fmt::Write;

use astral_yaml::{CollectionStyle, Node, NodeId, NodeKind, ScalarStyle, Span, Stream};

fn location(stream: &Stream<'_>, span: Span) -> String {
    let (start_line, start_column) = stream.location(span.start()).unwrap();
    let (end_line, end_column) = stream.location(span.end()).unwrap();

    format!("{start_line}:{start_column}..{end_line}:{end_column}")
}

fn scalar_style(style: ScalarStyle) -> &'static str {
    match style {
        ScalarStyle::Plain => "plain",
        ScalarStyle::SingleQuoted => "single-quoted",
        ScalarStyle::DoubleQuoted => "double-quoted",
        ScalarStyle::Literal => "literal",
        ScalarStyle::Folded => "folded",
    }
}

fn collection_style(style: CollectionStyle) -> &'static str {
    match style {
        CollectionStyle::Block => "block",
        CollectionStyle::Flow => "flow",
    }
}

fn header(stream: &Stream<'_>, node: &Node<'_>) -> String {
    let mut out = match &node.kind {
        NodeKind::Scalar(scalar) => {
            let style = scalar_style(scalar.style);
            let span = location(stream, scalar.span);

            if scalar.value.contains('\n') {
                format!("{style} [@{span}]")
            } else {
                format!("{:?} [{style} @{span}]", scalar.value)
            }
        }
        NodeKind::Sequence { style, .. } => {
            format!("sequence [{}]", collection_style(*style))
        }
        NodeKind::Mapping { style, .. } => {
            format!("mapping [{}]", collection_style(*style))
        }
        NodeKind::Alias { name, .. } => {
            format!("*{name} [@{}]", location(stream, node.span))
        }
    };

    if let Some(anchor) = &node.anchor {
        write!(out, " &{}", anchor.value).unwrap();
    }

    if let Some(tag) = &node.tag {
        write!(out, " !<{}>", tag.value).unwrap();
    }

    out
}

fn node(out: &mut String, stream: &Stream<'_>, id: NodeId, depth: usize, prefix: &str) {
    let current = stream.node(id).unwrap();
    let indent = "  ".repeat(depth);
    writeln!(out, "{indent}{prefix}{}", header(stream, current)).unwrap();

    match &current.kind {
        NodeKind::Scalar(scalar) if scalar.value.contains('\n') => {
            // Quote each decoded line to expose trailing spaces, blank lines,
            // and the presence or absence of a final newline.
            for line in scalar.value.split_inclusive('\n') {
                writeln!(out, "{indent}  {line:?}").unwrap();
            }
        }
        NodeKind::Sequence { items, .. } => {
            for &item in items {
                node(out, stream, item, depth + 1, "- ");
            }
        }
        NodeKind::Mapping { entries, .. } => {
            for entry in entries {
                let key = stream.node(entry.key).unwrap();

                if matches!(&key.kind, NodeKind::Scalar(scalar) if !scalar.value.contains('\n')) {
                    let prefix = format!("{}: ", header(stream, key));
                    node(out, stream, entry.value, depth + 1, &prefix);
                } else {
                    node(out, stream, entry.key, depth + 1, "? ");
                    node(out, stream, entry.value, depth + 1, ": ");
                }
            }
        }
        NodeKind::Scalar(_) | NodeKind::Alias { .. } => {}
    }
}

pub(super) fn tree(stream: &Stream<'_>) -> String {
    let mut out = String::new();

    for (index, document) in stream.documents().iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }

        let start = if document.start_marker.is_some() {
            "---"
        } else {
            "implicit"
        };
        let end = if document.end_marker.is_some() {
            "..."
        } else {
            "implicit"
        };

        writeln!(out, "document {} (start: {start}, end: {end})", index + 1).unwrap();
        node(&mut out, stream, document.root, 0, "");
    }

    out
}

pub(super) fn diagnostic(name: &str, source: &str, error: &astral_yaml::Error) -> String {
    // Corpus files use LF. Include the byte range because a multiline error's
    // caret covers only its first line.
    let prefix = &source[..error.span.start()];
    let line = prefix.bytes().filter(|&byte| byte == b'\n').count() + 1;
    let column = prefix.rsplit('\n').next().unwrap().chars().count() + 1;
    let mut out = format!(
        "{name}.yaml:{line}:{column}: {}\nbytes {}..{}\n",
        error.message,
        error.span.start(),
        error.span.end(),
    );

    for (index, text) in source.lines().enumerate() {
        let number = index + 1;

        if number < line.saturating_sub(2) || number > line + 1 {
            continue;
        }

        writeln!(out, "{number:>4} | {text}").unwrap();

        if number == line {
            let width = error
                .span
                .slice(source)
                .unwrap()
                .split('\n')
                .next()
                .unwrap()
                .chars()
                .count()
                .max(1);

            writeln!(
                out,
                "     | {}{}",
                " ".repeat(column - 1),
                "^".repeat(width)
            )
            .unwrap();
        }
    }

    out
}

#[test]
fn tree_preserves_presentation_and_decoded_lines() {
    let source = "%YAML 1.2\n---\nbase: &base !local {quoted: 'é', empty:}\nalias: *base\n? [a, b]\n: >-\n  folded\n  text\nscript: |+\n  echo \"hello\"\n\n...\n--- []\n";
    let stream = astral_yaml::parse(source).unwrap();

    insta::assert_snapshot!(tree(&stream), @r#"
    document 1 (start: ---, end: ...)
    mapping [block]
      "base" [plain @3:1..3:5]: mapping [flow] &base !<!local>
        "quoted" [plain @3:21..3:27]: "é" [single-quoted @3:29..3:32]
        "empty" [plain @3:34..3:39]: "" [plain @3:40..3:40]
      "alias" [plain @4:1..4:6]: *base [@4:8..4:13]
      ? sequence [flow]
        - "a" [plain @5:4..5:5]
        - "b" [plain @5:7..5:8]
      : "folded text" [folded @6:3..9:1]
      "script" [plain @9:1..9:7]: literal [@9:9..12:1]
        "echo \"hello\"\n"
        "\n"

    document 2 (start: ---, end: implicit)
    sequence [flow]
    "#);
}

#[test]
fn tree_distinguishes_line_endings_and_empty_scalars() {
    let source = "[plain, '', \"\", \"line\\nend\", \"line\\n\", \"line\\n\\n\", \"\\t\\r\", ]";
    let stream = astral_yaml::parse(source).unwrap();

    insta::assert_snapshot!(tree(&stream), @r#"
    document 1 (start: implicit, end: implicit)
    sequence [flow]
      - "plain" [plain @1:2..1:7]
      - "" [single-quoted @1:9..1:11]
      - "" [double-quoted @1:13..1:15]
      - double-quoted [@1:17..1:28]
        "line\n"
        "end"
      - double-quoted [@1:30..1:38]
        "line\n"
      - double-quoted [@1:40..1:50]
        "line\n"
        "\n"
      - "\t\r" [double-quoted @1:52..1:58]
    "#);
}
