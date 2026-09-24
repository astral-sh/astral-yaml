use std::borrow::Cow;

use astral_yaml::{NodeKind, ParseOptions, ScalarStyle, Span, parse, parse_with_options};

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
