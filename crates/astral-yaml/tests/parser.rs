use std::borrow::Cow;

use astral_yaml::{NodeKind, ParseOptions, ScalarStyle, Span, parse, parse_with_options};

// https://yaml.org/spec/1.2.2/#51-character-set
#[test]
fn ascii_character_checks_cover_chunk_boundaries() {
    for padding in [0, 7, 8, 15, 16, 31, 32, 63, 64, 65, 127] {
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
}

// https://yaml.org/spec/1.2.2/#73-flow-scalar-styles
#[test]
fn line_tracking_survives_scalar_lookahead() {
    let template = "\u{feff}first: one\nsecond:\n  - two\n  - three\nfolded: four\n  five\nlast: six\n---\n[seven,\n eight]\n";

    for newline in ["\n", "\r", "\r\n"] {
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
}

// https://yaml.org/spec/1.2.2/#54-line-break-characters
// https://yaml.org/spec/1.2.2/#73-flow-scalar-styles
#[test]
fn scalar_scans_preserve_content_and_spans_across_line_endings() {
    for newline in ["\n", "\r", "\r\n"] {
        for padding in [0, 15, 31, 64] {
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
    }
}

// https://yaml.org/spec/1.2.2/#57-escaped-characters
#[test]
fn json_surrogate_pairs_and_malformed_escapes() {
    for value in [0x10000, 0x1d11e, 0x1f30d, 0x0010_ffff] {
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

    for source in [
        r#""\uD800""#,
        r#""\uD800"#,
        r#""\uDC00""#,
        r#""\uDC00\uD800""#,
        r#""\uD800\uD800""#,
        r#""\uD800\u0041""#,
        r#""\uD800\uFFFF""#,
        r#""\uD800x""#,
        r#""\uD800\n""#,
        r#""\uD800\u""#,
        r#""\uD800\uZZZZ""#,
        r#""\uD800 \uDC00""#,
        r#""\uD800\U0000DC00""#,
        r#""\U0000D800\uDC00""#,
    ] {
        let error = parse(source).unwrap_err();
        assert!(error.span.slice(source).is_some(), "{source}");
    }
}

// https://yaml.org/spec/1.2.2/#8111-block-indentation-indicator
// https://yaml.org/spec/1.2.2/#8112-block-chomping-indicator
#[test]
fn block_scalar_eof_and_indentation_follow_the_test_suite() {
    // Follow the suite's EOF-as-line-break interpretation (yaml/yaml-test-suite#118).
    for style in ["|", ">"] {
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

    // A tab after the detected spaces is content (YAML 1.2.2 example 8.2).
    // LibYAML incorrectly rejects this when indentation is inferred.
    for (source, expected) in [
        ("|\n  \ttext\n", "\ttext\n"),
        (">\n  \t\n  text\n", "\t\ntext\n"),
    ] {
        let stream = parse(source).unwrap();
        let NodeKind::Scalar(scalar) = &stream.nodes()[0].kind else {
            panic!("expected scalar")
        };

        assert_eq!(scalar.value, expected);
    }
}

// https://yaml.org/spec/1.2.2/#51-character-set
// https://yaml.org/spec/1.2.2/#52-character-encodings
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
}

// https://yaml.org/spec/1.2.2/#742-flow-mappings
// https://yaml.org/spec/1.2.2/#822-block-mappings
#[test]
fn implicit_key_limits_include_separation_before_the_colon() {
    for key in ["key", "'key'", "[key]", "é", "'🌍'"] {
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

    for newline in ["\n", "\r", "\r\n"] {
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
#[test]
fn byte_order_marks_are_content_inside_quoted_continuations() {
    for (quote, escaped) in [('\'', false), ('"', false), ('"', true)] {
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
            assert!(stream.byte_order_marks().is_empty());
        }
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
#[test]
fn directive_parameters_and_comments_are_separate() {
    for (header, arguments, comment) in [
        ("%FOO", "", None),
        ("%FOO # comment", "", Some(" comment")),
        ("%FOO\t# comment", "", Some(" comment")),
        ("%FOO bar#hash # comment", "bar#hash", Some(" comment")),
        ("%FOO bar baz", "bar baz", None),
    ] {
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
}

// https://yaml.org/spec/1.2.2/#681-yaml-directives
#[test]
fn yaml_versions_have_decimal_components() {
    for (version, accepted) in [
        ("1.1", true),
        ("1.2", true),
        ("1.3", true),
        ("01.02", true),
        ("0001.0002", true),
        ("2.0", false),
        ("1", false),
        ("1.", false),
        (".2", false),
        ("+1.2", false),
        ("1.2.3", false),
        ("1.2 extra", false),
        ("1.２", false),
    ] {
        let source = format!("%YAML {version}\n--- value\n");
        assert_eq!(parse(&source).is_ok(), accepted, "{source:?}");
    }
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

// https://yaml.org/spec/1.2.2/#823-block-nodes
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

// https://yaml.org/spec/1.2.2/#822-block-mappings
#[test]
fn implicit_key_must_end_at_the_mapping_separator() {
    for source in ["\"a\" extra: b", "[a] extra: b", "{a: b} extra: c"] {
        assert_eq!(
            parse(source).unwrap_err().message,
            "expected ':' after mapping key"
        );
    }
}

// https://yaml.org/spec/1.2.2/#6822-tag-prefixes
#[test]
fn tag_prefixes_obey_the_first_character_rule() {
    for prefix in [",tag:example", "[tag:example", "]tag:example"] {
        let source = format!("%TAG !e! {prefix}\n---\nvalue\n");
        assert_eq!(
            parse(&source).unwrap_err().message,
            "invalid global tag prefix"
        );
    }

    for prefix in ["!", "![]", "tag:example,[prefix]", "%74ag:example,"] {
        let source = format!("%TAG !e! {prefix}\n---\nvalue\n");
        parse(&source).unwrap();
    }
}

// https://yaml.org/spec/1.2.2/#92-streams
#[test]
fn standalone_document_end_markers_retain_spans_and_use_the_budget() {
    let source = "...\n---\nvalue\n...\n... # suffix\n";
    let stream = parse(source).unwrap();
    assert_eq!(stream.documents().len(), 1);
    assert_eq!(
        stream
            .document_end_markers()
            .iter()
            .map(|span| span.slice(source))
            .collect::<Vec<_>>(),
        [Some("..."), Some("..."), Some("...")]
    );
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
#[test]
fn large_block_scalars_with_leading_empty_lines() {
    let blank_lines = "\n".repeat(30_000);
    let content = "  text\n".repeat(30_000);

    for (style, separator) in [("|", "\n"), (">", " ")] {
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
}
