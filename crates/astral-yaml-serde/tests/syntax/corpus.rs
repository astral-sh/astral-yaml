//! Compare composed edits with the original corpus's ordered syntax and aliases.

mod layouts;

use std::collections::HashMap;

use crate::syntax::{DocumentSyntax, Editor, NodeKind};
use astral_yaml::comments::CommentPlacement;
use astral_yaml::{CollectionStyle, NodeId};
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
    comments: Vec<(String, CommentPlacement, Option<usize>)>,
}

/// Preorder gives corresponding occurrences an identity independent of parser
/// arena indices. Alias edges use these identities, including shadowed anchors.
fn project(document: &DocumentSyntax) -> Vec<Occurrence> {
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
            let children: Vec<_> = match node.kind() {
                NodeKind::Sequence { items, .. } => items.clone(),
                NodeKind::Mapping { entries, .. } => entries.iter().map(|&(key, _)| key).collect(),
                _ => Vec::new(),
            };
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
                    .map(|comment| {
                        // Derive the collection gap from public source positions,
                        // independently of the editor's stored comment boundary.
                        let gap = (comment.placement() == CommentPlacement::Detached
                            && !children.is_empty())
                        .then(|| {
                            children
                                .iter()
                                .position(|&child| {
                                    comment.span().start()
                                        < document.node(child).unwrap().span().start()
                                })
                                .unwrap_or(children.len())
                        });
                        (comment.text().to_owned(), comment.placement(), gap)
                    })
                    .collect(),
            }
        })
        .collect()
}

fn assert_projection(document: &DocumentSyntax, expected: &[Occurrence], fixture: &str) {
    let actual = project(document);
    assert_eq!(actual.len(), expected.len(), "{fixture}: occurrence count");
    for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
        assert_eq!(actual, expected, "{fixture}: occurrence {index}");
    }
}

fn node_order(editor: &Editor<'_>) -> Vec<NodeId> {
    let mut order = Vec::new();
    let mut pending = vec![editor.root()];
    while let Some(id) = pending.pop() {
        match editor.node(id).unwrap().kind() {
            NodeKind::Sequence { items, .. } => pending.extend(items.iter().rev()),
            NodeKind::Mapping { entries, .. } => {
                for &(key, value) in entries.iter().rev() {
                    pending.extend([value, key]);
                }
            }
            _ => {}
        }
        order.push(id);
    }
    order
}

#[derive(Debug, Clone, Copy)]
enum CommentEdit {
    Replace,
    Remove,
}

#[test]
fn corpus_comment_edits_preserve_the_expected_graph() {
    let mut tested = 0;
    for fixture in fixtures::manifest()
        .into_iter()
        .filter(fixtures::Fixture::valid_syntax)
    {
        let source = fixture.source();
        if astral_yaml::parse(&source).unwrap().documents().len() != 1 {
            continue;
        }
        let original = DocumentSyntax::parse(&source).unwrap();
        let before = project(&original);
        if before.iter().all(|node| node.comments.is_empty()) {
            continue;
        }
        tested += 1;
        for operation in [CommentEdit::Replace, CommentEdit::Remove] {
            for flow in [false, true] {
                let mut document = DocumentSyntax::parse(&source).unwrap();
                let mut expected = project(&document);
                let context = format!("{} {operation:?} flow={flow}", fixture.file);
                document
                    .edit_syntax(|editor| {
                        for (id, node) in node_order(editor).into_iter().zip(&mut expected) {
                            for (index, (text, _, _)) in node.comments.iter_mut().enumerate().rev()
                            {
                                *text = [" edited", "", "\tcomment: [#] é\u{2028}end"][index % 3]
                                    .to_owned();
                                match operation {
                                    CommentEdit::Remove => editor.remove_comment(id, index)?,
                                    CommentEdit::Replace => {
                                        editor.replace_comment(id, index, text)?;
                                    }
                                }
                            }
                            if matches!(operation, CommentEdit::Remove) {
                                node.comments.clear();
                            }
                            if flow
                                && matches!(
                                    node.content,
                                    Content::Sequence(_) | Content::Mapping(_)
                                )
                            {
                                editor.set_collection_style(id, CollectionStyle::Flow)?;
                            }
                        }
                        Ok(())
                    })
                    .unwrap_or_else(|error| panic!("{context}: {error}"));
                assert_projection(&document, &expected, &context);
                let normalized = document.normalized_yaml().unwrap();
                let normalized = DocumentSyntax::parse(&normalized).unwrap();
                assert_projection(&normalized, &expected, &context);
            }
        }
    }
    assert!(tested > 0, "the corpus must contain comments");
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
        // Editing explicitly accepts one document. Syntax validity comes from the
        // corpus manifest. Unexpected parse failures are not skipped.
        if stream.documents().len() != 1 {
            continue;
        }
        tested += 1;
        let mut document = DocumentSyntax::parse(&source).unwrap();
        let mut expected = project(&document);
        let normalized = document
            .normalized_yaml()
            .unwrap_or_else(|error| panic!("{}: {error}", fixture.file));
        let normalized = DocumentSyntax::parse(&normalized).unwrap();
        assert_projection(&normalized, &expected, &fixture.file);

        let unchanged = document.source().to_owned();
        document.edit_syntax(|_| Ok(())).unwrap();
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
            .edit_syntax(|editor| {
                let scalars: Vec<_> = node_order(editor)
                    .into_iter()
                    .filter(|id| {
                        matches!(editor.node(*id).unwrap().kind(), NodeKind::Scalar { .. })
                    })
                    .collect();
                for index in selected {
                    editor.replace_preserving_style(scalars[index], "\"edited\"")?;
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

#[test]
fn corpus_collection_layout_changes_preserve_the_expected_graph() {
    let mut tested = 0;
    for fixture in fixtures::manifest()
        .into_iter()
        .filter(fixtures::Fixture::valid_syntax)
    {
        let source = fixture.source();
        if astral_yaml::parse(&source).unwrap().documents().len() != 1 {
            continue;
        }
        let original = DocumentSyntax::parse(&source).unwrap();
        tested += 1;
        let expected = project(&original);
        let mut document = original;
        for style in [
            CollectionStyle::Flow,
            CollectionStyle::Block,
            CollectionStyle::Flow,
        ] {
            let result = document.edit_syntax(|editor| {
                for id in node_order(editor) {
                    let empty = match editor.node(id)?.kind() {
                        NodeKind::Sequence { items, .. } => items.is_empty(),
                        NodeKind::Mapping { entries, .. } => entries.is_empty(),
                        _ => continue,
                    };
                    editor.set_collection_style(
                        id,
                        if empty { CollectionStyle::Flow } else { style },
                    )?;
                }
                Ok(())
            });
            result.unwrap_or_else(|error| panic!("{} {style:?}: {error}", fixture.file));
            assert_projection(&document, &expected, &fixture.file);
            let normalized = document.normalized_yaml().unwrap();
            let normalized = DocumentSyntax::parse(&normalized).unwrap();
            assert_projection(&normalized, &expected, &fixture.file);
            assert_eq!(
                normalized.normalized_yaml().unwrap(),
                normalized.source(),
                "{}: normalization is idempotent after {style:?}",
                fixture.file
            );
        }
    }
    assert!(
        tested > 0,
        "the corpus must contain single-document fixtures"
    );
}
