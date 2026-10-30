use super::{Draft, ReplacementStyle};
use crate::syntax::{CommentPosition, DocumentSyntax, NodeKind};
use astral_yaml::{CollectionStyle, Error, NodeId, ScalarStyle};
use astral_yaml_dev::indoc;
use std::collections::HashSet;

/// Derive ownership from the child edges, independently of the arena's index.
fn assert_ownership(draft: &Draft) {
    let mut seen = HashSet::new();
    let mut pending = vec![(draft.root(), None)];
    while let Some((id, parent)) = pending.pop() {
        assert!(seen.insert(id), "a syntax occurrence has exactly one owner");
        let slot = &draft.slots[id.index()];
        assert_eq!(slot.membership.as_ref().unwrap().parent, parent);
        let (children, boundaries) = match &slot.node.kind {
            NodeKind::Sequence { items, .. } => (items.clone(), items.clone()),
            NodeKind::Mapping { entries, .. } => (
                entries
                    .iter()
                    .flat_map(|&(key, value)| [key, value])
                    .collect(),
                entries.iter().map(|&(key, _)| key).collect(),
            ),
            NodeKind::Scalar { .. } | NodeKind::Alias { .. } => (Vec::new(), Vec::new()),
        };
        for comment in &slot.node.comments {
            if let CommentPosition::BeforeChild(child) = comment.position {
                assert!(
                    boundaries.contains(&child),
                    "comment boundary names an item or mapping key"
                );
            }
        }
        pending.extend(children.into_iter().map(|child| (child, Some(id))));
    }
    for (index, slot) in draft.slots.iter().enumerate() {
        assert_eq!(
            slot.membership.is_some(),
            seen.contains(&NodeId::from_index(index))
        );
    }
}

fn field(draft: &mut Draft, parent: NodeId, name: &str) -> NodeId {
    let entries = draft.mapping_entries_mut(parent).unwrap().clone();
    entries
        .iter()
        .find_map(|&(key, value)| {
            matches!(&draft.node(key).kind, NodeKind::Scalar { value: text, .. } if text == name)
                .then_some(value)
        })
        .unwrap()
}

#[test]
fn composed_mutations_keep_one_owner_and_retire_style_requests() {
    let original = DocumentSyntax::parse(indoc! {r"
        selected: &root
          - old

          # gap

          - removed
        copy: *root # retained
    "})
    .unwrap();
    let mut draft = Draft::from(&original);
    let root = draft.root();
    let selected = field(&mut draft, root, "selected");
    let old_children = draft.sequence_items_mut(selected).unwrap().clone();
    for &id in &old_children {
        draft
            .set_scalar_style(id, ScalarStyle::SingleQuoted)
            .unwrap();
    }
    assert_ownership(&draft);

    draft
        .replace(
            selected,
            "{first: [value], second: other}",
            ReplacementStyle::Preserve,
        )
        .unwrap();
    assert_ownership(&draft);
    let first = field(&mut draft, selected, "first");
    let leaf = draft.sequence_items_mut(first).unwrap()[0];
    draft
        .replace(leaf, "{nested: value}", ReplacementStyle::Preserve)
        .unwrap();
    assert_ownership(&draft);
    let alias = field(&mut draft, root, "copy");
    assert!(matches!(draft.node(alias).kind, NodeKind::Alias { target } if target == selected));

    draft
        .set_collection_style(first, CollectionStyle::Block)
        .unwrap();
    draft.remove_entry(selected, 0).unwrap();
    assert_ownership(&draft);
    draft.insert_entry(selected, 1, "tail", "[one]").unwrap();
    assert_ownership(&draft);
    let tail = field(&mut draft, selected, "tail");
    draft.insert_item(tail, 1, "temporary").unwrap();
    assert_ownership(&draft);
    draft.remove_item(tail, 1).unwrap();
    assert_ownership(&draft);
    draft
        .set_collection_style(tail, CollectionStyle::Flow)
        .unwrap();
    assert!(
        draft.contains(selected),
        "replacement retains the occurrence's identity"
    );
    assert!(
        old_children
            .into_iter()
            .chain([first, leaf])
            .all(|id| !draft.contains(id))
    );
    assert_eq!(draft.clone().freeze().unwrap().requested_styles, [tail]);
    draft
        .replace(selected, "done", ReplacementStyle::Preserve)
        .unwrap();
    assert_ownership(&draft);
    assert!(!draft.contains(tail));
    assert_eq!(draft.freeze().unwrap().requested_styles, []);
}

#[test]
fn failed_mutations_leave_the_tree_and_membership_unchanged() {
    type Operation = fn(&mut Draft) -> Result<(), Error>;
    let cases: &[(&str, &str, Operation)] = &[
        ("invalid mapping value", "{}", |draft| {
            draft.insert_entry(draft.root(), 0, "key", "[").map(|_| ())
        }),
        ("replacement removal budget", "[a, b]", |draft| {
            draft.work.set(2);
            draft.replace(draft.root(), "new", ReplacementStyle::FromValue)
        }),
        ("sequence removal budget", "[[a, b]]", |draft| {
            draft.work.set(1);
            draft.remove_item(draft.root(), 0)
        }),
        ("mapping removal budget", "{key: [a, b]}", |draft| {
            draft.work.set(1);
            draft.remove_entry(draft.root(), 0)
        }),
        ("scalar type change", "true", |draft| {
            draft.set_scalar_style(draft.root(), ScalarStyle::SingleQuoted)
        }),
    ];
    let mut errors = Vec::new();
    for &(name, source, operation) in cases {
        let original = DocumentSyntax::parse(source).unwrap();
        let mut draft = Draft::from(&original);
        let before = format!("{:?}", draft.slots);
        let error = operation(&mut draft).unwrap_err();
        assert_ownership(&draft);
        assert_eq!(format!("{:?}", draft.slots), before, "{name}");
        errors.push((name, error.message));
    }
    insta::assert_debug_snapshot!(errors, @r#"
    [
        (
            "invalid mapping value",
            "unterminated flow sequence",
        ),
        (
            "replacement removal budget",
            "editing resource limit exceeded",
        ),
        (
            "sequence removal budget",
            "editing resource limit exceeded",
        ),
        (
            "mapping removal budget",
            "editing resource limit exceeded",
        ),
        (
            "scalar type change",
            "requested style changes the scalar type",
        ),
    ]
    "#);
}
