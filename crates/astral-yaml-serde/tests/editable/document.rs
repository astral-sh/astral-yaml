use test_case::test_case;

use astral_yaml::ParseOptions;
use astral_yaml_dev::{edit_diff, indoc, span_snapshot};
use astral_yaml_serde::{Editable, Error, Spanned, Tagged};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    timeout: Spanned<u8>,
}

#[test]
fn typed_edits_preserve_unknown_fields_and_refresh_metadata() {
    let source = indoc! {r"
        timeout: 30 # seconds
        unknown : ['keep',  100]
    "};
    let mut config: Editable<Config> = source.parse().unwrap();
    let before = config.to_yaml();
    config.set_at(["timeout"], 100).unwrap();
    let timeout = &config.value().timeout;
    let field = (format!("timeout = {}", timeout.value), timeout.span);
    insta::assert_snapshot!(span_snapshot(config.syntax().source(), [field]), @"
      |
    1 | timeout: 100 # seconds
      |          ^^^ timeout = 100
    2 | unknown : ['keep',  100]
      |
    ");
    insta::assert_snapshot!(edit_diff(&before, config.syntax().source()), @"
    @@ -1,2 +1,2 @@
    -timeout: 30 # seconds
    +timeout: 100 # seconds
     unknown : ['keep',  100]
    ");
    assert_eq!(config.into_inner().timeout.value, 100);
}

#[test_case(0; "integer overflow")]
#[test_case(1; "duplicate field")]
#[test_case(2; "missing field")]
#[test_case(3; "sequence type mismatch")]
#[test_case(4; "mapping type mismatch")]
fn type_and_duplicate_key_failures_roll_back_both_views(operation: usize) {
    let source = indoc! {r"
        timeout: 30 # seconds
    "};
    let mut config = Editable::<Config>::parse(source).unwrap();
    let result = config.edit(|root| {
        let mut fields = root.into_mapping()?;
        match operation {
            0 => fields.field("timeout")?.set(256),
            1 => {
                fields.set("other", 60)?;
                fields.key_at(1)?.set("timeout")
            }
            2 => fields.remove("timeout").map(|_| ()),
            3 => fields.field("timeout")?.set([60]),
            _ => fields
                .field("timeout")?
                .set(serde_json::json!({"value": 60})),
        }
    });
    assert!(result.is_err());
    assert_eq!(config.value().timeout.value, 30);
    assert_eq!(config.syntax().source(), source);
}

#[test_case("["; "invalid syntax")]
#[test_case(
    indoc! {r"
        ---
        1
        ---
        2"};
    "multiple documents"
)]
#[test_case("timeout: bad"; "type mismatch")]
fn typed_parse_rejects_invalid_documents(source: &str) {
    assert!(source.parse::<Editable<Config>>().is_err());
}

#[test]
fn tags_and_nulls_use_the_existing_deserialization_policy() {
    let mut value = Editable::<Tagged<Option<u8>>>::parse("!!null null").unwrap();
    assert_eq!(value.value().value, None);
    let before = value.to_yaml();
    value
        .edit(|mut root| {
            root.set(Tagged {
                tag: "tag:yaml.org,2002:int".to_owned(),
                value: 42,
            })
        })
        .unwrap();
    assert_eq!(value.value().value, Some(42));
    assert_eq!(value.value().tag, "tag:yaml.org,2002:int");
    insta::assert_snapshot!(edit_diff(&before, value.syntax().source()), @r"
    @@ -1 +1 @@
    -!!null null
    \ No newline at end of file
    +!<tag:yaml.org,2002:int> 42
    \ No newline at end of file
    ");
    assert!(value.edit(|mut root| root.set(42)).is_err());
    assert_eq!(value.value().value, Some(42));
}

#[test]
fn typed_metadata_and_errors_use_the_lf_source() {
    let source = indoc! {r"
        # café
        timeout: 30 # seconds
    "};
    let mut config = Editable::<Config>::parse(&source.replace('\n', "\r\n")).unwrap();
    let expected: Config = astral_yaml_serde::from_str(source).unwrap();
    assert_eq!(config.value().timeout, expected.timeout);

    config.edit(|root| root.field("timeout")?.set(100)).unwrap();
    let expected: Config = astral_yaml_serde::from_str(config.source()).unwrap();
    assert_eq!(config.value().timeout, expected.timeout);
    insta::assert_snapshot!(edit_diff(source, config.source()), @"
    @@ -1,2 +1,2 @@
     # café
    -timeout: 30 # seconds
    +timeout: 100 # seconds
    ");

    let before = config.to_yaml();
    assert!(
        config
            .edit(|root| root.field("timeout")?.set("invalid"))
            .is_err()
    );
    assert_eq!(config.source(), before);
    assert_eq!(config.value().timeout, expected.timeout);

    let invalid = source.replace("30", "invalid");
    let error = Editable::<Config>::parse(&invalid.replace('\n', "\r\n")).unwrap_err();
    assert_eq!(
        error,
        astral_yaml_serde::from_str::<Config>(&invalid).unwrap_err()
    );
}

#[test]
fn typed_edits_preserve_carriage_returns_in_scalar_values() {
    let source = indoc! {r#"
        "before\r\nafter"
    "#};
    let mut document = Editable::<String>::parse(&source.replace('\n', "\r")).unwrap();
    assert_eq!(document.value(), "before\r\nafter");
    document.edit(|mut root| root.set("new\r\nvalue")).unwrap();
    assert_eq!(document.value(), "new\r\nvalue");
    insta::assert_snapshot!(edit_diff(source, document.source()), @r#"
    @@ -1 +1 @@
    -"before\r\nafter"
    +"new\r\nvalue"
    "#);
}

#[derive(Debug, PartialEq)]
enum ApplicationError {
    Cancelled(String),
    Yaml(Error),
}

impl From<Error> for ApplicationError {
    fn from(error: Error) -> Self {
        Self::Yaml(error)
    }
}

#[test]
fn application_errors_preserve_their_payload_and_roll_back_both_views() {
    let source = "timeout: 30 # seconds\n";
    let mut config = Editable::<Config>::parse(source).unwrap();
    let result: Result<(), ApplicationError> = config.try_edit(|root| {
        root.field("timeout")?.set(60)?;
        Err(ApplicationError::Cancelled(
            "reload was cancelled".to_owned(),
        ))
    });

    assert_eq!(
        result.unwrap_err(),
        ApplicationError::Cancelled("reload was cancelled".to_owned())
    );
    assert_eq!(config.source(), source);
    assert_eq!(config.value().timeout.value, 30);

    let result: Result<_, ApplicationError> = config.try_edit(|root| {
        root.field("timeout")?.set(60)?;
        Ok("reloaded".to_owned())
    });
    assert_eq!(result.unwrap(), "reloaded");
    assert_eq!(config.value().timeout.value, 60);
}

#[test_case(false; "propagated mutation error")]
#[test_case(true; "handled mutation error")]
fn application_errors_wrap_failed_mutations(handled: bool) {
    let mut config = Editable::<Config>::parse("timeout: 30\n").unwrap();
    let result: Result<(), ApplicationError> = config.try_edit(|root| {
        let error = root.field("timeout")?.remove_comment(0);
        if handled {
            assert!(error.is_err());
        } else {
            error?;
        }
        Ok(())
    });

    let ApplicationError::Yaml(error) = result.unwrap_err() else {
        panic!("expected a YAML error");
    };
    assert_eq!(error.message, "comment index out of bounds");
    assert_eq!(config.source(), "timeout: 30\n");
    assert_eq!(config.value().timeout.value, 30);
}

#[test_case(false; "syntax commit failure")]
#[test_case(true; "typed commit failure")]
fn application_errors_wrap_commit_failures(typed: bool) {
    let source = "[&port 80, *port]\n";
    let mut ports = Editable::<Vec<u16>>::parse(source).unwrap();
    let result: Result<(), ApplicationError> = ports.try_edit(|root| {
        if typed {
            root.item(0)?.set("http")?;
        } else {
            root.into_sequence()?.remove(0)?;
        }
        Ok(())
    });

    assert!(matches!(result, Err(ApplicationError::Yaml(_))));
    assert_eq!(ports.source(), source);
    assert_eq!(ports.value(), &[80, 80]);
}

#[test_case(0, 100, 100; "invalid depth")]
#[test_case(64, 1, 100; "node limit")]
#[test_case(64, 100, 1; "input limit")]
fn typed_parse_uses_explicit_limits(max_depth: usize, max_nodes: usize, max_input_bytes: usize) {
    let result = Editable::<Config>::parse_with_options(
        "timeout: 30\n",
        ParseOptions {
            max_depth,
            max_nodes,
            max_input_bytes,
        },
    );
    assert!(result.is_err());
}

#[test]
fn typed_edit_enforces_stored_limits_and_preserves_both_views() {
    let source = "timeout: 30\n";
    let mut config = Editable::<Config>::parse_with_options(
        source,
        ParseOptions {
            max_input_bytes: source.len(),
            ..ParseOptions::default()
        },
    )
    .unwrap();
    let result = config.edit(|root| root.field("timeout")?.set(100));
    assert!(result.is_err());
    assert_eq!(config.source(), source);
    assert_eq!(config.value().timeout.value, 30);
}

#[derive(Debug)]
struct Unvisited;

impl<'de> Deserialize<'de> for Unvisited {
    fn deserialize<D: serde::Deserializer<'de>>(_deserializer: D) -> Result<Self, D::Error> {
        Ok(Self)
    }
}

#[test]
fn explicit_limits_apply_to_initial_and_committed_typed_parsing() {
    let depth = ParseOptions::default().max_depth + 1;
    let source = format!("{}0{}", "[".repeat(depth), "]".repeat(depth));
    assert!(Editable::<Unvisited>::parse(&source).is_err());

    // This custom deserializer leaves children unvisited, so only the syntax
    // parser traverses the deeply nested sequence.
    let mut document = Editable::<Unvisited>::parse_with_options(
        &source,
        ParseOptions {
            max_depth: 128,
            ..ParseOptions::default()
        },
    )
    .unwrap();
    document
        .edit(|root| root.into_sequence()?.push(1).map(|_| ()))
        .unwrap();
    assert_eq!(document.syntax().options().max_depth, 128);
    assert_eq!(
        document.source(),
        format!("{}, 1]", &source[..source.len() - 1])
    );

    let options = document.syntax().options();
    let error = Editable::<serde::de::IgnoredAny>::parse_with_options(document.source(), options)
        .unwrap_err();
    assert!(error.message.contains("limit"));
}

// https://yaml.org/spec/1.2.2/#52-character-encodings
#[test_case("old\n", false; "root scalar")]
#[test_case("old: 1\n", true; "first mapping key")]
fn edits_at_the_stream_start_produce_readable_utf8(source: &str, mapping: bool) {
    let mut document = astral_yaml_serde::EditableYaml::parse(source).unwrap();
    document
        .edit(|mut root| {
            if mapping {
                root.into_mapping()?.key_at(0)?.set("éclair")
            } else {
                root.set("éclair")
            }
        })
        .unwrap();

    for yaml in [
        document.to_yaml(),
        document.syntax().normalized_yaml().unwrap(),
    ] {
        astral_yaml::Source::from_bytes(yaml.as_bytes()).unwrap();
    }
}

#[test_case("é\r\n", "\u{feff}é\n"; "decoded text with CRLF")]
#[test_case("\u{feff}é\n", "\u{feff}é\n"; "existing byte order mark")]
#[test_case("---\né\n", "---\né\n"; "ASCII prefix")]
fn editable_source_has_a_utf8_stream_prefix(source: &str, expected: &str) {
    let document = Editable::<String>::parse(source).unwrap();
    assert_eq!(document.into_yaml(), expected);
}

#[test]
fn a_required_utf8_mark_counts_toward_the_edit_budget() {
    let mut document = Editable::<String>::parse_with_options(
        "old\n",
        ParseOptions {
            max_input_bytes: 5,
            ..ParseOptions::default()
        },
    )
    .unwrap();
    let error = document.edit(|mut root| root.set("é")).unwrap_err();
    assert!(error.message.contains("limit"));
    assert_eq!(document.source(), "old\n");
}
