use super::*;
use crate::edit::EditableDocument;
use test_case::test_case;

#[test]
fn collection_views_allow_repeated_edits_and_nested_defaults() {
    let mut document = EditableDocument::parse(
        "services:\n  db:\n    healthcheck:\n      retries: 1\n      interval: '1s'\nitems: []\n",
    )
    .unwrap();
    let lengths = document
        .edit(|editor| {
            let mut health = editor
                .root_mut()
                .field("services")?
                .field("db")?
                .field("healthcheck")?
                .into_mapping()?;
            health.field("retries")?.set_yaml("5")?;
            health.field("interval")?.set_yaml("5s")?;
            health.set_yaml("retries", "8")?;
            health.entry("timeout")?.or_insert_yaml("10s")?;
            let occupied = health.entry("timeout")?;
            assert!(occupied.is_occupied());
            occupied.or_insert_yaml("[")?;
            let vacant = health.entry("nested")?;
            assert!(!vacant.is_occupied());
            vacant
                .or_insert_yaml("{}")?
                .into_mapping()?
                .entry("yes")?
                .or_insert_yaml("true")?;
            let mapping_len = health.len();
            assert!(!health.is_empty());
            assert!(health.get("absent")?.is_none());
            health.get("retries")?.unwrap().set_yaml("9")?;
            assert!(!health.remove("absent")?);
            assert!(health.remove("interval")?);
            health
                .into_cursor()
                .field("timeout")?
                .set_scalar_style(ScalarStyle::SingleQuoted)?;

            let mut items = editor.root_mut().field("items")?.into_sequence()?;
            assert!(items.is_empty());
            items
                .push_yaml("first")?
                .set_scalar_style(ScalarStyle::DoubleQuoted)?;
            items.push_yaml("third")?;
            items.insert_yaml(1, "second")?;
            assert!(!items.is_empty());
            items.get(1)?.unwrap().set_yaml("middle")?;
            assert!(items.get(8)?.is_none());
            items.item(0)?.set_yaml("start")?;
            let sequence_len = items.len();
            items.remove(2)?;
            items.into_cursor().item(1)?.replace_yaml("'end'")?;
            Ok((mapping_len, sequence_len))
        })
        .unwrap();
    insta::assert_debug_snapshot!((lengths, document.source()), @r###"
    (
        (
            4,
            3,
        ),
        "services:\n  db:\n    healthcheck:\n      retries: 9\n      \"timeout\": '10s'\n      \"nested\": {\"yes\": true}\nitems: [\"start\", 'end']\n",
    )
    "###);
}

// https://yaml.org/spec/1.2.2/#1032-tag-resolution
#[test]
fn decoded_string_lookup_and_arbitrary_keys_share_the_same_mapping() {
    let mut document = EditableDocument::parse(
        "{'a.b/c': one, !!str true: two, ! false: three, 1: numeric, !custom name: tagged, ? [x] : complex}",
    ).unwrap();
    let values = document
        .edit(|editor| {
            let mut mapping = editor.root_mut().into_mapping()?;
            mapping.field("a.b/c")?.set_yaml("updated")?;
            mapping.field("true")?.set_yaml("string")?;
            mapping.field("false")?.set_yaml("also-string")?;
            assert!(mapping.get("1")?.is_none());
            assert!(mapping.get("name")?.is_none());
            mapping.value_at(5)?.set_yaml("arbitrary")?;
            mapping.key_at(5)?.replace_yaml("[y]")?;
            let key = mapping.key_at(5)?.node()?.kind().clone();
            mapping.set_yaml("1", "quoted")?;
            mapping.set_yaml("\"\\\n\r\t\0\u{85}\u{2028}\u{2029}", "escaped")?;
            Ok(key)
        })
        .unwrap();
    insta::assert_debug_snapshot!((values, document.source()), @r###"
    (
        Sequence {
            items: [
                NodeId(
                    14,
                ),
            ],
            style: Flow,
        },
        "{'a.b/c': updated, !<tag:yaml.org,2002:str> true: string, ! false: also-string, 1: numeric, !<!custom> name: tagged, ? [y]\n  : arbitrary, \"1\": quoted, \"\\\"\\\\\\n\\r\\t\\U00000000\\U00000085\\U00002028\\U00002029\": escaped}",
    )
    "###);
}

#[test]
fn view_comments_styles_and_node_inspection() {
    let mut document = EditableDocument::parse("{mapping: {a: one}, sequence: [two]}").unwrap();
    let shapes = document
        .edit(|editor| {
            let mut mapping = editor.root_mut().field("mapping")?.into_mapping()?;
            mapping.set_collection_style(CollectionStyle::Flow)?;
            mapping.append_comment(" first", CommentPlacement::Leading)?;
            mapping.replace_comment(0, " temporary")?;
            let mapping_comments = mapping.node()?.comments().len();
            mapping.remove_comment(0)?;
            mapping.append_comment(" mapping", CommentPlacement::Leading)?;
            let mut sequence = editor.root_mut().field("sequence")?.into_sequence()?;
            sequence.set_collection_style(CollectionStyle::Flow)?;
            sequence.append_comment(" second", CommentPlacement::Leading)?;
            sequence.replace_comment(0, " temporary")?;
            let sequence_comments = sequence.node()?.comments().len();
            sequence.remove_comment(0)?;
            sequence.append_comment(" sequence", CommentPlacement::Leading)?;
            Ok((mapping_comments, sequence_comments))
        })
        .unwrap();
    insta::assert_debug_snapshot!((shapes, document.source()), @r###"
    (
        (
            1,
            1,
        ),
        "{\n  # mapping\n  mapping: {a: one}, \n  # sequence\n  sequence: [two]}",
    )
    "###);
}

#[derive(Clone, Copy)]
enum Navigation {
    Field,
    Get,
    Item,
    Mapping,
    Sequence,
    Entry,
    Key,
    Value,
}

#[test_case("scalar", Navigation::Field; "field on scalar")]
#[test_case("scalar", Navigation::Get; "get on scalar")]
#[test_case("scalar", Navigation::Item; "item on scalar")]
#[test_case("scalar", Navigation::Mapping; "wrong mapping conversion")]
#[test_case("scalar", Navigation::Sequence; "wrong sequence conversion")]
#[test_case("{}", Navigation::Field; "missing field")]
#[test_case("[]", Navigation::Item; "missing item")]
#[test_case("{a: 1, 'a': 2}", Navigation::Field; "ambiguous field")]
#[test_case("{a: 1, 'a': 2}", Navigation::Entry; "ambiguous entry")]
#[test_case("{}", Navigation::Key; "missing indexed key")]
#[test_case("{}", Navigation::Value; "missing indexed value")]
fn navigation_errors_are_recoverable(source: &str, operation: Navigation) {
    let mut document = EditableDocument::parse(source).unwrap();
    document
        .edit(|editor| {
            let result = match operation {
                Navigation::Field => editor.root_mut().field("a").map(|_| ()),
                Navigation::Get => editor.root_mut().get("a").map(|_| ()),
                Navigation::Item => editor.root_mut().item(0).map(|_| ()),
                Navigation::Mapping => editor.root_mut().into_mapping().map(|_| ()),
                Navigation::Sequence => editor.root_mut().into_sequence().map(|_| ()),
                Navigation::Entry => editor.root_mut().into_mapping()?.entry("a").map(|_| ()),
                Navigation::Key => editor.root_mut().into_mapping()?.key_at(0).map(|_| ()),
                Navigation::Value => editor.root_mut().into_mapping()?.value_at(0).map(|_| ()),
            };
            assert!(result.is_err());
            editor.root_mut().set_yaml("recovered")
        })
        .unwrap();
    assert_eq!(document.source(), "recovered");
}

#[derive(Clone, Copy)]
enum Mutation {
    Set,
    Remove,
    Entry,
    Insert,
    Bounds,
    RemoveItem,
}

#[test_case("{a: 1, 'a': 2}", Mutation::Set; "ambiguous upsert")]
#[test_case("{a: 1, 'a': 2}", Mutation::Remove; "ambiguous removal")]
#[test_case("{}", Mutation::Set; "invalid inserted value")]
#[test_case("{a: 1}", Mutation::Set; "invalid existing value")]
#[test_case("{}", Mutation::Entry; "invalid default")]
#[test_case("[]", Mutation::Insert; "invalid item")]
#[test_case("[]", Mutation::Bounds; "invalid insertion index")]
#[test_case("[]", Mutation::RemoveItem; "invalid removal index")]
fn handled_mutation_errors_still_prevent_commit(source: &str, operation: Mutation) {
    let mut document = EditableDocument::parse(source).unwrap();
    let result = document.edit(|editor| {
        let result = match operation {
            Mutation::Set => editor
                .root_mut()
                .into_mapping()?
                .set_yaml("a", "[")
                .map(|_| ()),
            Mutation::Remove => editor.root_mut().into_mapping()?.remove("a").map(|_| ()),
            Mutation::Entry => editor
                .root_mut()
                .into_mapping()?
                .entry("a")?
                .or_insert_yaml("[")
                .map(|_| ()),
            Mutation::Insert => editor
                .root_mut()
                .into_sequence()?
                .push_yaml("[")
                .map(|_| ()),
            Mutation::Bounds => editor
                .root_mut()
                .into_sequence()?
                .insert_yaml(1, "valid")
                .map(|_| ()),
            Mutation::RemoveItem => editor.root_mut().into_sequence()?.remove(0),
        };
        assert!(result.is_err());
        assert!(
            editor
                .root_mut()
                .append_comment("cannot commit", CommentPlacement::Leading)
                .is_err()
        );
        Ok(())
    });
    assert!(result.is_err());
    assert_eq!(document.source(), source);
}

#[test]
fn selection_and_insertion_handles_refer_to_the_exact_node() {
    let mut document = EditableDocument::parse("{sequence: []}").unwrap();
    document
        .edit(|editor| {
            let root = editor.root();
            let (_, sequence) = editor.mapping_entry(&root, 0)?.unwrap();
            let item = editor.insert_item(&sequence, 0, "first")?;
            editor.cursor_mut(&item)?.set_yaml("updated")?;
            let (key, value) = editor.insert_entry(&root, 1, "name", "value")?;
            editor
                .cursor_mut(&key)?
                .set_scalar_style(ScalarStyle::DoubleQuoted)?;
            editor.cursor_mut(&value)?.set_yaml("changed")?;
            let selected = editor.sequence_item(&sequence, 0)?.unwrap();
            editor
                .cursor_mut(&selected)?
                .set_scalar_style(ScalarStyle::SingleQuoted)?;
            assert!(editor.sequence_item(&sequence, 1)?.is_none());
            assert!(editor.mapping_entry(&root, 2)?.is_none());
            assert!(editor.mapping_entry(&sequence, 0).is_err());
            assert!(editor.sequence_item(&root, 0).is_err());
            Ok(())
        })
        .unwrap();
    insta::assert_snapshot!(document.source(), @r#"{sequence: ['updated'], "name": changed}"#);
}

#[test]
fn cursor_from_deleted_or_foreign_handle_is_rejected() {
    let mut document = EditableDocument::parse("[a]").unwrap();
    let foreign = document.edit(|editor| Ok(editor.root())).unwrap();
    document
        .edit(|editor| {
            assert!(editor.cursor_mut(&foreign).is_err());
            let root = editor.root();
            let item = editor.sequence_item(&root, 0)?.unwrap();
            editor.remove_item(&root, 0)?;
            assert!(editor.cursor_mut(&item).is_err());
            Ok(())
        })
        .unwrap();
}

#[test]
fn empty_mapping_and_removal_lengths() {
    let mut document = EditableDocument::parse("{}").unwrap();
    document
        .edit(|editor| {
            let mut mapping = editor.root_mut().into_mapping()?;
            assert!(mapping.is_empty());
            mapping
                .set_yaml("a", "[]")?
                .into_sequence()?
                .push_yaml("one")?;
            assert_eq!(mapping.len(), 1);
            assert!(mapping.remove("a")?);
            assert_eq!(mapping.len(), 0);
            assert!(mapping.is_empty());
            Ok(())
        })
        .unwrap();
}

#[derive(Clone, Copy)]
enum Collection {
    Mapping,
    Sequence,
}

#[test_case("{}", Collection::Mapping)]
#[test_case("[]", Collection::Sequence)]
fn lengths_remain_available_when_work_is_exhausted(source: &str, kind: Collection) {
    let document = EditableDocument::parse(source).unwrap();
    let mut editor = Editor::new(&document);
    let cursor = editor.root_mut();
    cursor
        .editor
        .draft
        .charge(document.options().max_nodes - 1)
        .unwrap();
    match kind {
        Collection::Mapping => {
            let mut mapping = cursor.into_mapping().unwrap();
            assert_eq!(mapping.len(), 0);
            assert!(mapping.is_empty());
            assert!(mapping.get("a").is_err());
        }
        Collection::Sequence => {
            let mut sequence = cursor.into_sequence().unwrap();
            assert_eq!(sequence.len(), 0);
            assert!(sequence.is_empty());
            assert!(sequence.get(0).is_err());
        }
    }
}

#[derive(Clone, Copy)]
enum MappingInsertion {
    Upsert,
    Default,
}

#[test_case(MappingInsertion::Upsert)]
#[test_case(MappingInsertion::Default)]
fn mapping_length_does_not_change_after_failed_insertion(operation: MappingInsertion) {
    let mut document = EditableDocument::parse("{a: one}").unwrap();
    assert!(
        document
            .edit(|editor| {
                let mut mapping = editor.root_mut().into_mapping()?;
                let result = match operation {
                    MappingInsertion::Upsert => mapping.set_yaml("b", "["),
                    MappingInsertion::Default => mapping.entry("b")?.or_insert_yaml("["),
                };
                assert!(result.is_err());
                assert_eq!(mapping.len(), 1);
                assert!(mapping.set_yaml("c", "valid").is_err());
                assert_eq!(mapping.len(), 1);
                Ok(())
            })
            .is_err()
    );
    assert_eq!(document.source(), "{a: one}");
}

#[derive(Clone, Copy)]
enum SequenceMutation {
    Insert,
    Remove,
}

#[test_case(SequenceMutation::Insert)]
#[test_case(SequenceMutation::Remove)]
fn sequence_length_does_not_change_after_failed_mutations(operation: SequenceMutation) {
    let mut document = EditableDocument::parse("[one]").unwrap();
    assert!(
        document
            .edit(|editor| {
                let mut sequence = editor.root_mut().into_sequence()?;
                let result = match operation {
                    SequenceMutation::Insert => sequence.insert_yaml(2, "valid").map(|_| ()),
                    SequenceMutation::Remove => sequence.remove(1),
                };
                assert!(result.is_err());
                assert_eq!(sequence.len(), 1);
                assert!(sequence.remove(0).is_err());
                assert_eq!(sequence.len(), 1);
                Ok(())
            })
            .is_err()
    );
    assert_eq!(document.source(), "[one]");
}

#[derive(Clone, Copy)]
enum Preparation {
    Upsert,
    Remove,
    Comment,
}

#[test_case(Preparation::Upsert)]
#[test_case(Preparation::Remove)]
#[test_case(Preparation::Comment)]
fn failed_mutation_preparation_poisoning(operation: Preparation) {
    let document = EditableDocument::parse("{a: one}").unwrap();
    let mut editor = Editor::new(&document);
    let mut mapping = editor.root_mut().into_mapping().unwrap();
    mapping
        .cursor
        .editor
        .draft
        .charge(document.options().max_nodes - 1)
        .unwrap();
    let result = match operation {
        Preparation::Upsert => mapping.set_yaml("b", "valid").map(|_| ()),
        Preparation::Remove => mapping.remove("a").map(|_| ()),
        Preparation::Comment => mapping.append_comment("valid", CommentPlacement::Leading),
    };
    assert!(result.is_err());
    assert_eq!(mapping.len(), 1);
    assert!(editor.failure.is_some());
    assert!(editor.finish().is_err());
}
