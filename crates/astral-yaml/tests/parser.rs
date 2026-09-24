use std::borrow::Cow;

use astral_yaml::{NodeKind, ParseOptions, ScalarStyle, Span, parse, parse_with_options};

#[test]
fn prefix_boms_and_multiple_restricted_quoted_characters() {
    let stream = parse("# prefix\n\u{feff}").unwrap();
    assert!(stream.documents().is_empty());
    assert_eq!(stream.byte_order_marks().len(), 1);
    let stream = parse("[\"\u{7f}\", '\u{80}']").unwrap();
    assert_eq!(stream.nodes().len(), 3);
    let source = "[\"\u{7f}\", \u{80}]";
    let error = parse(source).unwrap_err();
    assert_eq!(error.span.slice(source), Some("\u{80}"));
}

#[test]
fn spans_comments_and_locations() {
    let source = "# lead\r\nkey: &a \"v\\n\" # tail\r\ncopy: *a\n";
    let stream = parse(source).unwrap();
    assert_eq!(stream.span().slice(source), Some(source));
    assert_eq!(stream.source(), source);
    let root = stream.node(stream.documents()[0].root).unwrap();
    let NodeKind::Mapping { entries, .. } = &root.kind else {
        panic!("expected map")
    };
    let value = stream.node(entries[0].value).unwrap();
    assert_eq!(value.span.slice(source), Some("&a \"v\\n\""));
    assert_eq!(
        value.anchor.as_ref().unwrap().span.slice(source),
        Some("&a")
    );
    let NodeKind::Scalar(scalar) = &value.kind else {
        panic!("expected scalar")
    };
    assert_eq!(scalar.span.slice(source), Some("\"v\\n\""));
    assert_eq!(scalar.value, "v\n");
    assert_eq!(scalar.style, ScalarStyle::DoubleQuoted);
    assert_eq!(
        stream
            .comments()
            .iter()
            .map(|c| c.span.slice(source).unwrap())
            .collect::<Vec<_>>(),
        ["# lead", "# tail"]
    );
    assert_eq!(stream.location(8), Some((2, 1)));
    assert_eq!(stream.location(source.len() + 1), None);
    assert_eq!(stream.clone(), stream);
    assert!(!format!("{stream:?}").is_empty());
}

#[test]
fn scalar_borrowing_and_unicode() {
    for source in ["café", "'café'", "\"café\""] {
        let stream = parse(source).unwrap();
        let NodeKind::Scalar(s) = &stream.nodes()[0].kind else {
            panic!("expected scalar")
        };
        assert!(matches!(s.value, Cow::Borrowed("café")));
        assert_eq!(stream.location(1), Some((1, 2)));
    }
    let stream = parse("é\r\nx").unwrap();
    assert_eq!(stream.location(1), None);
    assert_eq!(stream.location(2), Some((1, 2)));
    assert_eq!(stream.location(3), Some((2, 1)));
    assert_eq!(stream.location(4), Some((2, 1)));
    assert_eq!(stream.location(5), Some((2, 2)));
}

#[test]
fn span_construction() {
    assert!(Span::new(2, 1).is_none());
    let span = Span::new(1, 3).unwrap();
    assert_eq!((span.start(), span.end(), span.len()), (1, 3, 2));
    assert!(!span.is_empty());
    assert!(Span::default().is_empty());
    assert_eq!(span.slice("abc"), Some("bc"));
    assert_eq!(span.slice("é"), None);
}

#[test]
fn budgets_and_adversarial_nesting() {
    let mut options = ParseOptions::default();
    for max_depth in [0, 129, usize::MAX] {
        options.max_depth = max_depth;
        assert!(parse_with_options("", options).is_err());
    }
    options = ParseOptions::default();
    options.max_nodes = 0;
    assert!(parse_with_options("null", options).is_err());
    options = ParseOptions::default();
    options.max_input_bytes = 2;
    assert!(parse_with_options("abc", options).is_err());
    assert!(parse_with_options("ab", options).is_ok());
    let deep = format!("{}0{}", "[".repeat(10_000), "]".repeat(10_000));
    assert_eq!(parse(&deep).unwrap_err().message, "nesting limit exceeded");
    let mut deep_blocks = String::new();
    for i in 0..1_000 {
        deep_blocks.push_str(&" ".repeat(i));
        deep_blocks.push_str("-\n");
    }
    assert_eq!(
        parse(&deep_blocks).unwrap_err().message,
        "nesting limit exceeded"
    );
}

