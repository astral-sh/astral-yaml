use astral_yaml_serde::from_str;
use serde_json::{Value, json};

fn presentations(scalar: &str) -> [String; 4] {
    let nested = scalar.replace('\n', "\n  ");
    let nested = if scalar.ends_with('\n') {
        nested.strip_suffix("  ").unwrap()
    } else {
        &nested
    };

    [
        scalar.to_owned(),
        format!("value: {scalar}"),
        format!("- {scalar}"),
        format!("outer:\n  value: {nested}"),
    ]
}

fn assert_presentations(scalar: &str, expected: &str) {
    for (source, value) in presentations(scalar).into_iter().zip([
        json!(expected),
        json!({"value": expected}),
        json!([expected]),
        json!({"outer": {"value": expected}}),
    ]) {
        let actual =
            from_str::<Value>(&source).unwrap_or_else(|error| panic!("{source:?}: {error}"));
        assert_eq!(actual, value, "{source:?}");
    }
}

// https://yaml.org/spec/1.2.2/#81-block-scalar-styles
#[test]
fn block_scalar_styles_indentation_and_chomping() {
    for (body, literal, folded) in [
        ("  text\n", "text\n", "text\n"),
        ("  text\n  next\n", "text\nnext\n", "text next\n"),
        ("  text\n\n  next\n", "text\n\nnext\n", "text\nnext\n"),
        (
            "  text\n    more\n  next\n",
            "text\n  more\nnext\n",
            "text\n  more\nnext\n",
        ),
        ("  # content\n", "# content\n", "# content\n"),
        ("\n  text\n", "\ntext\n", "\ntext\n"),
        ("  text\n\n", "text\n\n", "text\n\n"),
    ] {
        for (style, kept) in [("|", literal), (">", folded)] {
            let content = kept.trim_end_matches('\n');

            for (chomp, expected) in [
                ("-", content.to_owned()),
                ("", format!("{content}\n")),
                ("+", kept.to_owned()),
            ] {
                for indent in ["", "2"] {
                    let scalar = format!("{style}{indent}{chomp}\n{body}");
                    assert_presentations(&scalar, &expected);
                }
            }
        }
    }
}

// https://yaml.org/spec/1.2.2/#81-block-scalar-styles
#[test]
fn generated_block_scalars_agree_across_presentations() {
    let lines = [
        "",
        " ",
        "  ",
        "   ",
        "  text",
        "    more",
        "  # content",
        "  foo: bar",
    ];
    let mut cases = 0;

    for style in ["|", ">"] {
        for chomp in ["", "+", "-"] {
            for indent in ["", "2"] {
                for first in lines {
                    for second in lines {
                        for third in lines {
                            for ending in ["", "\n", "\n\n"] {
                                let scalar = format!(
                                    "{style}{indent}{chomp}\n{first}\n{second}\n{third}{ending}"
                                );
                                let body = [first, second, third];
                                let first_content =
                                    body.iter().position(|line| !line.trim().is_empty());
                                let over_indented = indent.is_empty()
                                    && first_content.is_some_and(|index| {
                                        body[index].trim_start().starts_with('#')
                                            && body[..index].iter().any(|line| line.len() > 2)
                                    });

                                if over_indented {
                                    // The indented # is content, so earlier empty lines
                                    // cannot be more indented (test-suite case S98Z).
                                    for source in presentations(&scalar) {
                                        assert!(from_str::<Value>(&source).is_err(), "{source:?}");
                                    }
                                } else {
                                    match from_str::<String>(&scalar) {
                                        Ok(expected) => assert_presentations(&scalar, &expected),
                                        Err(_) => {
                                            for source in presentations(&scalar) {
                                                assert!(
                                                    from_str::<Value>(&source).is_err(),
                                                    "{source:?}"
                                                );
                                            }
                                        }
                                    }
                                }

                                cases += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    assert_eq!(cases * 4, 73_728);
}

// https://yaml.org/spec/1.2.2/#73-flow-scalar-styles
#[test]
fn quoted_scalar_folding_and_escapes() {
    let single = [
        ("", ""),
        ("word", "word"),
        ("é", "é"),
        (" ", " "),
        ("\t", "\t"),
        ("''", "'"),
        ("#", "#"),
        (":", ":"),
        ("\\", "\\"),
    ];
    let double = [
        ("", ""),
        ("word", "word"),
        ("é", "é"),
        (" ", " "),
        ("\t", "\t"),
        ("\\\"", "\""),
        ("\\ ", " "),
        ("\\t", "\t"),
        ("\\n", "\n"),
        ("\\\\", "\\"),
    ];

    for (quote, fragments) in [('\'', single.as_slice()), ('"', double.as_slice())] {
        for &(first, first_value) in fragments {
            for &(second, second_value) in fragments {
                for (separator, folded) in [
                    (" ", " "),
                    ("\n ", " "),
                    (" \n  ", " "),
                    ("\n\n ", "\n"),
                    ("\r\n \r\n  ", "\n"),
                    ("\n \t ", " "),
                ] {
                    let scalar = format!("{quote}{first}{separator}{second}{quote}");
                    let multiline = separator.contains('\n');
                    let first_value = if multiline && matches!(first, " " | "\t") {
                        ""
                    } else {
                        first_value
                    };
                    let second_value = if multiline && matches!(second, " " | "\t") {
                        ""
                    } else {
                        second_value
                    };
                    let expected = format!("{first_value}{folded}{second_value}");

                    assert_presentations(&scalar, &expected);
                }
            }
        }
    }
}

// https://yaml.org/spec/1.2.2/#1032-tag-resolution
#[test]
fn decimal_integers_can_have_leading_zeroes() {
    for (source, expected) in [
        ("0755", json!(755)),
        ("0012", json!(12)),
        ("-0012", json!(-12)),
        ("+0012", json!(12)),
        ("'0755'", json!("0755")),
    ] {
        assert_eq!(from_str::<Value>(source).unwrap(), expected, "{source}");
    }
}

// https://yaml.org/spec/1.2.2/#742-flow-mappings
#[test]
fn flow_keys_and_values_can_be_adjacent() {
    use std::collections::BTreeMap;

    for source in ["{a: [b]}", "{'a':[b]}"] {
        assert_eq!(
            from_str::<BTreeMap<String, Vec<String>>>(source).unwrap(),
            BTreeMap::from([("a".into(), vec!["b".into()])])
        );
    }

    assert_eq!(
        from_str::<BTreeMap<Vec<String>, String>>("{[]:value}").unwrap(),
        BTreeMap::from([(vec![], "value".into())])
    );
}
