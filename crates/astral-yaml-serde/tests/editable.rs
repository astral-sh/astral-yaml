use astral_yaml_dev::edit_diff;
use astral_yaml_serde::{Editable, Spanned, Tagged};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Config {
    timeout: Spanned<u8>,
}

#[test]
fn typed_edits_preserve_unknown_fields_and_refresh_metadata() {
    let mut config = {
        let source = String::from("timeout: 30 # seconds\nunknown : ['keep',  2]\n");
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
    assert_eq!(
        config.to_yaml(),
        "timeout: 100 # seconds\nunknown : ['keep',  2]\n"
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

#[test]
fn syntax_type_and_duplicate_key_failures_roll_back_both_views() {
    for operation in 0..6 {
        let source = "timeout: 30 # seconds\n";
        let mut config = Editable::<Config>::parse(source).unwrap();
        let result = config.edit_syntax(|e| {
            let root = e.root();
            let entries = e.mapping_entries(&root)?;
            match operation {
                0 => e.replace(&entries[0].1, "256"),
                1 => e.replace(&entries[0].1, "["),
                2 => e.insert_entry(&root, 1, "timeout", "60"),
                3 => e.remove_entry(&root, 0),
                4 => e.replace(&entries[0].1, "[60]"),
                _ => e.replace(&entries[0].1, "{value: 60}"),
            }
        });
        assert!(result.is_err());
        assert_eq!(config.value().timeout.value, 30);
        assert_eq!(config.syntax().source(), source);
    }
    for source in ["[", "---\n1\n---\n2", "timeout: bad"] {
        assert!(Editable::<Config>::parse(source).is_err());
    }
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