#[test]
fn aliases_preserve_identity_and_cycles() {
    let stream = parse("&root [*root]").unwrap();
    let id = stream.documents()[0].root;
    let NodeKind::Sequence { items, .. } = &stream.node(id).unwrap().kind else {
        panic!("expected seq")
    };
    assert!(
        matches!(stream.node(items[0]).unwrap().kind, NodeKind::Alias { target, .. } if target == id)
    );
    assert_eq!(stream.nodes().len(), 2);
    assert!(parse("&x a\n---\n*x").is_err());
    assert!(parse("[*missing]").is_err());
}

#[test]
fn invalid_unicode_and_escape_regressions() {
    for source in [
        "\0",
        "\u{7f}",
        "\u{80}",
        "\u{fffe}",
        "\"\\é\"",
        "\"\\uD800\"",
        "\"\\U00110000\"",
        "\"\\u000X\"",
    ] {
        let error = parse(source).unwrap_err();
        assert!(error.span.slice(source).is_some(), "{source:?}: {error:?}");
        assert!(error.to_string().contains(error.message));
    }
    for source in ["{key\n: value}", "[key\n: value]"] {
        let _ = parse(source);
    }
}

#[test]
fn block_scalar_ranges_exclude_following_node() {
    let source = "a: |\n  text\nb: next\n";
    let stream = parse(source).unwrap();
    let scalar = stream
        .nodes()
        .iter()
        .find(|n| matches!(&n.kind, NodeKind::Scalar(s) if s.style == ScalarStyle::Literal))
        .unwrap();
    assert_eq!(scalar.span.slice(source), Some("|\n  text\n"));
}

#[test]
fn json_compatible_quoted_characters() {
    for c in [
        '\u{7f}', '\u{80}', '\u{9f}', '\u{fffe}', '\u{ffff}', '\u{feff}',
    ] {
        for quote in ['\'', '"'] {
            let source = format!("{quote}a{c}b{quote}");
            let stream = parse(&source).unwrap();
            let NodeKind::Scalar(s) = &stream.nodes()[0].kind else {
                panic!("scalar")
            };
            assert_eq!(s.value, format!("a{c}b"));
        }
        assert!(parse(&format!("a{c}b")).is_err());
        assert!(parse(&format!("# a{c}b")).is_err());
    }
    assert!(parse("\"\u{1f}\"").is_err());
}

#[test]
fn document_byte_order_marks() {
    for source in [
        "\u{feff}---\nx\n",
        "x\n...\n\u{feff}---\ny\n",
        "x\n\u{feff}---\ny\n",
        "\u{feff}# comment\n",
    ] {
        let stream = parse(source).unwrap();
        assert_eq!(stream.byte_order_marks().len(), 1);
        assert_eq!(stream.byte_order_marks()[0].slice(source), Some("\u{feff}"));
    }
    assert!(parse("- x\n\u{feff}- y\n").is_err());
    assert!(parse("[x,\u{feff}y]").is_err());
}

#[test]
fn metadata_respects_the_element_budget() {
    let options = ParseOptions {
        max_nodes: 2,
        ..ParseOptions::default()
    };
    for source in ["# a\n# b\n# c\n", "%A\n%B\n%C\n---\n", "# a\n[a,b]"] {
        assert_eq!(
            parse_with_options(source, options).unwrap_err().message,
            "syntax element limit exceeded"
        );
    }
    assert!(parse_with_options("# a\n# b\n", options).is_ok());
}

