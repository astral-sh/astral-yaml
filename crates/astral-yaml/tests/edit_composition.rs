//! Transaction composition against an independently maintained comment model.

use astral_yaml::{
    Error,
    comments::CommentPlacement,
    edit::{EditableDocument, Editor},
};
use astral_yaml_dev::indoc;

#[derive(Clone, Copy, Debug)]
enum Operation {
    ReplaceFirst,
    ReplaceLast,
    RemoveFirst,
    RemoveLast,
    PrependLeading,
    AppendTrailing,
    AppendInline,
    AppendDetached,
    SetSequence,
    SetScalar,
    SetMapping,
    SetFlowSequence,
    SetLiteral,
    SetEmpty,
    SetTailSequence,
    SetTailMapping,
}

impl Operation {
    fn apply(self, editor: &mut Editor<'_>) -> Result<(), Error> {
        let root = editor.root();
        let count = editor.node(&root)?.comments().len();
        match self {
            Self::ReplaceFirst | Self::ReplaceLast if count > 0 => {
                let index = if matches!(self, Self::ReplaceFirst) {
                    0
                } else {
                    count - 1
                };
                editor.replace_comment(&root, index, " changed")
            }
            Self::RemoveFirst | Self::RemoveLast if count > 0 => {
                let index = if matches!(self, Self::RemoveFirst) {
                    0
                } else {
                    count - 1
                };
                editor.remove_comment(&root, index)
            }
            Self::PrependLeading => {
                editor.insert_comment(&root, 0, " leading", CommentPlacement::Leading)
            }
            Self::AppendTrailing => {
                editor.insert_comment(&root, count, " trailing", CommentPlacement::Trailing)
            }
            Self::AppendInline => {
                editor.insert_comment(&root, count, " inline", CommentPlacement::Inline)
            }
            Self::AppendDetached => {
                editor.insert_comment(&root, count, " detached", CommentPlacement::Detached)
            }
            Self::SetTailSequence => editor.replace(
                &root,
                indoc! {r"
                    - |+
                        new

                "},
            ),
            Self::SetTailMapping => editor.replace(
                &root,
                indoc! {r"
                    key: |+
                      new

                "},
            ),
            Self::SetSequence => editor.replace(
                &root,
                indoc! {r"
                    - new
                "},
            ),
            Self::SetScalar => editor.replace(&root, "value"),
            Self::SetMapping => editor.replace(
                &root,
                indoc! {r"
                    key: new
                "},
            ),
            Self::SetFlowSequence => editor.replace(&root, "[new]"),
            Self::SetLiteral => editor.replace(
                &root,
                indoc! {r"
                    |+
                      new

                "},
            ),
            Self::SetEmpty => editor.replace(
                &root,
                indoc! {r"
                    ---
                "},
            ),
            _ => Ok(()),
        }
    }

    fn content(self) -> Option<&'static str> {
        match self {
            Self::SetTailSequence => Some(indoc! {r"
                - |+
                    new

            "}),
            Self::SetTailMapping => Some(indoc! {r"
                key: |+
                  new

            "}),
            Self::SetSequence => Some(indoc! {r"
                - new
            "}),
            Self::SetScalar => Some("value"),
            Self::SetMapping => Some(indoc! {r"
                key: new
            "}),
            Self::SetFlowSequence => Some("[new]"),
            Self::SetLiteral => Some(indoc! {r"
                |+
                  new

            "}),
            Self::SetEmpty => Some(indoc! {r"
                ---
            "}),
            _ => None,
        }
    }

    fn model(self, comments: &mut Vec<(String, CommentPlacement)>) {
        match self {
            Self::ReplaceFirst => {
                if let Some(comment) = comments.first_mut() {
                    " changed".clone_into(&mut comment.0);
                }
            }
            Self::ReplaceLast => {
                if let Some(comment) = comments.last_mut() {
                    " changed".clone_into(&mut comment.0);
                }
            }
            Self::RemoveFirst if !comments.is_empty() => {
                comments.remove(0);
            }
            Self::RemoveLast => {
                comments.pop();
            }
            Self::PrependLeading => {
                comments.insert(0, (" leading".to_owned(), CommentPlacement::Leading));
            }
            Self::AppendInline => comments.push((" inline".to_owned(), CommentPlacement::Inline)),
            Self::AppendDetached => {
                comments.push((" detached".to_owned(), CommentPlacement::Detached));
            }
            Self::AppendTrailing => {
                comments.push((" trailing".to_owned(), CommentPlacement::Trailing));
            }
            _ => {}
        }
    }
}

fn comments(document: &EditableDocument) -> Vec<(String, CommentPlacement)> {
    document
        .node(document.root())
        .unwrap()
        .comments()
        .iter()
        .map(|comment| (comment.text().to_owned(), comment.placement()))
        .collect()
}

#[test]
fn framing_comment_edits_compose_with_content_edits() {
    // Markers provide root-owned boundaries independent of the root's kind:
    // https://yaml.org/spec/1.2.2/#912-document-markers
    // Every case must succeed. Rejections are failures, not discarded samples.
    use Operation::{
        AppendInline, AppendTrailing, PrependLeading, RemoveFirst, RemoveLast, ReplaceFirst,
        ReplaceLast, SetEmpty, SetFlowSequence, SetLiteral, SetMapping, SetScalar, SetSequence,
    };
    let operations = [
        AppendInline,
        ReplaceFirst,
        ReplaceLast,
        RemoveFirst,
        RemoveLast,
        PrependLeading,
        AppendTrailing,
        SetSequence,
        SetScalar,
        SetMapping,
        SetFlowSequence,
        SetLiteral,
        SetEmpty,
    ];

    for &(source, initial) in root_cases() {
        let original = EditableDocument::parse(source).unwrap();
        for first in operations {
            for second in operations {
                for third in operations {
                    let sequence = [first, second, third];
                    check_root_sequence(&original, source, initial, sequence);
                }
            }
        }
    }
}

#[allow(clippy::too_many_lines, reason = "multiline YAML fixtures")]
fn root_cases() -> &'static [(&'static str, &'static [(&'static str, CommentPlacement)])] {
    use CommentPlacement::{Inline, Leading, Trailing};
    &[
        ("old", &[]),
        ("---", &[]),
        (
            indoc! {r"
                # child
                [ # open
                  old
                ] # close
            "},
            &[(" open", Inline), (" close", Inline)],
        ),
        (
            indoc! {r"
                # child
                { # open
                  key: old
                } # close
            "},
            &[(" open", Inline), (" close", Inline)],
        ),
        (
            indoc! {r"
                # prefix
                old
                # tail
            "},
            &[(" prefix", Leading), (" tail", Trailing)],
        ),
        (
            indoc! {r"
                old # inline
            "},
            &[(" inline", Inline)],
        ),
        (
            indoc! {r"
                [ # open
                  old
                ] # close
            "},
            &[(" open", Inline), (" close", Inline)],
        ),
        (
            indoc! {r"
                &a # property
                [ # open
                  old
                ] # close
            "},
            &[(" property", Inline), (" open", Inline), (" close", Inline)],
        ),
        (
            indoc! {r"
                old
                ...
                # tail
            "},
            &[(" tail", Trailing)],
        ),
        ("\u{feff}old # inline", &[(" inline", Inline)]),
        (
            indoc! {r"
                # prefix
                ---
                old
                ... # end
                # tail
            "},
            &[(" prefix", Leading), (" end", Inline), (" tail", Trailing)],
        ),
        (
            indoc! {r"
                ---
                ...
            "},
            &[],
        ),
        (
            indoc! {r"
                # prefix
                --- old

                ...
            "},
            &[(" prefix", Leading)],
        ),
        (
            indoc! {r"
                # prefix
                --- |-
                  old

                ...
            "},
            &[(" prefix", Leading)],
        ),
        (
            indoc! {r"
                ---
                # prefix
                old
                ...
                # tail
            "},
            &[(" prefix", Leading), (" tail", Trailing)],
        ),
        (
            indoc! {r"
                ---
                old # inline
                ...
            "},
            &[(" inline", Inline)],
        ),
        (
            indoc! {r"
                ---
                old
                # tail
                ...
            "},
            &[(" tail", Trailing)],
        ),
        (
            indoc! {r"
                # prefix
                ---
                &root [*root]
                ...
                # tail
            "},
            &[(" prefix", Leading), (" tail", Trailing)],
        ),
        (
            indoc! {r"
                # prefix
                ---
                - old
                ...
                # tail
            "},
            &[(" prefix", Leading), (" tail", Trailing)],
        ),
    ]
}

#[test]
fn longer_root_edits_match_the_independent_model() {
    use Operation::*;
    let operations = [
        AppendInline,
        AppendDetached,
        ReplaceFirst,
        ReplaceLast,
        RemoveFirst,
        RemoveLast,
        PrependLeading,
        AppendTrailing,
        SetSequence,
        SetScalar,
        SetMapping,
        SetFlowSequence,
        SetLiteral,
        SetEmpty,
        SetTailSequence,
        SetTailMapping,
    ];
    for &(source, initial) in root_cases() {
        let original = EditableDocument::parse(source).unwrap();
        for seed in 1..=128u64 {
            let mut state = seed;
            let mut sequence = [SetScalar; 48];
            let mut planned: Vec<_> = initial
                .iter()
                .map(|(text, placement)| (text.to_string(), *placement))
                .collect();
            for operation in &mut sequence {
                // Deterministic sampling makes failures reproducible without
                // a random-number dependency or exponentially many cases.
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let index = state % u64::try_from(operations.len()).unwrap();
                *operation = operations[usize::try_from(index).unwrap()];
                // A trailing suffix cannot follow detached suffix comments:
                // markers do not reset its adjacency to the document body.
                if matches!(operation, AppendTrailing)
                    && planned
                        .iter()
                        .any(|(_, placement)| *placement == CommentPlacement::Detached)
                {
                    *operation = AppendInline;
                }
                operation.model(&mut planned);
            }
            check_root_sequence(&original, source, initial, sequence);
        }
    }
}

fn check_root_sequence<const N: usize>(
    original: &EditableDocument,
    source: &str,
    initial: &[(&str, CommentPlacement)],
    sequence: [Operation; N],
) {
    let mut expected = initial
        .iter()
        .map(|&(text, placement)| (text.to_owned(), placement))
        .collect();
    for operation in sequence {
        operation.model(&mut expected);
    }
    let mut combined = original.clone();
    combined
        .edit(|editor| {
            for operation in sequence {
                operation.apply(editor)?;
            }
            Ok(())
        })
        .unwrap_or_else(|error| panic!("{source:?}, {sequence:?}: {error}"));
    assert_eq!(comments(&combined), expected, "{source:?}, {sequence:?}");

    let mut separate = original.clone();
    let mut step_comments = initial
        .iter()
        .map(|&(text, placement)| (text.to_owned(), placement))
        .collect();
    let mut step_content = content(original, original.root());
    for operation in sequence {
        operation.model(&mut step_comments);
        if let Some(fragment) = operation.content() {
            let expected = EditableDocument::parse(fragment).unwrap();
            step_content = content(&expected, expected.root());
        }
        separate
            .edit(|editor| operation.apply(editor))
            .unwrap_or_else(|error| panic!("{source:?}, {sequence:?}: {error}"));
        assert_eq!(
            comments(&separate),
            step_comments,
            "{source:?}, {sequence:?}"
        );
        assert_eq!(
            content(&separate, separate.root()),
            step_content,
            "{source:?}, {sequence:?}"
        );
    }
    assert_eq!(comments(&separate), expected, "{source:?}, {sequence:?}");
    // Compare with the requested fragment as well as the
    // other execution. Two executions could share a bug.
    let expected_source = sequence
        .iter()
        .rev()
        .find_map(|operation| operation.content())
        .unwrap_or(source);
    let expected_document = EditableDocument::parse(expected_source).unwrap();
    assert_eq!(
        content(&combined, combined.root()),
        content(&expected_document, expected_document.root()),
        "{source:?}, {sequence:?}"
    );
    assert_eq!(
        content(&separate, separate.root()),
        content(&expected_document, expected_document.root()),
        "{source:?}, {sequence:?}"
    );
    let semantic = |document: &EditableDocument| {
        let stream = astral_yaml::parse(document.source()).unwrap();
        astral_yaml_dev::events(&stream)
            .replace("+DOC ---", "+DOC")
            .replace("-DOC ...", "-DOC")
    };
    assert_eq!(
        semantic(&combined),
        semantic(&separate),
        "{source:?}, {sequence:?}"
    );
}

/// Deliberately excludes presentation, framing, and retained root properties.
/// Alias references are compared without following possibly cyclic targets.
#[derive(Debug, PartialEq)]
enum Content {
    Scalar(String),
    Sequence(Vec<Content>),
    Mapping(Vec<(Content, Content)>),
    Alias(String),
}

fn content(document: &EditableDocument, id: astral_yaml::NodeId) -> Content {
    use astral_yaml::edit::NodeKind;
    match document.node(id).unwrap().kind() {
        NodeKind::Scalar { value, .. } => Content::Scalar(value.clone()),
        NodeKind::Sequence { items, .. } => {
            Content::Sequence(items.iter().map(|&id| content(document, id)).collect())
        }
        NodeKind::Mapping { entries, .. } => Content::Mapping(
            entries
                .iter()
                .map(|&(key, value)| (content(document, key), content(document, value)))
                .collect(),
        ),
        NodeKind::Alias { name, .. } => Content::Alias(name.clone()),
    }
}

#[derive(Clone, Copy, Debug)]
enum NestedOperation {
    PrependItem,
    AppendItem,
    RemoveFirstItem,
    RemoveLastItem,
    ReplaceFirstItem,
    QuoteLastItem,
    FlowItems,
    PrependEntry,
    AppendEntry,
    RemoveFirstEntry,
    ReplaceLastValue,
    FlowMapping,
}

impl NestedOperation {
    fn apply(self, editor: &mut Editor<'_>) -> Result<(), Error> {
        use astral_yaml::{CollectionStyle, ScalarStyle};
        let root_items = editor.sequence_items(&editor.root())?;
        let fields = editor.mapping_entries(&root_items[0])?;
        let items = &fields[0].1;
        let mapping = &fields[1].1;
        let children = editor.sequence_items(items)?;
        let entries = editor.mapping_entries(mapping)?;
        match self {
            Self::PrependItem => editor.insert_item(items, 0, "'true'"),
            Self::AppendItem => editor.insert_item(items, children.len(), "'null'"),
            Self::RemoveFirstItem if !children.is_empty() => editor.remove_item(items, 0),
            Self::RemoveLastItem if !children.is_empty() => {
                editor.remove_item(items, children.len() - 1)
            }
            Self::ReplaceFirstItem if !children.is_empty() => {
                editor.replace(&children[0], "'a: b'")
            }
            Self::QuoteLastItem if !children.is_empty() => {
                editor.set_scalar_style(children.last().unwrap(), ScalarStyle::DoubleQuoted)
            }
            Self::FlowItems => editor.set_collection_style(items, CollectionStyle::Flow),
            Self::PrependEntry => {
                editor.insert_entry(mapping, 0, "[left, right]", "{inner: [new]}")
            }
            Self::AppendEntry => editor.insert_entry(mapping, entries.len(), "'0123'", "'---'"),
            Self::RemoveFirstEntry if !entries.is_empty() => editor.remove_entry(mapping, 0),
            Self::ReplaceLastValue if !entries.is_empty() => {
                editor.replace(&entries.last().unwrap().1, "'false'")
            }
            Self::FlowMapping => editor.set_collection_style(mapping, CollectionStyle::Flow),
            _ => Ok(()),
        }
    }

    fn model(self, items: &mut Vec<Content>, mapping: &mut Vec<(Content, Content)>) {
        let scalar = |value: &str| Content::Scalar(value.to_owned());
        match self {
            Self::PrependItem => items.insert(0, scalar("true")),
            Self::AppendItem => items.push(scalar("null")),
            Self::RemoveFirstItem if !items.is_empty() => {
                items.remove(0);
            }
            Self::RemoveLastItem => {
                items.pop();
            }
            Self::ReplaceFirstItem if !items.is_empty() => items[0] = scalar("a: b"),
            Self::PrependEntry => mapping.insert(
                0,
                (
                    Content::Sequence(vec![scalar("left"), scalar("right")]),
                    Content::Mapping(vec![(
                        scalar("inner"),
                        Content::Sequence(vec![scalar("new")]),
                    )]),
                ),
            ),
            Self::AppendEntry => mapping.push((scalar("0123"), scalar("---"))),
            Self::RemoveFirstEntry if !mapping.is_empty() => {
                mapping.remove(0);
            }
            Self::ReplaceLastValue if !mapping.is_empty() => {
                mapping.last_mut().unwrap().1 = scalar("false");
            }
            _ => {}
        }
    }
}

#[test]
fn nested_edits_match_an_independent_collection_model() {
    use NestedOperation::{
        AppendEntry, AppendItem, FlowItems, FlowMapping, PrependEntry, PrependItem, QuoteLastItem,
        RemoveFirstEntry, RemoveFirstItem, RemoveLastItem, ReplaceFirstItem, ReplaceLastValue,
    };
    let operations = [
        PrependItem,
        AppendItem,
        RemoveFirstItem,
        RemoveLastItem,
        ReplaceFirstItem,
        QuoteLastItem,
        FlowItems,
        PrependEntry,
        AppendEntry,
        RemoveFirstEntry,
        ReplaceLastValue,
        FlowMapping,
    ];
    let sources = [
        indoc! {r"
            - items: [a, b]
              mapping: {key: value}
            - &stable sentinel # keep
            - *stable
        "},
        indoc! {r"
            [{items: [a, b], mapping: {key: value}}, &stable sentinel, *stable] # keep
        "},
        indoc! {r"
            - items:
              - a
              - b
              mapping:
                key: value
            - &stable sentinel
            - *stable # keep
        "},
    ];
    for source in sources {
        let original = EditableDocument::parse(source).unwrap();
        for first in operations {
            for second in operations {
                for third in operations {
                    let sequence = [first, second, third];
                    let mut items = vec![
                        Content::Scalar("a".to_owned()),
                        Content::Scalar("b".to_owned()),
                    ];
                    let mut mapping = vec![(
                        Content::Scalar("key".to_owned()),
                        Content::Scalar("value".to_owned()),
                    )];
                    for operation in sequence {
                        operation.model(&mut items, &mut mapping);
                    }
                    let expected = Content::Sequence(vec![
                        Content::Mapping(vec![
                            (
                                Content::Scalar("items".to_owned()),
                                Content::Sequence(items),
                            ),
                            (
                                Content::Scalar("mapping".to_owned()),
                                Content::Mapping(mapping),
                            ),
                        ]),
                        Content::Scalar("sentinel".to_owned()),
                        Content::Alias("stable".to_owned()),
                    ]);
                    for combined in [false, true] {
                        let mut document = original.clone();
                        let result = if combined {
                            document.edit(|editor| {
                                for operation in sequence {
                                    operation.apply(editor)?;
                                }
                                Ok(())
                            })
                        } else {
                            sequence.into_iter().try_for_each(|operation| {
                                document.edit(|editor| operation.apply(editor))
                            })
                        };
                        result.unwrap_or_else(|error| {
                            panic!("{source:?}, {sequence:?}, combined={combined}: {error}")
                        });
                        assert_eq!(
                            content(&document, document.root()),
                            expected,
                            "{source:?}, {sequence:?}"
                        );
                        check_retained_occurrences(&original, &document);
                    }
                }
            }
        }
    }
}

fn check_retained_occurrences(original: &EditableDocument, document: &EditableDocument) {
    use astral_yaml::edit::NodeKind;
    let NodeKind::Sequence { items, .. } = document.node(document.root()).unwrap().kind() else {
        unreachable!()
    };
    let NodeKind::Sequence {
        items: old_items, ..
    } = original.node(original.root()).unwrap().kind()
    else {
        unreachable!()
    };
    assert_eq!(document.node(items[1]).unwrap().anchor(), Some("stable"));
    assert!(
        matches!(document.node(items[2]).unwrap().kind(), NodeKind::Alias { target, .. } if *target == items[1])
    );
    // The single comment must stay on the same occurrence, even when changed
    // descendants force a parent rewrite.
    let old_nodes = [original.root(), old_items[1], old_items[2]];
    for (old, new) in old_nodes
        .into_iter()
        .zip([document.root(), items[1], items[2]])
    {
        let comments = |document: &EditableDocument, id| {
            document
                .node(id)
                .unwrap()
                .comments()
                .iter()
                .map(|comment| (comment.text().to_owned(), comment.placement()))
                .collect::<Vec<_>>()
        };
        assert_eq!(comments(original, old), comments(document, new));
    }
}

#[test]
fn interleaved_suffixes_remain_ordered_across_longer_sequences() {
    use CommentPlacement::{Detached, Inline, Trailing};
    let source = indoc! {r"
        # child
        [ # open
          old
        ] # close
    "};
    let mut document = EditableDocument::parse(source).unwrap();
    let mut expected = vec![(" open".to_owned(), Inline), (" close".to_owned(), Inline)];
    for (index, placement) in [Inline, Inline, Trailing, Inline, Detached]
        .into_iter()
        .enumerate()
    {
        let text = format!(" added {index}");
        document
            .edit(|editor| {
                let root = editor.root();
                let count = editor.node(&root)?.comments().len();
                editor.insert_comment(&root, count, &text, placement)
            })
            .unwrap();
        expected.push((text, placement));
        assert_eq!(comments(&document), expected);
    }
    document
        .edit(|editor| editor.replace(&editor.root(), "'false'"))
        .unwrap();
    assert_eq!(comments(&document), expected);
    assert_eq!(
        content(&document, document.root()),
        Content::Scalar("false".to_owned())
    );
}
