//! Transaction composition against an independently maintained comment model.

use astral_yaml::{
    Error,
    comments::CommentPlacement,
    edit::{EditableDocument, Editor},
};

#[derive(Clone, Copy, Debug)]
enum Operation {
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
            Self::SetSequence => editor.replace(&root, "- new\n"),
            Self::SetScalar => editor.replace(&root, "value"),
            Self::SetMapping => editor.replace(&root, "key: new\n"),
            Self::SetFlowSequence => editor.replace(&root, "[new]"),
            Self::SetLiteral => editor.replace(&root, "|+\n  new\n\n"),
            Self::SetEmpty => editor.replace(&root, "---\n"),
            _ => Ok(()),
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
    // https://yaml.org/spec/1.2.2/#922-document-markers
    // Every case must succeed. Rejections are failures, not discarded samples.
    use CommentPlacement::{Inline, Leading, Trailing};
    use Operation::{
        AppendTrailing, PrependLeading, RemoveFirst, RemoveLast, ReplaceFirst, ReplaceLast,
        SetEmpty, SetFlowSequence, SetLiteral, SetMapping, SetScalar, SetSequence,
    };
    let operations = [
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
    let cases: &[(&str, &[(&str, CommentPlacement)])] = &[
        (
            "# prefix\n---\nold\n... # end\n# tail\n",
            &[(" prefix", Leading), (" end", Inline), (" tail", Trailing)],
        ),
        ("---\n...\n", &[]),
        (
            "---\n# prefix\nold\n...\n# tail\n",
            &[(" prefix", Leading), (" tail", Trailing)],
        ),
        ("---\nold # inline\n...\n", &[(" inline", Inline)]),
        ("---\nold\n# tail\n...\n", &[(" tail", Trailing)]),
        (
            "# prefix\n---\n&root [*root]\n...\n# tail\n",
            &[(" prefix", Leading), (" tail", Trailing)],
        ),
        (
            "# prefix\n---\n- old\n...\n# tail\n",
            &[(" prefix", Leading), (" tail", Trailing)],
        ),
    ];
    for newline in ["\n", "\r\n", "\r"] {
        for &(source, initial) in cases {
            let source = source.replace('\n', newline);
            let original = EditableDocument::parse(&source).unwrap();
            for first in operations {
                for second in operations {
                    for third in operations {
                        let sequence = [first, second, third];
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
                        for operation in sequence {
                            separate
                                .edit(|editor| operation.apply(editor))
                                .unwrap_or_else(|error| {
                                    panic!("{source:?}, {sequence:?}: {error}")
                                });
                        }
                        assert_eq!(comments(&separate), expected, "{source:?}, {sequence:?}");
                        // The final syntax kind and decoded content must also
                        // agree; the comparison ignores comments and incidental whitespace.
                        let semantic = |document: &EditableDocument| {
                            let stream = astral_yaml::parse(document.source()).unwrap();
                            astral_yaml_dev::events(&stream)
                        };
                        assert_eq!(
                            semantic(&combined),
                            semantic(&separate),
                            "{source:?}, {sequence:?}"
                        );
                    }
                }
            }
        }
    }
}