#[test]
fn invalid_directives_properties_and_keys() {
    for source in [
        "%TAG invalid x\n--- x",
        "%TAG !e! x\n%TAG !e! y\n--- x",
        "%TAG !e! %GG\n--- x",
        "%\n--- x",
        "& x",
        "[&a &b x]",
        "[!a !b x]",
        "!<> x",
        "!<%GG> x",
        "!a%GG x",
        "!!str!x x",
        "%TAG !e! x\n--- !e! x",
        "!a\n!b x",
        "&a\n*a",
        "{",
        "{,}",
        "\"\\",
        "\"\\q\"",
    ] {
        assert!(parse(source).is_err(), "accepted {source:?}");
    }
    assert!(parse(&format!("{}: x", "a".repeat(1025))).is_err());
}

#[test]
fn escape_repertoire_and_empty_pairs() {
    let source = "\"\\0\\a\\b\\t\\n\\v\\f\\r\\e\\ \\\"\\/\\\\\\N\\_\\L\\P\\x41\\u03bb\\U0001F30D\"";
    let stream = parse(source).unwrap();
    let NodeKind::Scalar(s) = &stream.nodes()[0].kind else {
        panic!("scalar")
    };
    assert_eq!(
        s.value,
        "\0\u{7}\u{8}\t\n\u{b}\u{c}\r\u{1b} \"/\\\u{85}\u{a0}\u{2028}\u{2029}Aλ🌍"
    );
    for source in ["[? a]", "[a: ]", "[? : ]", "?\n: a\n", "\"a\\\n\n  b\""] {
        assert!(parse(source).is_ok(), "{source:?}");
    }
}

#[test]
fn tag_uris_preserve_percent_escapes() {
    for (source, tag) in [
        ("!local%21 value", "!local%21"),
        (
            "!<tag:example.com,2000:%c3%a9> value",
            "tag:example.com,2000:%c3%a9",
        ),
        (
            "%TAG !e! tag:example:%FF\n--- !e!suffix value",
            "tag:example:%FFsuffix",
        ),
    ] {
        let stream = parse(source).unwrap();
        assert_eq!(
            stream
                .node(stream.documents()[0].root)
                .unwrap()
                .tag
                .as_ref()
                .unwrap()
                .value,
            tag
        );
    }
    for source in [
        "!é x",
        "!<tag:a|b> x",
        "!<!> x",
        "!<$:?> x",
        "!<relative> x",
        "%TAG !e! relative\n--- !e!path x",
    ] {
        assert!(parse(source).is_err(), "{source}");
    }
}

#[test]
fn multiline_properties_preserve_anchor_identity() {
    for source in [
        "!!seq\n&a [*a]",
        "&a\n!!seq [*a]",
        "&a\n- *a",
        "!!seq\n&a\n- *a",
    ] {
        let stream = parse(source).unwrap();
        let root = stream.documents()[0].root;
        assert_eq!(stream.nodes().len(), 2, "{source}");
        let NodeKind::Sequence { items, .. } = &stream.node(root).unwrap().kind else {
            panic!("sequence")
        };
        assert!(
            matches!(stream.node(items[0]).unwrap().kind, NodeKind::Alias { target, .. } if target == root)
        );
    }
}

#[test]
fn empty_scalar_span_follows_properties() {
    let source = "- &a\n- x\n";
    let stream = parse(source).unwrap();
    let node = stream
        .nodes()
        .iter()
        .find(|node| node.anchor.is_some())
        .unwrap();
    let NodeKind::Scalar(scalar) = &node.kind else {
        panic!("scalar")
    };
    assert_eq!(scalar.span.start(), source.find('\n').unwrap());
    assert!(scalar.span.is_empty());
    assert_eq!(node.span.slice(source), Some("&a"));
}

#[test]
fn continuation_indentation_uses_spaces() {
    for source in [
        "a: plain\n\tcontinued",
        "a: \"first\\\nsecond\"",
        "\"first\\\n---\nsecond\"",
    ] {
        assert!(parse(source).is_err(), "{source:?}");
    }
    for source in ["a: plain\n \tcontinued", "a: \"first\\\n \tsecond\""] {
        assert!(parse(source).is_ok(), "{source:?}");
    }
}

#[test]
fn implicit_key_must_end_at_the_mapping_separator() {
    for source in ["\"a\" extra: b", "[a] extra: b", "{a: b} extra: c"] {
        assert_eq!(
            parse(source).unwrap_err().message,
            "expected ':' after mapping key"
        );
    }
}
