//! Nested scalar termination and comment separation are layout dependencies.

use test_case::test_case;

use super::support::{edit, edits};
use astral_yaml_dev::{edit_diff, formatdoc, indoc};
use std::fmt::Write;

use crate::syntax::{DocumentSyntax, Editor, Handle, NodeKind};
use astral_yaml::{NodeId, ScalarStyle, comments::CommentPlacement};

fn first_leaf(editor: &Editor<'_>) -> Handle {
    let mut handle = editor.root();
    loop {
        handle = match editor.node(&handle).unwrap().kind() {
            NodeKind::Mapping { .. } => editor.mapping_entries(&handle).unwrap().remove(0).1,
            NodeKind::Sequence { .. } => editor.sequence_items(&handle).unwrap().remove(0),
            _ => return handle,
        };
    }
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn descendant_edits_preserve_enclosing_comment_boundaries() {
    // Blank separation belongs outside scalar content:
    // https://yaml.org/spec/1.2.2/#8112-block-chomping-indicator
    let diffs = edits(
        [
            indoc! {r"
                outer:
                  key: old

                # detached
            "},
            indoc! {r"
                outer:
                  key: old

                ...
                # detached
            "},
            indoc! {r"
                outer:
                  key: old

                  # detached
            "},
            indoc! {r"
                outer:
                  key: old

                    # detached
                next: retained
            "},
            indoc! {r"
                outer:
                  key: old
                  # trailing

                # detached
            "},
            indoc! {r"
                - - old

                  # detached
            "},
        ],
        |editor| {
            editor.replace(
                &first_leaf(editor),
                indoc! {r"
                    |+
                      new

                "},
            )
        },
    );
    insta::assert_snapshot!(diffs, @r#"
    @@ -1,4 +1,7 @@
     outer:
    -  key: old
    +  key: |2+
    +    new
    +    ⏎
    +...
     ⏎
     # detached

    @@ -1,5 +1,7 @@
     outer:
    -  key: old
    -
    +  key: |2+
    +    new
    +    ⏎
     ...
    +
     # detached

    @@ -1,4 +1,4 @@
     outer:
    -  key: old
    -
    +  key: "new\n\n"
    +  ⏎
       # detached

    @@ -1,5 +1,5 @@
     outer:
    -  key: old
    -
    -    # detached
    +  key: "new\n\n"
    +  ⏎
    +  # detached
     next: retained

    @@ -1,5 +1,7 @@
     outer:
    -  key: old
    +  key: |2+
    +    new
    +    ⏎
       # trailing
     ⏎
     # detached

    @@ -1,3 +1,4 @@
    -- - old
    -
    +-
    +  - "new\n\n"
    +  ⏎
       # detached
    "#);
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn explicit_block_style_requests_still_require_representable_separation() {
    let mut outcomes = Vec::new();
    for style in [ScalarStyle::Literal, ScalarStyle::Folded] {
        for indent in ["", "  "] {
            let source = formatdoc! {r#"
                outer:
                  key: "new\n\n"

                {indent}# detached
            "#};
            let mut document = DocumentSyntax::parse(&source).unwrap();
            let result =
                document.edit_syntax(|editor| editor.set_scalar_style(&first_leaf(editor), style));
            outcomes.push(match result {
                Ok(()) => edit_diff(&source, document.source()),
                Err(error) => {
                    assert_eq!(document.source(), source);
                    error.message.to_owned()
                }
            });
        }
    }
    insta::assert_snapshot!(outcomes.join("\n"), @r#"
    @@ -1,4 +1,7 @@
     outer:
    -  key: "new\n\n"
    +  key: |2+
    +    new
    +    ⏎
    +...
     ⏎
     # detached

    requested style is not representable in this context
    @@ -1,4 +1,7 @@
     outer:
    -  key: "new\n\n"
    +  key: >2+
    +    new
    +    ⏎
    +...
     ⏎
     # detached

    requested style is not representable in this context
    "#);
}

fn comments(document: &DocumentSyntax) -> Vec<(Vec<usize>, String, CommentPlacement)> {
    let mut comments = Vec::new();
    let mut pending = vec![(document.root(), Vec::new())];
    while let Some((id, path)) = pending.pop() {
        let node = document.node(id).unwrap();
        comments.extend(
            node.comments()
                .iter()
                .map(|comment| (path.clone(), comment.text().to_owned(), comment.placement())),
        );
        let children: Vec<NodeId> = match node.kind() {
            NodeKind::Mapping { entries, .. } => {
                entries.iter().flat_map(|&(k, v)| [k, v]).collect()
            }
            NodeKind::Sequence { items, .. } => items.clone(),
            _ => Vec::new(),
        };
        for (index, child) in children.into_iter().enumerate() {
            let mut path = path.clone();
            path.push(index);
            pending.push((child, path));
        }
    }
    comments
}

#[test_case(1)]
#[test_case(2)]
#[test_case(3)]
#[test_case(4)]
fn scalar_layouts_preserve_values_and_comment_owners_at_each_depth(depth: usize) {
    let mut body = String::new();
    for n in 0..depth {
        write!(
            body,
            "{}key{n}:{}",
            "  ".repeat(n),
            if n + 1 == depth { " old" } else { "\n" }
        )
        .unwrap();
    }
    for gap in ["\n", "\n\n", "\n \n", "\n\t\n"] {
        for indent in 0..=depth {
            for ending in [
                "",
                "\n",
                "\n\n",
                indoc! {r"

                    ...
                "},
                indoc! {r"


                    ... # end
                "},
            ] {
                let source = format!("{body}{gap}{}# comment{ending}", "  ".repeat(indent));
                for fragment in [
                    indoc! {r"
                        |+
                          new

                    "},
                    indoc! {r"
                        >+
                          new

                    "},
                ] {
                    let mut document = DocumentSyntax::parse(&source).unwrap();
                    let expected_comments = comments(&document);
                    document
                        .edit_syntax(|editor| editor.replace(&first_leaf(editor), fragment))
                        .unwrap_or_else(|error| panic!("{source:?}, {fragment:?}: {error}"));
                    assert_eq!(comments(&document), expected_comments);
                    let mut id = document.root();
                    for _ in 0..depth {
                        let NodeKind::Mapping { entries, .. } = document.node(id).unwrap().kind()
                        else {
                            panic!("expected mapping")
                        };
                        id = entries[0].1;
                    }
                    assert!(
                        matches!(document.node(id).unwrap().kind(), NodeKind::Scalar { value, .. } if value == indoc! {r"
                            new

                        "})
                    );
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Operation {
    Scalar,
    Sequence,
    Plain,
    RootComment,
    InnerComment,
    RemoveRootComment,
    RemoveInnerComment,
    AddSibling,
    RemoveSibling,
    Flow,
    Block,
}

impl Operation {
    const ALL: [Self; 11] = [
        Self::Scalar,
        Self::Sequence,
        Self::Plain,
        Self::RootComment,
        Self::InnerComment,
        Self::RemoveRootComment,
        Self::RemoveInnerComment,
        Self::AddSibling,
        Self::RemoveSibling,
        Self::Flow,
        Self::Block,
    ];

    fn apply(self, editor: &mut Editor<'_>) -> Result<(), astral_yaml::Error> {
        let root = editor.root();
        let inner = editor.mapping_entries(&root)?[0].1.clone();
        let entries = editor.mapping_entries(&inner)?;
        match self {
            Self::Scalar => editor.replace(
                &entries[0].1,
                indoc! {r"
                    |+
                      new

                "},
            ),
            Self::Sequence => editor.replace(
                &entries[0].1,
                indoc! {r"
                    - |+
                        new

                "},
            ),
            Self::Plain => editor.replace(&entries[0].1, "old"),
            Self::RootComment | Self::InnerComment => {
                let owner = if matches!(self, Self::RootComment) {
                    root
                } else {
                    inner
                };
                editor.insert_comment(
                    &owner,
                    editor.node(&owner)?.comments().len(),
                    " added",
                    CommentPlacement::Detached,
                )
            }
            Self::RemoveRootComment | Self::RemoveInnerComment => {
                let owner = if matches!(self, Self::RemoveRootComment) {
                    root
                } else {
                    inner
                };
                if editor.node(&owner)?.comments().is_empty() {
                    Ok(())
                } else {
                    editor.remove_comment(&owner, 0)
                }
            }
            Self::AddSibling => editor
                .insert_entry(&inner, entries.len(), "next", "sibling")
                .map(|_| ()),
            Self::RemoveSibling if entries.len() > 1 => {
                editor.remove_entry(&inner, entries.len() - 1)
            }
            Self::Flow => editor.set_collection_style(&inner, astral_yaml::CollectionStyle::Flow),
            Self::Block => editor.set_collection_style(&inner, astral_yaml::CollectionStyle::Block),
            Self::RemoveSibling => Ok(()),
        }
    }
}

#[derive(Clone)]
struct Model {
    value: &'static str,
    sequence: bool,
    siblings: usize,
    comments: [Vec<(String, CommentPlacement)>; 2],
}

impl Model {
    fn apply(&mut self, operation: Operation) {
        match operation {
            Operation::Scalar | Operation::Sequence | Operation::Plain => {
                self.value = if matches!(operation, Operation::Plain) {
                    "old"
                } else {
                    indoc! {r"
                        new

                    "}
                };
                self.sequence = matches!(operation, Operation::Sequence);
            }
            Operation::AddSibling => self.siblings += 1,
            Operation::RemoveSibling => self.siblings = self.siblings.saturating_sub(1),
            Operation::RootComment | Operation::InnerComment => {
                let index = usize::from(matches!(operation, Operation::InnerComment));
                self.comments[index].push((" added".to_owned(), CommentPlacement::Detached));
            }
            Operation::RemoveRootComment | Operation::RemoveInnerComment => {
                let index = usize::from(matches!(operation, Operation::RemoveInnerComment));
                if !self.comments[index].is_empty() {
                    self.comments[index].remove(0);
                }
            }
            Operation::Flow | Operation::Block => {}
        }
    }

    fn check(&self, document: &DocumentSyntax) {
        let mapping = |id| match document.node(id).unwrap().kind() {
            NodeKind::Mapping { entries, .. } => entries,
            _ => panic!("expected mapping"),
        };
        let scalar = |id| match document.node(id).unwrap().kind() {
            NodeKind::Scalar { value, .. } => value.as_str(),
            _ => panic!("expected scalar"),
        };
        let root = mapping(document.root());
        assert_eq!(root.len(), 1);
        assert_eq!(scalar(root[0].0), "outer");
        let inner = mapping(root[0].1);
        assert_eq!(inner.len(), self.siblings + 1);
        assert_eq!(scalar(inner[0].0), "key");
        let value = if self.sequence {
            let NodeKind::Sequence { items, .. } = document.node(inner[0].1).unwrap().kind() else {
                panic!("expected sequence")
            };
            assert_eq!(items.len(), 1);
            items[0]
        } else {
            inner[0].1
        };
        assert_eq!(scalar(value), self.value);
        for &(key, value) in &inner[1..] {
            assert_eq!((scalar(key), scalar(value)), ("next", "sibling"));
        }
        for (id, expected) in [document.root(), root[0].1].into_iter().zip(&self.comments) {
            let actual: Vec<_> = document
                .node(id)
                .unwrap()
                .comments()
                .iter()
                .map(|comment| (comment.text().to_owned(), comment.placement()))
                .collect();
            assert_eq!(&actual, expected);
        }
    }
}

#[test]
#[allow(clippy::too_many_lines, reason = "multiline YAML fixtures")]
fn nested_layout_edits_match_an_independent_model_after_each_commit() {
    for (source, root, inner) in [
        (
            indoc! {r"
                outer:
                  key: old
            "},
            None,
            None,
        ),
        (
            indoc! {r"
                outer: {key: old}

                   # root
            "},
            Some((" root", CommentPlacement::Detached)),
            None,
        ),
        (
            indoc! {r"
                outer: {key: old} # inline
            "},
            None,
            Some((" inline", CommentPlacement::Inline)),
        ),
        (
            indoc! {r"
                outer:
                  key: old
                # trailing
            "},
            Some((" trailing", CommentPlacement::Trailing)),
            None,
        ),
        (
            indoc! {r"
                outer:
                  key: old
                # trailing

            "},
            Some((" trailing", CommentPlacement::Trailing)),
            None,
        ),
        (
            indoc! {r"
                outer:
                  key: old

                # root
            "},
            Some((" root", CommentPlacement::Detached)),
            None,
        ),
        (
            indoc! {r"
                outer:
                  key: old

                  # inner
            "},
            None,
            Some((" inner", CommentPlacement::Detached)),
        ),
        (
            indoc! {r"
                outer:
                  key: old
                  # trailing
            "},
            None,
            Some((" trailing", CommentPlacement::Trailing)),
        ),
    ] {
        let original = DocumentSyntax::parse(source).unwrap();
        let initial = Model {
            value: "old",
            sequence: false,
            siblings: 0,
            comments: [root, inner].map(|comment| {
                comment
                    .into_iter()
                    .map(|(text, placement)| (text.to_owned(), placement))
                    .collect()
            }),
        };
        // A nested block-style request cannot enclose a detached suffix
        // while an enclosing trailing attachment requires adjacency.
        let operations =
            if root.is_some_and(|(_, placement)| placement == CommentPlacement::Trailing) {
                &Operation::ALL[..10]
            } else {
                &Operation::ALL[..]
            };
        for &first in operations {
            for &second in operations {
                for &third in operations {
                    let sequence = [first, second, third];
                    let mut model = initial.clone();
                    let mut separate = original.clone();
                    for operation in sequence {
                        model.apply(operation);
                        separate
                            .edit_syntax(|editor| operation.apply(editor))
                            .unwrap_or_else(|error| panic!("{source:?}, {sequence:?}: {error}"));
                        model.check(&separate);
                    }
                    let mut combined = original.clone();
                    combined
                        .edit_syntax(|editor| {
                            for operation in sequence {
                                operation.apply(editor)?;
                            }
                            Ok(())
                        })
                        .unwrap_or_else(|error| panic!("{source:?}, {sequence:?}: {error}"));
                    model.check(&combined);
                }
            }
        }
    }
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn nested_detached_comments_keep_enclosing_trailing_attachments() {
    let mut diffs = Vec::new();
    for (source, depth) in [
        (
            indoc! {r"
                outer:
                  key: old
                # trailing
            "},
            1,
        ),
        (
            indoc! {r"
                outer:
                  key: |+
                    old

                # trailing
            "},
            1,
        ),
        (
            indoc! {r"
                outer:
                  inner:
                    key: old
                  # trailing
                next: retained
            "},
            2,
        ),
    ] {
        diffs.push(edit(source, |editor| {
            let mut owner = editor.root();
            for _ in 0..depth {
                owner = editor.mapping_entries(&owner)?[0].1.clone();
            }
            editor.insert_comment(&owner, 0, " detached", CommentPlacement::Detached)
        }));
    }
    insta::assert_snapshot!(diffs.join("\n"), @r#"
    @@ -1,3 +1,5 @@
    -outer:
    -  key: old
    +outer: {key: old,
    +  ⏎
    +  # detached
    +  }
     # trailing

    @@ -1,5 +1,5 @@
    -outer:
    -  key: |+
    -    old
    -
    +outer: {key: "old\n\n",
    +  ⏎
    +  # detached
    +  }
     # trailing

    @@ -1,5 +1,7 @@
     outer:
    -  inner:
    -    key: old
    +  inner: {key: old,
    +    ⏎
    +    # detached
    +    }
       # trailing
     next: retained
    "#);
}

#[test]
fn required_collection_boundaries_do_not_override_explicit_styles() {
    for (source, index, placement) in [
        (
            indoc! {r"
                outer:
                  key: old
                # trailing
            "},
            0,
            CommentPlacement::Detached,
        ),
        (
            indoc! {r"
                &root
                outer:
                  # collection
                    key: value
            "},
            1,
            CommentPlacement::Inline,
        ),
    ] {
        let mut document = DocumentSyntax::parse(source).unwrap();
        let error = document
            .edit_syntax(|editor| {
                let inner = editor.mapping_entries(&editor.root())?[0].1.clone();
                editor.insert_comment(&inner, index, " added", placement)?;
                editor.set_collection_style(&inner, astral_yaml::CollectionStyle::Block)
            })
            .unwrap_err();
        assert_eq!(document.source(), source);
        insta::allow_duplicates! {
            insta::assert_snapshot!(error.message, @"requested style is not representable in this context");
        }
    }
}

#[test]
fn replacing_a_tail_and_removing_its_boundary_share_the_final_layout() {
    let source = indoc! {r"
        outer:
          key: old
        # trailing

    "};
    let diff = edit(source, |editor| {
        editor.remove_comment(&editor.root(), 0)?;
        editor.replace(
            &first_leaf(editor),
            indoc! {r"
                |+
                  new

            "},
        )
    });
    insta::assert_snapshot!(diff, @"
    @@ -1,4 +1,7 @@
     outer:
    -  key: old
    -# trailing
    +  key: |2+
    +    new
    +    ⏎
    +...
    +
     ⏎
    ");
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn block_rewrites_keep_indented_suffixes_at_their_root_owner() {
    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
            {outer: {key: old}} # inline

               # root
        "},
        indoc! {r"
            outer: {key: old}

               # root
        "},
        indoc! {r"
            [{key: old}]

               # root
        "},
    ] {
        let mut document = DocumentSyntax::parse(source).unwrap();
        let expected_comments = comments(&document);
        document
            .edit_syntax(|editor| {
                let mut pending = vec![editor.root()];
                while let Some(handle) = pending.pop() {
                    match editor.node(&handle)?.kind() {
                        NodeKind::Mapping { .. } => {
                            pending.extend(
                                editor.mapping_entries(&handle)?.into_iter().map(|(_, v)| v),
                            );
                        }
                        NodeKind::Sequence { .. } => {
                            pending.extend(editor.sequence_items(&handle)?);
                        }
                        _ => continue,
                    }
                    editor.set_collection_style(&handle, astral_yaml::CollectionStyle::Block)?;
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(comments(&document), expected_comments);
        diffs.push(edit_diff(source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    @@ -1,3 +1,5 @@
    -{outer: {key: old}} # inline
    +--- # inline
    +outer:
    +  key: old
     ⏎
    -   # root
    +# root

    @@ -1,3 +1,4 @@
    -outer: {key: old}
    +outer:
    +  key: old
     ⏎
    -   # root
    +# root

    @@ -1,3 +1,4 @@
    -[{key: old}]
    +-
    +  key: old
     ⏎
    -   # root
    +# root
    ");
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn retained_end_markers_bound_removed_suffix_comments() {
    let mut diffs = Vec::new();
    for source in [
        indoc! {r"
            ---
            {outer: {key: old}} # inline
            ...

            # detached
        "},
        indoc! {r"
            {outer: {key: old}} # inline
            # trailing

        "},
        indoc! {r"
            {outer: {key: old}} # inline
            # trailing
            ...
        "},
        indoc! {r"
            {outer: {key: old}}
            ... # inline
            # trailing
        "},
        indoc! {r"
            {outer: {key: old}}
            ...

            # detached
            ...
        "},
    ] {
        diffs.push(edit(source, |editor| {
            let root = editor.root();
            let inner = editor.mapping_entries(&root)?[0].1.clone();
            let leaf = editor.mapping_entries(&inner)?[0].1.clone();
            editor.remove_comment(&root, editor.node(&root)?.comments().len() - 1)?;
            editor.set_collection_style(&root, astral_yaml::CollectionStyle::Block)?;
            editor.set_collection_style(&inner, astral_yaml::CollectionStyle::Block)?;
            editor.replace(
                &leaf,
                indoc! {r"
                    |+
                      new

                "},
            )
        }));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    @@ -1,5 +1,7 @@
    ----
    -{outer: {key: old}} # inline
    +--- # inline
    +outer:
    +  key: |2+
    +    new
    +    ⏎
     ...
     ⏎
    -# detached

    @@ -1,3 +1,8 @@
    -{outer: {key: old}} # inline
    -# trailing
    +--- # inline
    +outer:
    +  key: |2+
    +    new
    +    ⏎
    +...
    +
     ⏎

    @@ -1,3 +1,8 @@
    -{outer: {key: old}} # inline
    -# trailing
    +--- # inline
    +outer:
    +  key: |2+
    +    new
    +    ⏎
    +...
    +
     ...

    @@ -1,3 +1,5 @@
    -{outer: {key: old}}
    +outer:
    +  key: |2+
    +    new
    +    ⏎
     ... # inline
    -# trailing

    @@ -1,5 +1,7 @@
    -{outer: {key: old}}
    +outer:
    +  key: |2+
    +    new
    +    ⏎
     ...
     ⏎
    -# detached
     ...
    ");
}

#[test]
fn retained_trailing_comments_stay_adjacent_when_replacing_a_tail() {
    let diffs = edits(
        [
            indoc! {r"
                old
                # remove
                # tail
            "},
            indoc! {r"
                outer:
                  key: old
                # remove
                # tail
            "},
            indoc! {r"
                - - old
                # remove
                # tail
            "},
            indoc! {r"
                outer:
                  key: old
                # remove
                ... # boundary
                # tail
            "},
        ],
        |editor| {
            let root = editor.root();
            editor.remove_comment(&root, 0)?;
            editor.replace(
                &first_leaf(editor),
                indoc! {r"
                    |+
                      new

                "},
            )
        },
    );
    insta::assert_snapshot!(diffs, @"
    @@ -1,3 +1,5 @@
    -old
    -# remove
    +|3+
    +  new
    +  ⏎
    +...
     # tail

    @@ -1,4 +1,6 @@
     outer:
    -  key: old
    -# remove
    +  key: |2+
    +    new
    +    ⏎
    +...
     # tail

    @@ -1,3 +1,6 @@
    -- - old
    -# remove
    +-
    +  - |2+
    +    new
    +    ⏎
    +...
     # tail

    @@ -1,5 +1,7 @@
     outer:
    -  key: old
    -# remove
    +  key: |2+
    +    new
    +    ⏎
    +...
     ... # boundary
     # tail
    ");
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn generated_suffix_boundaries_follow_descendant_comment_regions() {
    use crate::syntax::NodeKind;

    let diffs = edits(
        [
            indoc! {r"
                outer:
                  value: old

                  # nested

                # keep

                # remove
            "},
            indoc! {r"
                -
                  - old

                  # nested

                # keep

                # remove
            "},
            indoc! {r"
                outer:
                  inner:
                    value: old

                    # nested

                # keep

                # remove
            "},
            indoc! {r"
                outer:
                  value: old

                  # nested
                ...

                # keep

                # remove
            "},
        ],
        |editor| {
            let root = editor.root();
            editor.remove_comment(&root, 1)?;
            let mut node = root.clone();
            loop {
                node = match editor.node(&node)?.kind() {
                    NodeKind::Mapping { .. } => editor.mapping_entries(&node)?[0].1.clone(),
                    NodeKind::Sequence { .. } => editor.sequence_items(&node)?[0].clone(),
                    _ => break,
                };
                while !editor.node(&node)?.comments().is_empty() {
                    editor.remove_comment(&node, 0)?;
                }
            }
            editor.replace(
                &node,
                indoc! {r"
                    |+
                      new

                "},
            )
        },
    );
    insta::assert_snapshot!(diffs, @r#"
    @@ -1,8 +1,9 @@
     outer:
    -  value: old
    +  value: |2+
    +    new
    +    ⏎
    +...
     ⏎
    -  # nested
     ⏎
     # keep
     ⏎
    -# remove

    @@ -1,8 +1,9 @@
     -
    -  - old
    +  - |2+
    +    new
    +    ⏎
    +...
     ⏎
    -  # nested
     ⏎
     # keep
     ⏎
    -# remove

    @@ -1,9 +1,10 @@
     outer:
       inner:
    -    value: old
    +    value: |2+
    +      new
    +      ⏎
    +...
     ⏎
    -    # nested
     ⏎
     # keep
     ⏎
    -# remove

    @@ -1,9 +1,8 @@
     outer:
    -  value: old
    -
    -  # nested
    +  value: |2+
    +    new
    +    ⏎
     ...
     ⏎
     # keep
     ⏎
    -# remove
    "#);
}
