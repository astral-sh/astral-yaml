use test_case::test_case;

use super::{Cursor, Error};
use crate::syntax::DocumentSyntax;
use astral_yaml::ParseOptions;
use astral_yaml_dev::edit_diff;

fn edit(
    document: &mut DocumentSyntax,
    operation: impl FnOnce(Cursor<'_, '_>) -> Result<(), Error>,
) -> Result<(), Error> {
    document.try_edit_syntax(|syntax| super::transaction(syntax, operation))
}

#[test]
fn string_key_updates_leave_application_tagged_entries_intact() {
    let mut diffs = Vec::new();
    // Exercise mixed key tags without imposing a Serde key type on the syntax.
    for source in [
        "!custom name: tagged\n",
        "!custom name: tagged\nname: old\n",
    ] {
        let mut document = DocumentSyntax::parse(source).unwrap();
        edit(&mut document, |root| {
            root.into_mapping()?.set("name", "new")?;
            Ok(())
        })
        .unwrap();
        diffs.push(format!(
            "set_entry\n{}",
            edit_diff(source, document.source())
        ));
        let before = document.to_yaml();
        edit(&mut document, |root| {
            assert!(root.into_mapping()?.remove("name")?);
            Ok(())
        })
        .unwrap();
        diffs.push(format!(
            "remove_field\n{}",
            edit_diff(&before, document.source())
        ));
    }
    insta::assert_snapshot!(diffs.join("\n"), @r#"
    set_entry
    @@ -1 +1,2 @@
    -!custom name: tagged
    +!<!custom> name: tagged
    +"name": "new"

    remove_field
    @@ -1,2 +1 @@
     !<!custom> name: tagged
    -"name": "new"

    set_entry
    @@ -1,2 +1,2 @@
     !custom name: tagged
    -name: old
    +name: new

    remove_field
    @@ -1,2 +1 @@
    -!custom name: tagged
    -name: new
    +!<!custom> name: tagged
    "#);
}

#[test]
fn exhausted_mutations_poison_subsequent_setters() {
    let mut document = DocumentSyntax::parse_with_options(
        "1",
        ParseOptions {
            max_nodes: 3,
            ..ParseOptions::default()
        },
    )
    .unwrap();
    let result: Result<(), Error> = edit(&mut document, |mut root| {
        root.set(2)?;
        let error = root.set(3).unwrap_err();
        assert_eq!(root.set(4).unwrap_err(), error);
        Ok(())
    });
    assert!(result.is_err());
    assert_eq!(document.source(), "1");
}

#[test_case(false)]
#[test_case(true)]
fn cursor_reads_and_mutations_respect_the_syntax_work_budget(mutate: bool) {
    let mut document = DocumentSyntax::parse_with_options(
        "1",
        ParseOptions {
            max_nodes: 1,
            ..ParseOptions::default()
        },
    )
    .unwrap();
    let result: Result<(), Error> = edit(&mut document, |mut root| {
        root.node()?;
        if mutate {
            root.set(2)
        } else {
            root.node().map(|_| ())
        }
    });
    assert!(
        result
            .unwrap_err()
            .message
            .contains("editing resource limit")
    );
    assert_eq!(document.source(), "1");
}

#[test_case(0, "$[\"key\"]"; "optional lookup")]
#[test_case(1, "$[\"key\"]"; "required lookup")]
#[test_case(2, "$[\"key\"]"; "default entry")]
#[test_case(3, "$"; "indexed key")]
#[test_case(4, "$"; "indexed value")]
#[test_case(5, "$"; "node inspection")]
fn mapping_views_report_resource_errors_with_paths(operation: usize, path: &str) {
    let source = "key: value";
    let mut document = DocumentSyntax::parse_with_options(
        source,
        ParseOptions {
            max_nodes: 12,
            ..ParseOptions::default()
        },
    )
    .unwrap();
    let result: Result<(), Error> = edit(&mut document, |root| {
        let mut mapping = root.into_mapping()?;
        while mapping.node().is_ok() {}
        assert_eq!(mapping.len(), 1);
        assert!(!mapping.is_empty());
        match operation {
            0 => mapping.get("key").map(|_| ()),
            1 => mapping.field("key").map(|_| ()),
            2 => mapping.entry("key").map(|_| ()),
            3 => mapping.key_at(0).map(|_| ()),
            4 => mapping.value_at(0).map(|_| ()),
            _ => mapping.node().map(|_| ()),
        }
    });
    let error = result.unwrap_err();
    assert_eq!(error.path.as_deref(), Some(path));
    assert_eq!(error.message, "editing resource limit exceeded");
    assert_eq!(document.source(), source);
}

#[test_case(false)]
#[test_case(true)]
fn sequence_views_report_resource_errors_with_paths(required: bool) {
    let source = "[value]";
    let mut document = DocumentSyntax::parse_with_options(
        source,
        ParseOptions {
            max_nodes: 12,
            ..ParseOptions::default()
        },
    )
    .unwrap();
    let result: Result<(), Error> = edit(&mut document, |root| {
        let mut sequence = root.into_sequence()?;
        while sequence.node().is_ok() {}
        assert_eq!(sequence.len(), 1);
        assert!(!sequence.is_empty());
        if required {
            sequence.item(0).map(|_| ())
        } else {
            sequence.get(0).map(|_| ())
        }
    });
    let error = result.unwrap_err();
    assert_eq!(error.path.as_deref(), Some("$"));
    assert_eq!(error.message, "editing resource limit exceeded");
    assert_eq!(document.source(), source);
}

#[test]
fn serialization_failure_blocks_retries_and_rolls_back_prior_edits() {
    struct FallibleValue {
        attempts: std::cell::Cell<usize>,
    }

    impl serde::Serialize for FallibleValue {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let attempt = self.attempts.get() + 1;
            self.attempts.set(attempt);
            if attempt == 2 {
                Err(serde::ser::Error::custom("serialization failed"))
            } else {
                serializer.serialize_str("changed")
            }
        }
    }

    let source = "old\n";
    let mut document = crate::Editable::<String>::parse(source).unwrap();
    let value = FallibleValue {
        attempts: std::cell::Cell::new(0),
    };
    let error = document
        .edit(|mut root| {
            root.set(&value)?;
            let first_error = root.set(&value).unwrap_err();
            assert_eq!(root.set(&value).unwrap_err(), first_error);
            assert_eq!(value.attempts.get(), 2);
            assert!(matches!(root.node()?.kind(), super::NodeKind::Scalar { value, .. } if value == "changed"));
            Ok(())
        })
        .unwrap_err();
    assert_eq!(error.to_string(), "$: serialization failed");
    assert_eq!(document.source(), source);
    assert_eq!(document.value(), "old");
}
