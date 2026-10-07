use super::*;
use astral_yaml_dev::indoc;
use std::collections::HashSet;

/// Derive ownership from the child edges, independently of the arena's index.
fn assert_ownership(draft: &Draft) {
    let mut seen = HashSet::new();
    let mut pending = vec![(draft.root(), None)];
    while let Some((id, parent)) = pending.pop() {
        assert!(seen.insert(id), "a syntax occurrence has exactly one owner");
        let slot = &draft.slots[id.0];
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
        assert_eq!(slot.membership.is_some(), seen.contains(&NodeId(index)));
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
    let original = EditableDocument::parse(indoc! {r"
        selected:
          - old

          # gap

          - removed
        retained: untouched # retained
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
            "&root {first: [&leaf value, *leaf], self: *root}",
            false,
        )
        .unwrap();
    assert_ownership(&draft);
    let first = field(&mut draft, selected, "first");
    let items = draft.sequence_items_mut(first).unwrap();
    let (leaf, alias) = (items[0], items[1]);
    draft.replace(leaf, "&leaf {nested: value}", false).unwrap();
    assert_ownership(&draft);
    let self_alias = field(&mut draft, selected, "self");
    let links = [
        matches!(draft.node(alias).kind, NodeKind::Alias { target, .. } if target == leaf),
        matches!(draft.node(self_alias).kind, NodeKind::Alias { target, .. } if target == selected),
    ];

    draft
        .set_collection_style(first, CollectionStyle::Block)
        .unwrap();
    draft.remove_entry(selected, 0).unwrap();
    assert_ownership(&draft);
    draft
        .insert_entry(selected, 1, "tail", "&tail [*tail]")
        .unwrap();
    assert_ownership(&draft);
    let tail = field(&mut draft, selected, "tail");
    draft.insert_item(tail, 1, "temporary").unwrap();
    assert_ownership(&draft);
    draft.remove_item(tail, 1).unwrap();
    assert_ownership(&draft);
    draft
        .set_collection_style(tail, CollectionStyle::Flow)
        .unwrap();
    let imported_alias = draft.sequence_items_mut(tail).unwrap()[0];
    let inserted_alias =
        matches!(draft.node(imported_alias).kind, NodeKind::Alias { target, .. } if target == tail);
    assert!(
        draft.contains(selected),
        "replacement retains the occurrence's identity"
    );
    assert!(
        old_children
            .into_iter()
            .chain([first, leaf, alias])
            .all(|id| !draft.contains(id))
    );
    assert!(
        links.into_iter().all(|linked| linked),
        "aliases retain replaced targets"
    );
    assert!(
        inserted_alias,
        "an imported alias targets its own fragment root"
    );
    let candidate = draft.freeze();
    assert_eq!(candidate.requested_styles, [tail]);
}

#[test]
fn failed_mutations_leave_the_tree_and_membership_unchanged() {
    type Operation = fn(&mut Draft) -> Result<(), Error>;
    let cases: &[(&str, &str, Operation)] = &[
        ("invalid mapping value", "{}", |draft| {
            draft.insert_entry(draft.root(), 0, "key", "[")
        }),
        ("replacement removal budget", "[a, b]", |draft| {
            draft.work.set(2);
            draft.replace(draft.root(), "new", false)
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
            draft
                .set_scalar_style(draft.root(), ScalarStyle::SingleQuoted)
                .map(|_| ())
        }),
    ];
    let mut errors = Vec::new();
    for &(name, source, operation) in cases {
        let original = EditableDocument::parse(source).unwrap();
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
