use test_case::{test_case, test_matrix};

use std::borrow::Cow;

use astral_yaml::{NodeKind, ParseOptions, ScalarStyle, Span, parse, parse_with_options};
use astral_yaml_dev::span_snapshot;

// https://yaml.org/spec/1.2.2/#51-character-set
#[test_case(0)]
#[test_case(7)]
#[test_case(8)]
#[test_case(15)]
#[test_case(16)]
#[test_case(31)]
#[test_case(32)]
#[test_case(63)]
#[test_case(64)]
#[test_case(65)]
#[test_case(127)]
fn ascii_character_checks_cover_chunk_boundaries(padding: usize) {
    for byte in 0..=127 {
        let character = char::from(byte);
        let source = format!("# {}{character}\n", "a".repeat(padding));
        let allowed = matches!(byte, b'\t' | b'\r' | b'\n' | b' '..=b'~');
        let result = parse(&source);
        assert_eq!(result.is_ok(), allowed, "{source:?}");

        if let Err(error) = result {
            let expected = character.to_string();
            assert_eq!(error.span.slice(&source), Some(expected.as_str()));
        }
    }
}

// https://yaml.org/spec/1.2.2/#742-flow-mappings
#[test]
fn an_empty_explicit_flow_pair_can_precede_another_item() {
    let events = astral_yaml_dev::events(&parse("[? , x]").unwrap());
    insta::assert_snapshot!(events, @"
    +STR
    +DOC
    +SEQ []
    +MAP {}
    =VAL :
    =VAL :
    -MAP
    =VAL :x
    -SEQ
    -DOC
    -STR
    ");
}

// https://yaml.org/spec/1.2.2/#912-document-markers
#[test_case("{a:\n---\n}")]
#[test_case("[a:\n... # end\n]")]
#[test_case("{a: &anchor\n---\r\n}")]
#[test_case("[?\r...\r]")]
fn a_document_marker_cannot_supply_a_flow_node(source: &str) {
    assert_eq!(
        parse(source).unwrap_err().message,
        "document marker inside flow collection"
    );
}

#[test_case("{a:\n ---\n}", "---")]
#[test_case("{a:\n...suffix\n}", "...suffix")]
fn flow_values_can_resemble_document_markers(source: &str, expected: &str) {
    let parsed = parse(source).unwrap();
    let NodeKind::Scalar(value) = &parsed.nodes()[2].kind else {
        panic!("expected scalar")
    };
    assert_eq!(value.value, expected);
}

// https://yaml.org/spec/1.2.2/#73-flow-scalar-styles
#[test_case("\n")]
#[test_case("\r")]
#[test_case("\r\n")]
fn line_tracking_survives_scalar_lookahead(newline: &str) {
    let template = "\u{feff}first: one\nsecond:\n  - two\n  - three\nfolded: four\n  five\nlast: six\n---\n[seven,\n eight]\n";

    let source = template.replace('\n', newline);
    let stream = parse(&source).unwrap();
    assert_eq!(stream.documents().len(), 2);
    let scalars: Vec<_> = stream
        .nodes()
        .iter()
        .filter_map(|node| match &node.kind {
            NodeKind::Scalar(scalar) => Some(scalar.value.as_ref()),
            _ => None,
        })
        .collect();
    assert_eq!(
        scalars,
        [
            "first",
            "one",
            "second",
            "two",
            "three",
            "folded",
            "four five",
            "last",
            "six",
            "seven",
            "eight",
        ]
    );
}

// https://yaml.org/spec/1.2.2/#54-line-break-characters
// https://yaml.org/spec/1.2.2/#73-flow-scalar-styles
#[test_matrix(["\n", "\r", "\r\n"], [0, 15, 31, 64])]
fn scalar_scans_preserve_content_and_spans_across_line_endings(newline: &str, padding: usize) {
    let prefix = format!("{}é", "a".repeat(padding));

    for (body, expected) in [
        (
            format!("{prefix}:url#fragment \t"),
            format!("{prefix}:url#fragment"),
        ),
        (
            format!("{prefix}\n  continued\t"),
            format!("{prefix} continued"),
        ),
        (format!("'{prefix}''tail'"), format!("{prefix}'tail")),
        (format!("\"{prefix}\\\"tail\""), format!("{prefix}\"tail")),
        (
            format!("\"{prefix}\n  continued\""),
            format!("{prefix} continued"),
        ),
    ] {
        let body = body.replace('\n', newline);
        let source = format!("key: {body} # {prefix}comment{newline}last: done{newline}");
        let stream = parse(&source).unwrap();
        let root = stream.node(stream.documents()[0].root).unwrap();
        let NodeKind::Mapping { entries, .. } = &root.kind else {
            panic!("expected mapping");
        };
        let NodeKind::Scalar(scalar) = &stream.node(entries[0].value).unwrap().kind else {
            panic!("expected scalar");
        };

        assert_eq!(scalar.value, expected, "{source:?}");
        assert_eq!(
            scalar.span.slice(&source),
            Some(body.trim_end_matches([' ', '\t']))
        );
        assert_eq!(stream.comments().len(), 1);
        assert_eq!(stream.comments()[0].text, format!(" {prefix}comment"));
        assert_eq!(entries.len(), 2);
    }
}

// https://yaml.org/spec/1.2.2/#57-escaped-characters
#[test_case(0x10000_u32)]
#[test_case(0x1d11e_u32)]
#[test_case(0x1f30d_u32)]
#[test_case(0x0010_ffff_u32)]
fn json_surrogate_pairs(value: u32) {
    let character = char::from_u32(value).unwrap();
    let mut units = [0; 2];
    character.encode_utf16(&mut units);
    let source = format!(r#""before\u{:04X}\u{:04X}after""#, units[0], units[1]);
    let stream = parse(&source).unwrap();
    let NodeKind::Scalar(scalar) = &stream.nodes()[0].kind else {
        panic!("expected scalar");
    };

    assert_eq!(scalar.value, format!("before{character}after"));
    assert_eq!(scalar.span.slice(&source), Some(source.as_str()));
}

#[test_case(r#""\uD800""#)]
#[test_case(r#""\uD800"#)]
#[test_case(r#""\uDC00""#)]
#[test_case(r#""\uDC00\uD800""#)]
#[test_case(r#""\uD800\uD800""#)]
#[test_case(r#""\uD800\u0041""#)]
#[test_case(r#""\uD800\uFFFF""#)]
#[test_case(r#""\uD800x""#)]
#[test_case(r#""\uD800\n""#)]
#[test_case(r#""\uD800\u""#)]
#[test_case(r#""\uD800\uZZZZ""#)]
#[test_case(r#""\uD800 \uDC00""#)]
#[test_case(r#""\uD800\U0000DC00""#)]
#[test_case(r#""\U0000D800\uDC00""#)]
fn malformed_surrogate_escapes(source: &str) {
    let error = parse(source).unwrap_err();
    assert!(error.span.slice(source).is_some(), "{source}");
}

// https://yaml.org/spec/1.2.2/#8111-block-indentation-indicator
// https://yaml.org/spec/1.2.2/#8112-block-chomping-indicator
#[test_case("|")]
#[test_case(">")]
fn block_scalar_eof_and_indentation_follow_the_test_suite(style: &str) {
    // Follow the suite's EOF-as-line-break interpretation (yaml/yaml-test-suite#118).
    for (body, clipped, kept) in [
        ("  text", "text\n", "text\n"),
        ("  text\n", "text\n", "text\n"),
        ("  text\r", "text\n", "text\n"),
        ("  text\r\n", "text\n", "text\n"),
        ("  text\n ", "text\n", "text\n\n"),
        ("  text\n   ", "text\n \n", "text\n \n"),
        ("", "", ""),
        ("  ", "", "\n"),
    ] {
        for (chomp, expected) in [
            ("", clipped),
            ("+", kept),
            ("-", clipped.trim_end_matches('\n')),
        ] {
            let source = format!("{style}{chomp}\n{body}");
            let stream = parse(&source).unwrap();
            let NodeKind::Scalar(scalar) = &stream.nodes()[0].kind else {
                panic!("expected scalar")
            };

            assert_eq!(scalar.value, expected, "{source:?}");
            assert_eq!(scalar.span.slice(&source), Some(source.as_str()));
        }
    }

    // yaml/yaml-test-suite#37: an indented # is the first content line,
    // not a trailing comment. Prior empty lines must not be more indented.
    let source = format!("key: {style}\n\n   \n  # content\n");
    assert_eq!(
        parse(&source).unwrap_err().message,
        "leading empty line is over-indented"
    );
}

// https://yaml.org/spec/1.2.2/#911-document-prefix
#[test_case("|1-", "first\n", "first")]
#[test_case(">", "", "")]
fn a_root_block_scalar_stops_at_the_next_document_bom(header: &str, body: &str, expected: &str) {
    let source = format!("{header}\n{body}\u{feff}--- second");
    let stream = parse(&source).unwrap();
    let [first, second] = stream.documents() else {
        panic!("expected two documents")
    };
    let scalar = |id| match &stream.node(id).unwrap().kind {
        NodeKind::Scalar(s) => s.value.as_ref(),
        _ => panic!("expected scalar"),
    };
    assert_eq!(
        (scalar(first.root), scalar(second.root)),
        (expected, "second")
    );
    assert_eq!(
        stream.byte_order_marks()[0].slice(&source),
        Some("\u{feff}")
    );
}

#[test_case("|-")]
#[test_case(">-")]
fn stripping_a_long_run_of_blank_lines(header: &str) {
    let source = format!("{header}\n{}", "\n".repeat(128 * 1024));
    let parsed = parse(&source).unwrap();
    let NodeKind::Scalar(value) = &parsed.nodes()[0].kind else {
        panic!("expected scalar")
    };
    assert_eq!(value.value, "");
}

// https://yaml.org/spec/1.2.2/#913-bare-documents
// https://yaml.org/spec/1.2.2/#8111-block-indentation-indicator
#[test_case("|1\ntext\n", "text\n")]
#[test_case("|2\n  foo\n", " foo\n")]
#[test_case("--- &root >2\n  foo\n", " foo\n")]
fn explicit_root_block_indentation_starts_at_minus_one(source: &str, expected: &str) {
    let parsed = parse(source).unwrap();
    let NodeKind::Scalar(value) = &parsed.nodes()[0].kind else {
        panic!("expected scalar")
    };
    assert_eq!(value.value, expected);
}

#[test_case("|\n  \ttext\n", "\ttext\n")]
#[test_case(">\n  \t\n  text\n", "\t\ntext\n")]
fn block_scalar_tabs_after_indentation_are_content(source: &str, expected: &str) {
    // A tab after the detected spaces is content (YAML 1.2.2 example 8.2).
    let stream = parse(source).unwrap();
    let NodeKind::Scalar(scalar) = &stream.nodes()[0].kind else {
        panic!("expected scalar")
    };

    assert_eq!(scalar.value, expected);
}

// https://yaml.org/spec/1.2.2/#51-character-set
// https://yaml.org/spec/1.2.2/#52-character-encodings
#[test]
fn prefix_boms_and_multiple_restricted_quoted_characters() {
    let stream = parse("# prefix\n\u{feff}").unwrap();
    assert_eq!(stream.documents(), []);
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

// https://yaml.org/spec/1.2.2/#733-plain-style
#[test]
fn plain_scalars_preserve_unicode_and_embedded_indicators() {
    let source = "café:λ 中#文 🌍{x,y}  \n";
    let stream = parse(source).unwrap();
    let NodeKind::Scalar(scalar) = &stream.nodes()[0].kind else {
        panic!("expected scalar")
    };

    assert_eq!(scalar.value, "café:λ 中#文 🌍{x,y}");
    assert_eq!(scalar.span.slice(source), Some(scalar.value.as_ref()));
    assert!(matches!(scalar.value, Cow::Borrowed(_)));

    let source = "[café:λ, 中#文, 🌍, fin]";
    let stream = parse(source).unwrap();
    let NodeKind::Sequence { items, .. } = &stream.nodes()[0].kind else {
        panic!("expected sequence")
    };

    for (&id, expected) in items.iter().zip(["café:λ", "中#文", "🌍", "fin"]) {
        let NodeKind::Scalar(scalar) = &stream.node(id).unwrap().kind else {
            panic!("expected scalar")
        };

        assert_eq!(scalar.value, expected);
        assert_eq!(scalar.span.slice(source), Some(expected));
    }

    assert_eq!(items.len(), 4);
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

// https://yaml.org/spec/1.2.2/#742-flow-mappings
#[test_case("[a: []]", "[{a: []}]", 3)]
#[test_case("[[[]]: v]", "[{[[]]: v}]", 4)]
#[test_case("[? [a: []] : [b: []]]", "[{? [{a: []}] : [{b: []}]}]", 5)]
#[test_case("[[[x]], a: b]", "[[[x]], {a: b}]", 4)]
fn compact_flow_pairs_have_the_same_depth_as_braced_mappings(
    compact: &str,
    braced: &str,
    required: usize,
) {
    for max_depth in 1..=required {
        let options = ParseOptions {
            max_depth,
            ..ParseOptions::default()
        };
        for source in [compact, braced] {
            assert_eq!(
                parse_with_options(source, options).is_ok(),
                max_depth == required,
                "{source} at depth {max_depth}"
            );
        }
    }
}

#[test_case("&a value", 1)]
#[test_case("&a\n!local\nvalue", 1)]
#[test_case("&a\n[]", 1)]
#[test_case("- &a\n  value", 2)]
#[test_case("k: &a\n  - value", 3)]
#[test_case("-", 2)]
#[test_case("-\n- x", 2)]
#[test_case(": ", 2)]
#[test_case("?", 2)]
#[test_case("outer:\n  -", 3)]
fn node_depth_is_independent_of_properties_and_omitted_content(source: &str, required: usize) {
    for max_depth in 1..=required {
        let options = ParseOptions {
            max_depth,
            ..ParseOptions::default()
        };
        let result = parse_with_options(source, options);
        assert_eq!(
            result.err().map(|error| error.message),
            (max_depth < required).then_some("nesting limit exceeded"),
            "{source:?} at {max_depth}"
        );
    }
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

// https://yaml.org/spec/1.2.2/#71-alias-nodes
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

// https://yaml.org/spec/1.2.2/#51-character-set
// https://yaml.org/spec/1.2.2/#57-escaped-characters
#[test_case("\0")]
#[test_case("\u{7f}")]
#[test_case("\u{80}")]
#[test_case("\u{fffe}")]
#[test_case("\"\\é\"")]
#[test_case("\"\\uD800\"")]
#[test_case("\"\\U00110000\"")]
#[test_case("\"\\u000X\"")]
fn invalid_unicode_and_escape_regressions(source: &str) {
    let error = parse(source).unwrap_err();
    assert!(error.span.slice(source).is_some(), "{source:?}: {error:?}");
    assert!(error.to_string().contains(error.message));
}

// https://yaml.org/spec/1.2.2/#742-flow-mappings
// https://yaml.org/spec/1.2.2/#822-block-mappings
#[test_case("key")]
#[test_case("'key'")]
#[test_case("[key]")]
#[test_case("é")]
#[test_case("'🌍'")]
fn implicit_key_limits_include_separation_before_the_colon(key: &str) {
    for length in [1024, 1025] {
        let padding = " ".repeat(length - key.chars().count());

        for (context, source, accepted) in [
            (
                "block mapping",
                format!("{key}{padding}: value"),
                length == 1024,
            ),
            (
                "flow pair",
                format!("[{key}{padding}: value]"),
                length == 1024,
            ),
            ("flow mapping", format!("{{{key}{padding}: value}}"), true),
            (
                "explicit block key",
                format!("? {key}{padding}\n: value"),
                true,
            ),
        ] {
            assert_eq!(
                parse(&source).is_ok(),
                accepted,
                "{context}: {key:?}, {length} characters"
            );
        }
    }
}

#[test_case("\n")]
#[test_case("\r")]
#[test_case("\r\n")]
fn implicit_keys_restrict_physical_line_breaks(newline: &str) {
    for (source, accepted) in [
        ("{key\n: value}", true),
        ("[key\n: value]", false),
        ("'key\n  tail': value", false),
        ("\"key\\\n  tail\": value", false),
        ("[key,\n  tail]: value", false),
        ("\"key\\ntail\": value", true),
    ] {
        let source = source.replace('\n', newline);
        assert_eq!(parse(&source).is_ok(), accepted, "{source:?}");
    }
}

// https://yaml.org/spec/1.2.2/#812-literal-style
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

// https://yaml.org/spec/1.2.2/#51-character-set
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

// https://yaml.org/spec/1.2.2/#52-character-encodings
#[test_case('\'', false)]
#[test_case('"', false)]
#[test_case('"', true)]
fn byte_order_marks_are_content_inside_quoted_continuations(quote: char, escaped: bool) {
    for line_break in ["\n", "\r\n", "\r"] {
        let escape = if escaped { "\\" } else { "" };
        let separator = if escaped { "" } else { " " };
        let source = format!("{quote}a{escape}{line_break}\u{feff}b{quote}");
        let stream = parse(&source).unwrap();
        let NodeKind::Scalar(scalar) = &stream.nodes()[0].kind else {
            panic!("expected a scalar");
        };

        assert_eq!(scalar.value, format!("a{separator}\u{feff}b"));
        assert_eq!(scalar.span.slice(&source), Some(source.as_str()));
        assert_eq!(stream.byte_order_marks(), []);
    }
}

// https://yaml.org/spec/1.2.2/#52-character-encodings
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

// https://yaml.org/spec/1.2.2/#68-directives
#[test_case("%FOO", "", None)]
#[test_case("%FOO # comment", "", Some(" comment"))]
#[test_case("%FOO\t# comment", "", Some(" comment"))]
#[test_case("%FOO bar#hash # comment", "bar#hash", Some(" comment"))]
#[test_case("%FOO bar baz", "bar baz", None)]
fn directive_parameters_and_comments_are_separate(
    header: &str,
    arguments: &str,
    comment: Option<&str>,
) {
    for newline in ["\n", "\r", "\r\n"] {
        let source = format!("{header}{newline}--- value{newline}");
        let stream = parse(&source).unwrap();
        let directive = &stream.documents()[0].directives[0];
        assert_eq!(directive.arguments, arguments, "{source:?}");
        assert_eq!(
            stream.comments().first().map(|comment| comment.text),
            comment,
            "{source:?}"
        );

        for comment in stream.comments() {
            assert_eq!(
                comment.span.slice(&source),
                Some(format!("#{}", comment.text).as_str())
            );
        }
    }
}

// https://yaml.org/spec/1.2.2/#68-directives
#[test_case("APP", None)]
#[test_case("YAML 1.2", Some("invalid YAML version directive"))]
#[test_case("TAG !e! !prefix/", Some("invalid TAG directive"))]
fn directive_with_many_parameters(header: &str, error: Option<&str>) {
    let arguments = " x".repeat(32_768);
    let source = format!("%{header}{arguments}\n--- value");
    match parse(&source) {
        Ok(stream) => {
            assert_eq!(error, None);
            assert_eq!(
                stream.documents()[0].directives[0].arguments,
                arguments.trim_start()
            );
        }
        Err(err) => assert_eq!(Some(err.message), error),
    }
}

// https://yaml.org/spec/1.2.2/#6821-tag-handles
#[test]
fn undefined_tag_handle_span() {
    let source = "key: !missing!type";
    let error = parse(source).unwrap_err();
    insta::assert_snapshot!(astral_yaml_dev::diagnostic_snapshot(source, error), @"
      |
    1 | key: !missing!type
      |      ^^^^^^^^^ undefined tag handle
    ");
}

// https://yaml.org/spec/1.2.2/#681-yaml-directives
#[test_case("1.1", true)]
#[test_case("1.2", true)]
#[test_case("1.3", true)]
#[test_case("01.02", true)]
#[test_case("0001.0002", true)]
#[test_case("2.0", false)]
#[test_case("1", false)]
#[test_case("1.", false)]
#[test_case(".2", false)]
#[test_case("+1.2", false)]
#[test_case("1.2.3", false)]
#[test_case("1.2 extra", false)]
#[test_case("1.２", false; "non ASCII minor version")]
fn yaml_versions_have_decimal_components(version: &str, accepted: bool) {
    let source = format!("%YAML {version}\n--- value\n");
    assert_eq!(parse(&source).is_ok(), accepted, "{source:?}");
}

// https://yaml.org/spec/1.2.2/#68-directives
// https://yaml.org/spec/1.2.2/#69-node-properties
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

// https://yaml.org/spec/1.2.2/#57-escaped-characters
// https://yaml.org/spec/1.2.2/#742-flow-mappings
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

// https://yaml.org/spec/1.2.2/#56-miscellaneous-characters
// https://yaml.org/spec/1.2.2/#691-node-tags
#[test_case("! value", "!")]
#[test_case("!local%21 value", "!local%21")]
#[test_case("!<tag:example.com,2000:%c3%a9> value", "tag:example.com,2000:%c3%a9")]
fn unchanged_tags_borrow_their_spelling(source: &str, spelling: &str) {
    let stream = parse(source).unwrap();
    let tag = stream.nodes()[0].tag.as_ref().unwrap();
    let Cow::Borrowed(value) = tag.value else {
        panic!("expected a borrowed tag");
    };
    assert_eq!(value, spelling);
}

// https://yaml.org/spec/1.2.2/#682-tag-directives
#[test]
fn tag_expansion_preserves_percent_escapes() {
    let stream = parse("%TAG !e! tag:example:%FF\n--- !e!suffix value").unwrap();
    let tag = stream.nodes()[0].tag.as_ref().unwrap();
    assert_eq!(tag.value, "tag:example:%FFsuffix");
}

#[test_case("!é x")]
#[test_case("!<tag:a|b> x")]
#[test_case("!<!> x")]
#[test_case("!<$:?> x")]
#[test_case("!<relative> x")]
#[test_case("%TAG !e! relative\n--- !e!path x")]
fn invalid_tag_uris_are_rejected(source: &str) {
    assert!(parse(source).is_err(), "{source}");
}

// https://yaml.org/spec/1.2.2/#691-node-tags
// https://datatracker.ietf.org/doc/html/rfc3986#section-3
#[test_case("tag:example,a:b", true)]
#[test_case("https://user:pw@host:123/a?one?two#part?", true)]
#[test_case("file://", true)]
#[test_case("https://host:", true)]
#[test_case("https://[2001:db8::1]:443", true)]
#[test_case("https://[::ffff:192.0.2.1]", true)]
#[test_case("x://[Va.next:host+name]", true)]
#[test_case("https://host/%5B%23%5D", true)]
#[test_case("!local[brackets]#one#two", true)]
#[test_case("https://[not-an-ip]", false)]
#[test_case("https://[::1", false)]
#[test_case("https://[::1]extra", false)]
#[test_case("https://user[name]@host", false)]
#[test_case("https://user@other@host", false)]
#[test_case("https://ho]st", false)]
#[test_case("https://host:port", false)]
#[test_case("https://[::1]:port", false)]
#[test_case("x://[v.name]", false)]
#[test_case("x://[vG.name]", false)]
#[test_case("x://[v1.]", false)]
#[test_case("x://[v1.%20]", false)]
#[test_case("https://host/a[b]", false)]
#[test_case("tag:a[b]", false)]
#[test_case("tag:a?q[x]", false)]
#[test_case("tag:a#x#y", false)]
fn verbatim_and_expanded_tags_obey_uri_syntax(prefix: &str, accepted: bool) {
    let verbatim = format!("!<{prefix}/x> value");
    let shorthand = format!("%TAG !e! {prefix}\n--- !e!/x value");
    for source in [verbatim, shorthand] {
        assert_eq!(parse(&source).is_ok(), accepted, "{source}");
    }
}

// https://yaml.org/spec/1.2.2/#823-block-nodes
#[test_case("!!seq\n&a [*a]")]
#[test_case("&a\n!!seq [*a]")]
#[test_case("&a\n- *a")]
#[test_case("!!seq\n&a\n- *a")]
fn multiline_properties_preserve_anchor_identity(source: &str) {
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

// https://yaml.org/spec/1.2.2/#72-empty-nodes
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

// https://yaml.org/spec/1.2.2/#61-indentation-spaces
// https://yaml.org/spec/1.2.2/#63-line-prefixes
// https://yaml.org/spec/1.2.2/#64-empty-lines
#[test_case("a: plain\n\tcontinued")]
#[test_case("a: plain\n\t\n  continued")]
#[test_case("a: 'first\n\t\n  second'")]
#[test_case("outer:\n  a: \"first\\\n \t\n    second\"")]
#[test_case("a: \"first\\\nsecond\"")]
#[test_case("\"first\\\n---\nsecond\"")]
fn tab_indented_continuations_are_rejected(source: &str) {
    assert!(parse(source).is_err(), "{source:?}");
}

#[test_case("a: plain\n \tcontinued")]
#[test_case("a: \"first\\\n \tsecond\"")]
#[test_case("outer:\n  a: \"first\\\n   \t\n    second\"")]
fn tabs_after_continuation_indentation_are_content(source: &str) {
    assert!(parse(source).is_ok(), "{source:?}");
}

// https://yaml.org/spec/1.2.2/#822-block-mappings
#[test_case("\"a\" extra: b")]
#[test_case("[a] extra: b")]
#[test_case("{a: b} extra: c")]
fn implicit_key_must_end_at_the_mapping_separator(source: &str) {
    assert_eq!(
        parse(source).unwrap_err().message,
        "expected ':' after mapping key"
    );
}

// https://yaml.org/spec/1.2.2/#733-plain-style
// https://yaml.org/spec/1.2.2/#822-block-mappings
#[test]
fn collection_keys_allow_quotes_inside_plain_scalars() {
    for source in [
        "[flo']: block",
        "[f\"ow]: block",
        "[http:\"foo]: block",
        "[{firs\"t: Sammy}]: block",
        "[\"a]b\", 'c''d']: block",
        "[!local 'a]b']: block",
        "[{\"key\":'a]b'}]: block",
    ] {
        let stream = parse(source).unwrap_or_else(|error| panic!("{source:?}: {error}"));
        let root = stream.node(stream.documents()[0].root).unwrap();
        let NodeKind::Mapping { entries, .. } = &root.kind else {
            panic!("expected a mapping: {source:?}");
        };
        assert_eq!(entries.len(), 1, "{source:?}");
        assert!(matches!(
            stream.node(entries[0].key).unwrap().kind,
            NodeKind::Sequence { .. }
        ));
        let NodeKind::Scalar(value) = &stream.node(entries[0].value).unwrap().kind else {
            panic!("expected a scalar value: {source:?}");
        };
        assert_eq!(value.value, "block", "{source:?}");
    }

    let stream = parse("[flo']: block").unwrap();
    insta::assert_snapshot!(astral_yaml_dev::events(&stream), @"
    +STR
    +DOC
    +MAP
    +SEQ []
    =VAL :flo'
    -SEQ
    =VAL :block
    -MAP
    -DOC
    -STR
    ");
}

// https://yaml.org/spec/1.2.2/#61-indentation-spaces
// https://yaml.org/spec/1.2.2/#822-block-mappings
#[test_case("a: ?")]
#[test_case("a: ? # comment\n")]
#[test_case("- a: ?\n")]
#[test_case("foo:\n  a: 1\n \tb: 2\n")]
#[test_case("foo:\n  a: 1\n\t b: 2\n")]
#[test_case("foo:\n - a: 1\n  \tb: 2\n")]
#[test_case("foo:\n  - a\n \t- b\n")]
fn block_collections_reject_invalid_indentation_and_compact_values(source: &str) {
    assert!(parse(source).is_err(), "{source:?}");
}

#[test_case("a: ?foo")]
#[test_case("a: plain\n\t\nb: next")]
#[test_case("a: '?'")]
#[test_case("a:\n  ?\n")]
#[test_case("? a\n: ?\n")]
#[test_case("[\n\ta,\n\tb\n]")]
#[test_case("foo:\n  a: 1\n\t\n  b: 2\n")]
fn collection_indicators_and_blank_line_tabs_are_accepted(source: &str) {
    assert!(parse(source).is_ok(), "{source:?}");
}

#[test_case(0, true; "at the limit")]
#[test_case(1, false; "one byte over")]
fn expanded_tag_budget_spans_documents(shortfall: usize, accepted: bool) {
    let prefix = format!("tag:example.org,2000:{}", "a".repeat(64));
    let local = "!local";
    let document = format!("%TAG !t! {prefix}\n---\n- !t!x v\n- !t!x v\n- {local} v\n...\n");
    let source = document.repeat(2);
    let document_tag_bytes = 2 * (prefix.len() + 1) + local.len();
    let options = ParseOptions {
        max_input_bytes: 2 * document_tag_bytes - shortfall,
        ..ParseOptions::default()
    };
    let result = parse_with_options(&source, options);
    assert_eq!(
        result.err().map(|error| error.message),
        (!accepted).then_some("expanded tag byte limit exceeded")
    );
}

// https://yaml.org/spec/1.2.2/#682-tag-directives
#[test]
fn tag_handle_lookup_in_a_large_directive_list() {
    use std::fmt::Write;

    let mut directives = String::new();
    for index in 0..16_384 {
        writeln!(directives, "%TAG !h{index}! !prefix{index}/").unwrap();
    }
    let source = format!("{directives}--- !h12345!suffix value");
    let stream = parse(&source).unwrap();
    assert_eq!(
        stream.nodes()[0].tag.as_ref().unwrap().value,
        "!prefix12345/suffix"
    );

    let duplicate = format!("{directives}%TAG !h12345! !other/\n--- value");
    assert_eq!(
        parse(&duplicate).unwrap_err().message,
        "duplicate TAG directive"
    );
}

// https://yaml.org/spec/1.2.2/#6822-tag-prefixes
#[test_case(",tag:example")]
#[test_case("[tag:example")]
#[test_case("]tag:example")]
fn tag_prefixes_reject_leading_flow_indicators(prefix: &str) {
    let source = format!("%TAG !e! {prefix}\n---\nvalue\n");
    assert_eq!(
        parse(&source).unwrap_err().message,
        "invalid global tag prefix"
    );
}

#[test_case("!")]
#[test_case("![]")]
#[test_case("tag:example,[prefix]")]
#[test_case("%74ag:example,")]
fn tag_prefixes_accept_local_and_global_forms(prefix: &str) {
    let source = format!("%TAG !e! {prefix}\n---\nvalue\n");
    parse(&source).unwrap();
}

// https://yaml.org/spec/1.2.2/#911-document-prefix
#[test_case("\u{feff}\n\u{feff}\n\u{feff}\n", &[])]
#[test_case("\u{feff}\u{feff}--- value", &["value"])]
#[test_case("\u{feff}\n# before\n\u{feff}\n\u{feff}value", &["value"])]
#[test_case("one\n\u{feff}\n\u{feff}--- two", &["one", "two"])]
#[test_case("one\n...\n\u{feff}\n\u{feff}two", &["one", "two"])]
fn document_prefixes_do_not_create_nodes(source: &str, expected: &[&str]) {
    let stream = parse(source).unwrap();
    let roots: Vec<_> = stream
        .documents()
        .iter()
        .map(|doc| {
            let NodeKind::Scalar(scalar) = &stream.node(doc.root).unwrap().kind else {
                panic!("expected scalar");
            };
            scalar.value.as_ref()
        })
        .collect();
    assert_eq!(roots, expected);
    let marks: Vec<_> = stream
        .byte_order_marks()
        .iter()
        .map(|span| span.start())
        .collect();
    assert_eq!(
        marks,
        source
            .match_indices('\u{feff}')
            .map(|(at, _)| at)
            .collect::<Vec<_>>()
    );
}

// https://yaml.org/spec/1.2.2/#92-streams
#[test_case("\n"; "LF")]
#[test_case("\r"; "CR")]
#[test_case("\r\n"; "CRLF")]
fn comment_ownership_retains_indentation_in_later_bare_documents(newline: &str) {
    use astral_yaml::comments::{CommentIndex, CommentPlacement};

    let source = [
        "first",
        "... # first end",
        "  a: one",
        "  # next value",
        "  b: two",
        "... # second end",
        "... # suffix",
        "--- third",
    ]
    .join(newline);
    let stream = parse(&source).unwrap();
    let doc = &stream.documents()[1];
    let NodeKind::Mapping { entries, .. } = &stream.node(doc.root).unwrap().kind else {
        panic!("expected mapping");
    };
    let index = CommentIndex::new(&stream, 1).unwrap();
    let mut comments: Vec<_> = index.direct().collect();
    comments.sort_by_key(|(_, comment)| comment.comment.span.start());
    let attachments: Vec<_> = comments
        .iter()
        .map(|(owner, comment)| (*owner, comment.comment.text, comment.placement))
        .collect();
    assert_eq!(
        attachments,
        [
            (entries[1].value, " next value", CommentPlacement::Leading),
            (doc.root, " second end", CommentPlacement::Inline),
            (doc.root, " suffix", CommentPlacement::Inline),
        ]
    );
}

// https://yaml.org/spec/1.2.2/#92-streams
#[test]
fn standalone_document_end_markers_retain_spans_and_use_the_budget() {
    let source = "...\n---\nvalue\n...\n... # suffix\n";
    let stream = parse(source).unwrap();
    assert_eq!(stream.documents().len(), 1);

    let markers = stream
        .document_end_markers()
        .iter()
        .enumerate()
        .map(|(index, &span)| (format!("end marker {}", index + 1), span));
    insta::assert_snapshot!(span_snapshot(source, markers), @"
      |
    1 | ...
      | ^^^ end marker 1
    2 | ---
    3 | value
    4 | ...
      | ^^^ end marker 2
    5 | ... # suffix
      | ^^^ end marker 3
    ");
    assert_eq!(
        stream.documents()[0].end_marker,
        Some(stream.document_end_markers()[1])
    );

    for source in ["...\n...", "\u{feff}\n\u{feff}"] {
        let options = ParseOptions {
            max_nodes: 1,
            ..ParseOptions::default()
        };

        assert_eq!(
            parse_with_options(source, options).unwrap_err().message,
            "syntax element limit exceeded"
        );
    }
}

// https://yaml.org/spec/1.2.2/#742-flow-mappings
#[test]
fn flow_values_require_separation_after_yaml_keys() {
    for source in ["{a:[]}", "[a:[]]", "{:[]}", "[:[]]", "[&a x, {*a :value}]"] {
        assert!(parse(source).is_err(), "{source}");
    }

    for source in [
        "{:foo}",
        "{? :foo}",
        "[:foo]",
        "{: foo}",
        "{a: [b]}",
        "{'a':[b]}",
        "{[]:value}",
    ] {
        parse(source).unwrap();
    }

    let stream = parse("{:foo}").unwrap();
    let scalars: Vec<_> = stream
        .nodes()
        .iter()
        .filter_map(|node| {
            if let NodeKind::Scalar(s) = &node.kind {
                Some(s.value.as_ref())
            } else {
                None
            }
        })
        .collect();
    assert_eq!(scalars, [":foo", ""]);
}

#[test]
fn large_flow_collections_and_line_backtracking() {
    for item in ["0,", "\"\\n\","] {
        let source = format!("[{}]", item.repeat(30_000));
        let stream = parse(&source).unwrap();
        assert_eq!(stream.nodes().len(), 30_001);
    }

    for (source, expected) in [
        ("a#".repeat(30_000), "a#".repeat(30_000)),
        (format!("\"{}\"", "\\n".repeat(30_000)), "\n".repeat(30_000)),
    ] {
        let stream = parse(&source).unwrap();
        let NodeKind::Scalar(scalar) = &stream.nodes()[0].kind else {
            panic!("expected scalar");
        };

        assert_eq!(scalar.value, expected);
    }

    for ending in ["\n", "\r", "\r\n"] {
        let source = format!(
            "a: >{ending}  one{ending}  two{ending}b: [x,{ending}    y]{ending}c: plain{ending}  continued{ending}"
        );
        let stream = parse(&source).unwrap();
        assert_eq!(stream.documents().len(), 1);

        for node in stream.nodes() {
            assert!(node.span.slice(&source).is_some());
        }
    }
}

// https://yaml.org/spec/1.2.2/#8111-block-indentation-indicator
// https://yaml.org/spec/1.2.2/#813-folded-style
#[test_case("|", "\n")]
#[test_case(">", " ")]
fn large_block_scalars_with_leading_empty_lines(style: &str, separator: &str) {
    let blank_lines = "\n".repeat(30_000);
    let content = "  text\n".repeat(30_000);

    let source = format!("{style}\n{blank_lines}{content}");
    let stream = parse(&source).unwrap();
    let NodeKind::Scalar(scalar) = &stream.nodes()[0].kind else {
        panic!("expected scalar");
    };

    let expected = format!(
        "{blank_lines}{}text\n",
        format!("text{separator}").repeat(29_999)
    );
    assert_eq!(scalar.value, expected);
    assert_eq!(scalar.span.slice(&source), Some(source.as_str()));
}
