use test_case::test_case;

use super::{Cursor, Error};
use crate::syntax::DocumentSyntax;
use astral_yaml::ParseOptions;
use astral_yaml_dev::edit_diff;
use serde::ser::{Serialize, SerializeSeq, Serializer};
use std::cell::Cell;

fn edit(
    document: &mut DocumentSyntax,
    operation: impl FnOnce(Cursor<'_, '_>) -> Result<(), Error>,
) -> Result<(), Error> {
    document.try_edit_syntax(|syntax| super::transaction(syntax, operation))
}

#[test]
fn string_key_updates_leave_application_tagged_entries_intact() {
    // Two scalar keys with the same text can differ by tag.
    // https://yaml.org/spec/1.2.2/#3213-node-comparison
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
        attempts: Cell<usize>,
    }

    impl Serialize for FallibleValue {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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
        attempts: Cell::new(0),
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

struct Generated<'a>(&'a Cell<usize>);

impl Serialize for Generated<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(None)?;
        for _ in 0..100 {
            self.0.set(self.0.get() + 1);
            seq.serialize_element("value")?;
        }
        seq.end()
    }
}

#[test]
fn public_serialization_can_write_the_full_sequence() {
    let visits = Cell::new(0);
    let yaml = crate::to_string(&Generated(&visits)).unwrap();
    assert_eq!(
        crate::from_str::<Vec<String>>(&yaml).unwrap(),
        ["value"; 100]
    );
    assert_eq!(visits.get(), 100);
}

#[derive(Clone, Copy)]
enum Write {
    Replace,
    SetEntry,
    InsertItem,
    DefaultEntry,
}

#[test_case("null", Write::Replace, "$", 3; "root replacement")]
#[test_case("{key: 0}", Write::SetEntry, "$[\"key\"]", 3; "mapping replacement")]
#[test_case("{}", Write::SetEntry, "$[\"key\"]", 2; "new mapping entry")]
#[test_case("[0]", Write::InsertItem, "$", 3; "new sequence item")]
#[test_case("{}", Write::DefaultEntry, "$[\"key\"]", 2; "vacant entry default")]
fn oversized_edit_stops_serializing_early(source: &str, write: Write, path: &str, count: usize) {
    let options = ParseOptions {
        max_input_bytes: 20,
        ..ParseOptions::default()
    };
    let mut document = crate::EditableYaml::parse_with_options(source, options).unwrap();
    let visits = Cell::new(0);
    let error = document
        .edit(|mut root| {
            let value = Generated(&visits);
            match write {
                Write::Replace => root.set(value),
                Write::SetEntry => root.into_mapping()?.set("key", value).map(drop),
                Write::InsertItem => root.into_sequence()?.insert(0, value).map(drop),
                Write::DefaultEntry => root
                    .into_mapping()?
                    .entry("key")?
                    .or_insert(value)
                    .map(drop),
            }
        })
        .unwrap_err();
    assert_eq!(error.path.as_deref(), Some(path));
    assert_eq!(error.message, "inserted YAML exceeds input byte limit");
    assert_eq!(error.span, astral_yaml::Span::new(0, source.len()));
    assert_eq!(visits.get(), count);
    assert_eq!(document.source(), source);
}

#[test_case(8, 0; "no bytes remain")]
#[test_case(10, 1; "first item exceeds the remaining bytes")]
fn replacements_share_the_budget_and_roll_back(limit: usize, count: usize) {
    let source = "old\n";
    let options = ParseOptions {
        max_input_bytes: limit,
        ..ParseOptions::default()
    };
    let mut document = crate::Editable::<String>::parse_with_options(source, options).unwrap();
    let visits = Cell::new(0);
    let error = document
        .edit(|mut root| {
            root.set("first")?;
            let error = root.set(Generated(&visits)).unwrap_err();
            assert_eq!(root.set("another").unwrap_err(), error);
            assert_eq!(root.read::<String>()?, "first");
            Ok(())
        })
        .unwrap_err();
    assert_eq!(error.message, "inserted YAML exceeds input byte limit");
    assert_eq!(visits.get(), count);
    assert_eq!(document.source(), source);
    assert_eq!(document.value(), "old");
}

#[test]
fn escaped_new_key_uses_the_budget_before_the_value_is_serialized() {
    let options = ParseOptions {
        max_input_bytes: 10,
        ..ParseOptions::default()
    };
    let mut document = crate::EditableYaml::parse_with_options("{}", options).unwrap();
    let visits = Cell::new(0);
    let error = document
        .edit(|root| {
            root.into_mapping()?
                .set("\n\u{7}", Generated(&visits))
                .map(drop)
        })
        .unwrap_err();
    assert_eq!(error.message, "inserted YAML exceeds input byte limit");
    assert_eq!(visits.get(), 0);
    assert_eq!(document.source(), "{}");
}
