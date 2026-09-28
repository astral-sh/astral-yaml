//! Compare generated scalar syntax with `yaml_serde` without using our serializer.

fn check_block_presentations(scalar: &str, invalid_comment_indent: bool) {
    for source in [
        scalar.to_owned(),
        format!("value: {scalar}"),
        format!("- {scalar}"),
        format!("outer:\n  value: {}", scalar.replace('\n', "\n  ")),
    ] {
        let actual = astral_yaml_serde::from_str::<yaml_serde::Value>(&source);
        if invalid_comment_indent {
            assert!(actual.is_err(), "{source:?}");
            continue;
        }

        // Supply the suite's implicit EOF line break to LibYAML
        // (yaml/yaml-test-suite#118).
        let terminated = if source.ends_with('\n') {
            source.clone()
        } else {
            format!("{source}\n")
        };
        let reference = yaml_serde::from_str::<yaml_serde::Value>(&terminated);

        match (actual, reference) {
            (Ok(actual), Ok(expected)) => assert_eq!(actual, expected, "{source:?}"),
            (Err(_), Err(_)) => {}
            (actual, expected) => panic!("{source:?}: {actual:?} != {expected:?}"),
        }
    }
}

// https://yaml.org/spec/1.2.2/#81-block-scalar-styles
#[test]
fn block_scalar_combinations_match_reference() {
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
                                let source = format!(
                                    "{style}{indent}{chomp}\n{first}\n{second}\n{third}{ending}"
                                );
                                // LibYAML accepts an over-indented leading blank before
                                // a # content line, contrary to S98Z (yaml/yaml-test-suite#37).
                                let body = [first, second, third];
                                let first_content =
                                    body.iter().position(|line| !line.trim().is_empty());
                                let invalid_comment_indent = indent.is_empty()
                                    && first_content.is_some_and(|index| {
                                        body[index].trim_start().starts_with('#')
                                            && body[..index].iter().any(|line| line.len() > 2)
                                    });

                                check_block_presentations(&source, invalid_comment_indent);

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
fn quoted_scalar_folding_and_escapes_match_reference() {
    let mut cases = 0;

    for quote in ['\'', '"'] {
        let fragments = if quote == '\'' {
            vec!["", "word", "é", " ", "\t", "''", "#", ":", "\\"]
        } else {
            vec![
                "", "word", "é", " ", "\t", "\\\"", "\\ ", "\\t", "\\n", "\\\\",
            ]
        };

        for first in &fragments {
            for second in &fragments {
                for separator in [" ", "\n ", " \n  ", "\n\n ", "\r\n \r\n  ", "\n \t "] {
                    let scalar = format!("{quote}{first}{separator}{second}{quote}");

                    for source in [
                        scalar.clone(),
                        format!("[{scalar}]"),
                        format!("value: {scalar}"),
                    ] {
                        let actual = astral_yaml_serde::from_str::<yaml_serde::Value>(&source);
                        let expected = yaml_serde::from_str::<yaml_serde::Value>(&source);
                        assert_eq!(actual.unwrap(), expected.unwrap(), "{source:?}");
                        cases += 1;
                    }
                }
            }
        }
    }

    assert_eq!(cases, 3_258);
}
