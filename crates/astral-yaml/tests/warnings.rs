use astral_yaml::{WarningKind, parse};
use test_case::test_case;

// https://yaml.org/spec/1.2.2/#68-directives
// https://yaml.org/spec/1.2.2/#54-line-break-characters
#[test]
fn warnings_follow_source_order_across_documents() {
    let source = concat!(
        "%FUTURE option # ignored\n",
        "%YAML 1.3\n",
        "--- current\u{85}document\n",
        "...\n",
        "%UNKNOWN \u{85}\n",
        "%YAML 01.001 # header\u{2028}comment\n",
        "--- \"a\u{2029}b\\N\"\n",
        "# trailing\u{85}comment\n",
        "--- next\u{85}document\n",
    );
    let stream = parse(source).unwrap();
    let warnings = stream
        .warnings()
        .map(|warning| {
            let text = warning.span.slice(source).unwrap();
            format!("{} {text:?} ({})", warning.span.start(), warning.kind)
        })
        .collect::<Vec<_>>()
        .join("\n");
    insta::assert_snapshot!(warnings, @r#"
    0 "%FUTURE option" (unknown directive ignored)
    25 "%YAML 1.3" (higher YAML minor version processed as YAML 1.2)
    61 "%UNKNOWN \u{85}" (unknown directive ignored)
    70 "\u{85}" (YAML 1.1 line break treated as a non-break character)
    94 "\u{2028}" (YAML 1.1 line break treated as a non-break character)
    111 "\u{2029}" (YAML 1.1 line break treated as a non-break character)
    129 "\u{85}" (YAML 1.1 line break treated as a non-break character)
    "#);
}

// https://yaml.org/spec/1.2.2/#681-yaml-directives
#[test_case("1.0", None)]
#[test_case("01.0002", None)]
#[test_case("1.10", Some(WarningKind::HigherMinorVersion))]
#[test_case(
    "1.999999999999999999999999999999",
    Some(WarningKind::HigherMinorVersion)
)]
fn minor_versions_are_compared_as_decimal_numbers(version: &str, expected: Option<WarningKind>) {
    let source = format!("%YAML {version}\n--- value");
    let stream = parse(&source).unwrap();
    assert!(stream.warnings().map(|warning| warning.kind).eq(expected));
}

// https://yaml.org/spec/1.2.2/#54-line-break-characters
#[test_case("%YAML 1.1\n--- \"\\N\\L\\P\""; "escaped line breaks")]
#[test_case("%TAG !e! !example/\n--- !e!tag text\u{2028}here"; "tag directive")]
#[test_case("%YAML 1.1\n--- text\n...\n# comment\u{85}outside\n"; "after explicit end")]
#[test_case("%YAML 1.1\n--- text\n\u{feff}# next\u{85}prefix\n--- value"; "after a byte order mark")]
fn compatible_input_has_no_warnings(source: &str) {
    assert_eq!(parse(source).unwrap().warnings().next(), None);
}
