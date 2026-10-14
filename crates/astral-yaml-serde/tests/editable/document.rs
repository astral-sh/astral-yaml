use test_case::test_case;

use astral_yaml::ParseOptions;
use astral_yaml_dev::{edit_diff, indoc};
use astral_yaml_serde::{Editable, Error, Spanned, Tagged};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    timeout: Spanned<u8>,
}

#[test]
fn typed_edits_preserve_unknown_fields_and_refresh_metadata() {
    let mut config = {
        let source = String::from(indoc! {r"
            timeout: 30 # seconds
            unknown : ['keep',  2]
        "});
        Editable::<Config>::parse(&source).unwrap()
    };
    let before = config.to_yaml();
    config
        .edit_syntax(|e| {
            let entries = e.mapping_entries(&e.root())?;
            e.replace(&entries[0].1, "100")
        })
        .unwrap();
    assert_eq!(config.value().timeout.value, 100);
    assert_eq!(
        config.value().timeout.span.slice(config.syntax().source()),
        Some("100")
    );
    insta::assert_snapshot!(edit_diff(&before, config.syntax().source()), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
    -timeout: 30 # seconds
    +timeout: 100 # seconds
     unknown : ['keep',  2]
    ");
    assert_eq!(config.into_inner().timeout.value, 100);
}

#[test_case(0; "integer overflow")]
#[test_case(1; "invalid syntax")]
#[test_case(2; "duplicate field")]
#[test_case(3; "missing field")]
#[test_case(4; "sequence type mismatch")]
#[test_case(5; "mapping type mismatch")]
fn syntax_type_and_duplicate_key_failures_roll_back_both_views(operation: usize) {
    let source = indoc! {r"
        timeout: 30 # seconds
    "};
    let mut config = Editable::<Config>::parse(source).unwrap();
    let result = config.edit_syntax(|e| {
        let root = e.root();
        let entries = e.mapping_entries(&root)?;
        match operation {
            0 => e.replace(&entries[0].1, "256"),
            1 => e.replace(&entries[0].1, "["),
            2 => e.insert_entry(&root, 1, "timeout", "60").map(|_| ()),
            3 => e.remove_entry(&root, 0),
            4 => e.replace(&entries[0].1, "[60]"),
            _ => e.replace(&entries[0].1, "{value: 60}"),
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
    assert!(Editable::<Config>::parse(source).is_err());
}

#[test]
fn tags_and_nulls_use_the_existing_deserialization_policy() {
    let mut value = Editable::<Tagged<Option<u8>>>::parse("!!null null").unwrap();
    assert_eq!(value.value().value, None);
    let before = value.to_yaml();
    value
        .edit_syntax(|e| e.replace(&e.root(), "!!int 42"))
        .unwrap();
    assert_eq!(value.value().value, Some(42));
    assert_eq!(value.value().tag, "tag:yaml.org,2002:int");
    insta::assert_snapshot!(edit_diff(&before, value.syntax().source()), @r"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1 @@
    -!!null null
    \ No newline at end of file
    +!<tag:yaml.org,2002:int> 42
    \ No newline at end of file
    ");
    assert!(value.edit_syntax(|e| e.replace(&e.root(), "42")).is_err());
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

    config
        .edit(|editor| editor.root_mut().field("timeout")?.set(100))
        .unwrap();
    let expected: Config = astral_yaml_serde::from_str(config.source()).unwrap();
    assert_eq!(config.value().timeout, expected.timeout);
    insta::assert_snapshot!(edit_diff(source, config.source()), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
     # café
    -timeout: 30 # seconds
    +timeout: 100 # seconds
    ");

    let before = config.to_yaml();
    assert!(
        config
            .edit(|editor| editor.root_mut().field("timeout")?.set("invalid"))
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
    document
        .edit(|editor| editor.root_mut().set("new\r\nvalue"))
        .unwrap();
    assert_eq!(document.value(), "new\r\nvalue");
    insta::assert_snapshot!(edit_diff(source, document.source()), @r#"
    --- before.yaml
    +++ after.yaml
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
    let result: Result<(), ApplicationError> = config.try_edit(|editor| {
        editor.root_mut().field("timeout")?.set(60)?;
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

    let result: Result<_, ApplicationError> = config.try_edit(|editor| {
        editor.root_mut().field("timeout")?.set(60)?;
        Ok("reloaded".to_owned())
    });
    assert_eq!(result.unwrap(), "reloaded");
    assert_eq!(config.value().timeout.value, 60);
}

#[test_case(false; "propagated mutation error")]
#[test_case(true; "handled mutation error")]
fn application_errors_wrap_failed_mutations(handled: bool) {
    let mut config = Editable::<Config>::parse("timeout: 30\n").unwrap();
    let result: Result<(), ApplicationError> = config.try_edit(|editor| {
        let error = editor.root_mut().field("timeout")?.set_yaml("[");
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
    assert!(error.message.contains("flow sequence"));
    assert_eq!(config.source(), "timeout: 30\n");
    assert_eq!(config.value().timeout.value, 30);
}

#[test_case(false; "syntax commit failure")]
#[test_case(true; "typed commit failure")]
fn application_errors_wrap_commit_failures(typed: bool) {
    let source = "[&port 80, *port]\n";
    let mut ports = Editable::<Vec<u16>>::parse(source).unwrap();
    let result: Result<(), ApplicationError> = ports.try_edit(|editor| {
        if typed {
            editor.root_mut().item(0)?.set("http")?;
        } else {
            editor.root_mut().into_sequence()?.remove(0)?;
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
    let result = config.edit(|editor| editor.root_mut().field("timeout")?.set(100));
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
        .edit(|editor| editor.root_mut().into_sequence()?.push(1).map(|_| ()))
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
