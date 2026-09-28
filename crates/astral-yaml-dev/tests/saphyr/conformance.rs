use std::{collections::HashMap, fmt::Write, fs, path::PathBuf};

use astral_yaml_dev::saphyr::{astral_documents, options};
use serde_json::Value;
use serde_saphyr::granit_parser::{Event, Parser, ScalarStyle, StructureStyle};

fn corpus_paths() -> Vec<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/yaml-test-suite");
    let mut directories = vec![root];
    let mut inputs = Vec::new();

    while let Some(directory) = directories.pop() {
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();

            if path.is_dir() {
                directories.push(path);
            } else if path.file_name().is_some_and(|name| name == "in.yaml") {
                inputs.push(path);
            }
        }
    }

    inputs.sort();
    assert_eq!(inputs.len(), 402);

    inputs
}

#[test]
fn yaml_suite_acceptance() {
    let mut differences = Vec::new();

    for path in corpus_paths() {
        let source = fs::read_to_string(&path).unwrap();
        let valid = !path.with_file_name("error").exists();
        assert_eq!(
            astral_yaml::parse(&source).is_ok(),
            valid,
            "{}",
            path.display()
        );

        if !valid {
            let accepted = serde_saphyr::from_multiple::<serde::de::IgnoredAny>(&source).is_ok();
            assert_eq!(
                accepted,
                path.parent().unwrap().ends_with("9C9N"),
                "{}",
                path.display()
            );
        }

        let result = Parser::new_from_str(&source).collect::<Result<Vec<_>, _>>();
        if result.is_ok() != valid {
            differences.push(
                path.parent()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned(),
            );
        }
    }

    // serde-saphyr deliberately permits under-indented flow sequences.
    assert_eq!(differences, ["9C9N"]);
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
        .replace('\0', "\\0")
        .replace('\u{8}', "\\b")
}

fn granit_events(source: &str) -> Vec<String> {
    Parser::new_from_str(source)
        .filter_map(|result| {
            let (event, _) = result.unwrap();
            let mut properties = String::new();

            if let Some(anchor) = event.anchor_id().filter(|id| *id != 0) {
                write!(properties, " &{anchor}").unwrap();
            }

            if let Some(tag) = event.tag() {
                write!(properties, " <{}{}>", tag.handle(), tag.suffix()).unwrap();
            }

            Some(match event {
                Event::StreamStart => "+STR".into(),
                Event::StreamEnd => "-STR".into(),
                Event::DocumentStart(explicit, _) => {
                    if explicit { "+DOC ---" } else { "+DOC" }.into()
                }
                Event::DocumentEnd => "-DOC".into(),
                Event::SequenceStart(style, ..) => format!(
                    "+SEQ{}{properties}",
                    if style == StructureStyle::Flow {
                        " []"
                    } else {
                        ""
                    }
                ),
                Event::SequenceEnd => "-SEQ".into(),
                Event::MappingStart(style, ..) => format!(
                    "+MAP{}{properties}",
                    if style == StructureStyle::Flow {
                        " {}"
                    } else {
                        ""
                    }
                ),
                Event::MappingEnd => "-MAP".into(),
                Event::Alias(id) => format!("=ALI *{id}"),
                Event::Scalar(value, style, ..) => {
                    let style = match style {
                        ScalarStyle::Plain => ':',
                        ScalarStyle::SingleQuoted => '\'',
                        ScalarStyle::DoubleQuoted => '"',
                        ScalarStyle::Literal => '|',
                        ScalarStyle::Folded => '>',
                    };
                    format!("=VAL{properties} {style}{}", escape(&value))
                }
                Event::Comment(..) => return None,
                _ => panic!("unhandled parser event: {event:?}"),
            })
        })
        .collect()
}

fn normalized_events(expected: &str) -> Vec<String> {
    let mut anchors = HashMap::new();
    let mut next_anchor = 1;

    expected
        .lines()
        .map(|line| {
            let mut line = line.to_owned();

            if let Some(name) = line.strip_prefix("=ALI *") {
                return format!("=ALI *{}", anchors[name]);
            }

            if line.starts_with("=VAL &") || line.starts_with("+SEQ") || line.starts_with("+MAP") {
                if let Some(start) = line.find(" &") {
                    let end = line[start + 2..]
                        .find(' ')
                        .map_or(line.len(), |end| start + 2 + end);
                    anchors.insert(line[start + 2..end].to_owned(), next_anchor);
                    line.replace_range(start + 2..end, &next_anchor.to_string());
                    next_anchor += 1;
                }
            }

            if line == "-DOC ..." {
                line = "-DOC".into();
            }

            // Granit represents an implicit empty scalar with the null spelling `~`.
            if line.starts_with("=VAL") && line.ends_with(" :") && !line.contains('<') {
                line.push('~');
            }

            line
        })
        .collect()
}

#[test]
fn yaml_suite_events() {
    let mut differences = Vec::new();

    for path in corpus_paths() {
        if path.with_file_name("error").exists() {
            continue;
        }

        let source = fs::read_to_string(&path).unwrap();
        let expected = fs::read_to_string(path.with_file_name("test.event")).unwrap();
        let astral = astral_yaml::parse(&source).unwrap();
        assert_eq!(
            astral_yaml_dev::events(&astral),
            expected,
            "{}",
            path.display()
        );
        let actual = granit_events(&source);
        let expected = normalized_events(&expected);

        if actual != expected {
            differences.push((path, actual, expected));
        }
    }

    assert!(differences.is_empty(), "{differences:#?}");
}

#[test]
fn yaml_suite_json_values() {
    let mut count = 0;
    let mut rejected = Vec::new();

    for path in corpus_paths() {
        if path.with_file_name("error").exists() || !path.with_file_name("in.json").exists() {
            continue;
        }

        let source = fs::read_to_string(&path).unwrap();
        let json = fs::read_to_string(path.with_file_name("in.json")).unwrap();
        let expected = serde_json::Deserializer::from_str(&json)
            .into_iter::<Value>()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let astral = astral_documents::<Value>(&source);
        let saphyr = serde_saphyr::from_multiple_with_options::<Value>(&source, options());

        // serde-saphyr's multi-document API deliberately omits null documents.
        let non_null: Vec<_> = expected
            .iter()
            .filter(|value| !value.is_null())
            .cloned()
            .collect();

        if saphyr.is_err() {
            assert!(astral.is_err());
            rejected.push(
                path.parent()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned(),
            );
        } else {
            assert_eq!(
                astral.as_ref().ok(),
                Some(&expected),
                "{}: {astral:?}",
                path.display()
            );
            assert_eq!(
                saphyr.as_ref().ok(),
                Some(&non_null),
                "{}: {saphyr:?}",
                path.display()
            );
        }

        count += 1;
    }

    assert_eq!(count, 279);
    // JSON erases tags. Both libraries reject these non-core tags or binary values
    // under the comparison settings.
    assert_eq!(
        rejected,
        [
            "2XXW", "565N", "5TYM", "6CK3", "6WLZ", "7FWL", "9WXW", "C4HZ", "CC74", "CUP7", "J7PZ",
            "M5C3", "P76L", "UGM3", "Z67P", "Z9M4"
        ]
    );
}
