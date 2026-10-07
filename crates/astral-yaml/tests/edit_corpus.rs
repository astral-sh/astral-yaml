//! Compare composed edits with the original corpus's ordered syntax and aliases.

use std::collections::HashMap;

use astral_yaml::{
    NodeId,
    comments::CommentPlacement,
    edit::{EditableDocument, NodeKind},
};
use astral_yaml_dev::fixtures;

#[derive(Debug, PartialEq)]
enum Content {
    Scalar(String),
    Sequence(usize),
    Mapping(usize),
    Alias(usize),
}

#[derive(Debug, PartialEq)]
struct Occurrence {
    content: Content,
    anchor: Option<String>,
    tag: Option<String>,
    comments: Vec<(String, CommentPlacement)>,
}

/// Preorder gives corresponding occurrences an identity independent of parser
/// arena indices. Alias edges use these identities, including shadowed anchors.
fn project(document: &EditableDocument) -> Vec<Occurrence> {
    let mut order = Vec::new();
    let mut pending = vec![document.root()];
    while let Some(id) = pending.pop() {
        order.push(id);
        match document.node(id).unwrap().kind() {
            NodeKind::Sequence { items, .. } => pending.extend(items.iter().rev()),
            NodeKind::Mapping { entries, .. } => {
                for &(key, value) in entries.iter().rev() {
                    pending.extend([value, key]);
                }
            }
            _ => {}
        }
    }
    let identities: HashMap<NodeId, usize> = order
        .iter()
        .enumerate()
        .map(|(index, &id)| (id, index))
        .collect();
    order
        .into_iter()
        .map(|id| {
            let node = document.node(id).unwrap();
            Occurrence {
                content: match node.kind() {
                    NodeKind::Scalar { value, .. } => Content::Scalar(value.clone()),
                    NodeKind::Sequence { items, .. } => Content::Sequence(items.len()),
                    NodeKind::Mapping { entries, .. } => Content::Mapping(entries.len()),
                    NodeKind::Alias { target } => Content::Alias(identities[target]),
                },
                anchor: node.anchor().map(str::to_owned),
                tag: node.tag().map(str::to_owned),
                comments: node
                    .comments()
                    .iter()
                    .map(|comment| (comment.text().to_owned(), comment.placement()))
                    .collect(),
            }
        })
        .collect()
}

fn assert_projection(document: &EditableDocument, expected: &[Occurrence], fixture: &str) {
    let actual = project(document);
    assert_eq!(actual.len(), expected.len(), "{fixture}: occurrence count");
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(actual, expected, "{fixture}: occurrence {index}");
    }
}

#[test]
fn corpus_normalization_and_composed_edits_preserve_the_expected_graph() {
    let mut tested = 0;
    for fixture in fixtures::manifest()
        .into_iter()
        .filter(fixtures::Fixture::valid_syntax)
    {
        let source = fixture.source();
        let stream = astral_yaml::parse(&source).unwrap();
        // Editing explicitly accepts one document. Syntax validity comes from
        // the corpus manifest; unexpected parse failures are not skipped.
        if stream.documents().len() != 1 {
            continue;
        }
        tested += 1;
        let mut document = EditableDocument::parse(&source).unwrap();
        let mut expected = project(&document);
        let normalized = document
            .normalized_yaml()
            .unwrap_or_else(|error| panic!("{}: {error}", fixture.file));
        let normalized = EditableDocument::parse(&normalized).unwrap();
        assert_projection(&normalized, &expected, &fixture.file);

        let unchanged = document.source().to_owned();
        document.edit(|_| Ok(())).unwrap();
        assert_eq!(document.source(), unchanged);

        let positions: Vec<_> = expected
            .iter()
            .enumerate()
            .filter_map(|(index, node)| matches!(node.content, Content::Scalar(_)).then_some(index))
            .collect();
        if positions.is_empty() {
            continue;
        }
        let mut selected = vec![0, positions.len() / 2, positions.len() - 1];
        selected.sort_unstable();
        selected.dedup();
        for &index in &selected {
            expected[positions[index]].content = Content::Scalar("edited".to_owned());
            expected[positions[index]].tag = None;
        }
        document
            .edit(|editor| {
                let mut pending = vec![editor.root()];
                let mut scalars = Vec::new();
                while let Some(handle) = pending.pop() {
                    match editor.node(&handle)?.kind() {
                        NodeKind::Scalar { .. } => scalars.push(handle),
                        NodeKind::Sequence { .. } => {
                            pending.extend(editor.sequence_items(&handle)?.into_iter().rev());
                        }
                        NodeKind::Mapping { .. } => {
                            for (key, value) in editor.mapping_entries(&handle)?.into_iter().rev() {
                                pending.extend([value, key]);
                            }
                        }
                        NodeKind::Alias { .. } => {}
                    }
                }
                for index in selected {
                    editor.replace_preserving_style(&scalars[index], "\"edited\"")?;
                }
                Ok(())
            })
            .unwrap_or_else(|error| panic!("{}: {error}", fixture.file));
        assert_projection(&document, &expected, &fixture.file);
    }
    assert!(
        tested > 0,
        "the corpus must contain single-document fixtures"
    );
}
