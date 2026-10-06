//! Nested scalar termination and comment separation are layout dependencies.

use std::fmt::Write;

use astral_yaml::{
    NodeId, ScalarStyle,
    comments::CommentPlacement,
    edit::{EditableDocument, Editor, Handle, NodeKind},
};
use astral_yaml_dev::edit_diff;

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
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        for source in [
            "outer:\n  key: old\n\n# detached\n",
            "outer:\n  key: old\n\n...\n# detached\n",
            "outer:\n  key: old\n\n  # detached\n",
            "outer:\n  key: old\n\n    # detached\nnext: retained\n",
            "outer:\n  key: old\n  # trailing\n\n# detached\n",
            "- - old\n\n  # detached\n",
        ] {
            let source = source.replace('\n', newline);
            let mut document = EditableDocument::parse(&source).unwrap();
            document
                .edit(|editor| editor.replace(&first_leaf(editor), "|+\n  new\n\n"))
                .unwrap();
            diffs.push(edit_diff(&source, document.source()));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,7 @@
     outer:
    -  key: old
    +  key: |2+
    +    new
    +    ⏎
    +...
     ⏎
     # detached

    --- before.yaml
    +++ after.yaml
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

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
     outer:
    -  key: old
    -
    +  key: "new\n\n"
    +  ⏎
       # detached

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     outer:
    -  key: old
    -
    -    # detached
    +  key: "new\n\n"
    +  ⏎
    +  # detached
     next: retained

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,7 @@
     outer:
    -  key: old
    +  key: |2+
    +    new
    +    ⏎
       # trailing
     ⏎
     # detached

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
    -- - old
    -
    +-
    +  - "new\n\n"
    +  ⏎
       # detached

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,7 @@
     outer:␍
    -  key: old␍
    +  key: |2+␍
    +    new␍
    +    ␍
    +...␍
     ␍
     # detached␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,7 @@
     outer:␍
    -  key: old␍
    -␍
    +  key: |2+␍
    +    new␍
    +    ␍
     ...␍
    +␍
     # detached␍

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
     outer:␍
    -  key: old␍
    -␍
    +  key: "new\n\n"␍
    +  ␍
       # detached␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     outer:␍
    -  key: old␍
    -␍
    -    # detached␍
    +  key: "new\n\n"␍
    +  ␍
    +  # detached␍
     next: retained␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,7 @@
     outer:␍
    -  key: old␍
    +  key: |2+␍
    +    new␍
    +    ␍
       # trailing␍
     ␍
     # detached␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
    -- - old␍
    -␍
    +-␍
    +  - "new\n\n"␍
    +  ␍
       # detached␍

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,7 @@
     outer:␍-  key: old␍+  key: |2+␍+    new␍+    ␍+...␍ ␍ # detached␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,7 @@
     outer:␍-  key: old␍-␍+  key: |2+␍+    new␍+    ␍ ...␍+␍ # detached␍

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
     outer:␍-  key: old␍-␍+  key: "new\n\n"␍+  ␍   # detached␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     outer:␍-  key: old␍-␍-    # detached␍+  key: "new\n\n"␍+  ␍+  # detached␍ next: retained␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,7 @@
     outer:␍-  key: old␍+  key: |2+␍+    new␍+    ␍   # trailing␍ ␍ # detached␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
    -- - old␍-␍+-␍+  - "new\n\n"␍+  ␍   # detached␍
    "#);
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn explicit_block_style_requests_still_require_representable_separation() {
    let mut outcomes = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        for style in [ScalarStyle::Literal, ScalarStyle::Folded] {
            for indent in ["", "  "] {
                let source = format!("outer:\n  key: \"new\\n\\n\"\n\n{indent}# detached\n")
                    .replace('\n', newline);
                let mut document = EditableDocument::parse(&source).unwrap();
                let result =
                    document.edit(|editor| editor.set_scalar_style(&first_leaf(editor), style));
                outcomes.push(match result {
                    Ok(()) => edit_diff(&source, document.source()),
                    Err(error) => {
                        assert_eq!(document.source(), source);
                        error.message.to_owned()
                    }
                });
            }
        }
    }
    insta::assert_snapshot!(outcomes.join("\n"), @r#"
    --- before.yaml
    +++ after.yaml
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
    --- before.yaml
    +++ after.yaml
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
    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,7 @@
     outer:␍
    -  key: "new\n\n"␍
    +  key: |2+␍
    +    new␍
    +    ␍
    +...␍
     ␍
     # detached␍

    requested style is not representable in this context
    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,7 @@
     outer:␍
    -  key: "new\n\n"␍
    +  key: >2+␍
    +    new␍
    +    ␍
    +...␍
     ␍
     # detached␍

    requested style is not representable in this context
    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,7 @@
     outer:␍-  key: "new\n\n"␍+  key: |2+␍+    new␍+    ␍+...␍ ␍ # detached␍

    requested style is not representable in this context
    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,7 @@
     outer:␍-  key: "new\n\n"␍+  key: >2+␍+    new␍+    ␍+...␍ ␍ # detached␍

    requested style is not representable in this context
    "#);
}

fn comments(document: &EditableDocument) -> Vec<(Vec<usize>, String, CommentPlacement)> {
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

#[test]
fn scalar_layouts_preserve_values_and_comment_owners_at_each_depth() {
    for newline in ["\n", "\r\n", "\r"] {
        for depth in 1..=4 {
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
                    for ending in ["", "\n", "\n\n", "\n...\n", "\n\n... # end\n"] {
                        let source = format!("{body}{gap}{}# comment{ending}", "  ".repeat(indent))
                            .replace('\n', newline);
                        for fragment in ["|+\n  new\n\n", ">+\n  new\n\n"] {
                            let mut document = EditableDocument::parse(&source).unwrap();
                            let expected_comments = comments(&document);
                            document
                                .edit(|editor| editor.replace(&first_leaf(editor), fragment))
                                .unwrap_or_else(|error| {
                                    panic!("{source:?}, {fragment:?}: {error}")
                                });
                            assert_eq!(comments(&document), expected_comments);
                            let mut id = document.root();
                            for _ in 0..depth {
                                let NodeKind::Mapping { entries, .. } =
                                    document.node(id).unwrap().kind()
                                else {
                                    panic!("expected mapping")
                                };
                                id = entries[0].1;
                            }
                            assert!(
                                matches!(document.node(id).unwrap().kind(), NodeKind::Scalar { value, .. } if value == "new\n\n")
                            );
                        }
                    }
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
            Self::Scalar => editor.replace(&entries[0].1, "|+\n  new\n\n"),
            Self::Sequence => editor.replace(&entries[0].1, "- |+\n    new\n\n"),
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
            Self::AddSibling => editor.insert_entry(&inner, entries.len(), "next", "sibling"),
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
                    "new\n\n"
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

    fn check(&self, document: &EditableDocument) {
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
fn nested_layout_edits_match_an_independent_model_after_each_commit() {
    for newline in ["\n", "\r\n", "\r"] {
        for (source, root, inner) in [
            ("outer:\n  key: old\n", None, None),
            (
                "outer: {key: old}\n\n   # root\n",
                Some((" root", CommentPlacement::Detached)),
                None,
            ),
            (
                "outer: {key: old} # inline\n",
                None,
                Some((" inline", CommentPlacement::Inline)),
            ),
            (
                "outer:\n  key: old\n# trailing\n",
                Some((" trailing", CommentPlacement::Trailing)),
                None,
            ),
            (
                "outer:\n  key: old\n# trailing\n\n",
                Some((" trailing", CommentPlacement::Trailing)),
                None,
            ),
            (
                "outer:\n  key: old\n\n# root\n",
                Some((" root", CommentPlacement::Detached)),
                None,
            ),
            (
                "outer:\n  key: old\n\n  # inner\n",
                None,
                Some((" inner", CommentPlacement::Detached)),
            ),
            (
                "outer:\n  key: old\n  # trailing\n",
                None,
                Some((" trailing", CommentPlacement::Trailing)),
            ),
        ] {
            let source = source.replace('\n', newline);
            let original = EditableDocument::parse(&source).unwrap();
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
                                .edit(|editor| operation.apply(editor))
                                .unwrap_or_else(|error| {
                                    panic!("{source:?}, {sequence:?}: {error}")
                                });
                            model.check(&separate);
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
                        model.check(&combined);
                    }
                }
            }
        }
    }
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn nested_detached_comments_keep_enclosing_trailing_attachments() {
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        for (source, depth) in [
            ("outer:\n  key: old\n# trailing\n", 1),
            ("outer:\n  key: |+\n    old\n\n# trailing\n", 1),
            (
                "outer:\n  inner:\n    key: old\n  # trailing\nnext: retained\n",
                2,
            ),
        ] {
            let source = source.replace('\n', newline);
            let mut document = EditableDocument::parse(&source).unwrap();
            document
                .edit(|editor| {
                    let mut owner = editor.root();
                    for _ in 0..depth {
                        owner = editor.mapping_entries(&owner)?[0].1.clone();
                    }
                    editor.insert_comment(&owner, 0, " detached", CommentPlacement::Detached)
                })
                .unwrap();
            diffs.push(edit_diff(&source, document.source()));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @r#"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,5 @@
    -outer:
    -  key: old
    +outer: {key: old
    +  ⏎
    +  # detached
    +  }
     # trailing

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
    -outer:
    -  key: |+
    -    old
    -
    +outer: {key: "old\n\n"
    +  ⏎
    +  # detached
    +  }
     # trailing

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,7 @@
     outer:
    -  inner:
    -    key: old
    +  inner: {key: old
    +    ⏎
    +    # detached
    +    }
       # trailing
     next: retained

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,5 @@
    -outer:␍
    -  key: old␍
    +outer: {key: old␍
    +  ␍
    +  # detached␍
    +  }␍
     # trailing␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
    -outer:␍
    -  key: |+␍
    -    old␍
    -␍
    +outer: {key: "old\n\n"␍
    +  ␍
    +  # detached␍
    +  }␍
     # trailing␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,7 @@
     outer:␍
    -  inner:␍
    -    key: old␍
    +  inner: {key: old␍
    +    ␍
    +    # detached␍
    +    }␍
       # trailing␍
     next: retained␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,5 @@
    -outer:␍-  key: old␍+outer: {key: old␍+  ␍+  # detached␍+  }␍ # trailing␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
    -outer:␍-  key: |+␍-    old␍-␍+outer: {key: "old\n\n"␍+  ␍+  # detached␍+  }␍ # trailing␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,7 @@
     outer:␍-  inner:␍-    key: old␍+  inner: {key: old␍+    ␍+    # detached␍+    }␍   # trailing␍ next: retained␍
    "#);
}

#[test]
fn required_collection_boundaries_do_not_override_explicit_styles() {
    let source = "outer:\n  key: old\n# trailing\n";
    let mut document = EditableDocument::parse(source).unwrap();
    let error = document
        .edit(|editor| {
            let inner = editor.mapping_entries(&editor.root())?[0].1.clone();
            editor.insert_comment(&inner, 0, " detached", CommentPlacement::Detached)?;
            editor.set_collection_style(&inner, astral_yaml::CollectionStyle::Block)
        })
        .unwrap_err();
    assert_eq!(document.source(), source);
    insta::assert_snapshot!(error.message, @"requested style is not representable in this context");
}

#[test]
fn replacing_a_tail_and_removing_its_boundary_share_the_final_layout() {
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        let source = "outer:\n  key: old\n# trailing\n\n".replace('\n', newline);
        let mut document = EditableDocument::parse(&source).unwrap();
        document
            .edit(|editor| {
                editor.remove_comment(&editor.root(), 0)?;
                editor.replace(&first_leaf(editor), "|+\n  new\n\n")
            })
            .unwrap();
        diffs.push(edit_diff(&source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
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

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,7 @@
     outer:␍
    -  key: old␍
    -# trailing␍
    +  key: |2+␍
    +    new␍
    +    ␍
    +...␍
    +␍
     ␍

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,7 @@
     outer:␍-  key: old␍-# trailing␍+  key: |2+␍+    new␍+    ␍+...␍+␍ ␍
    ");
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn block_rewrites_keep_indented_suffixes_at_their_root_owner() {
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        for source in [
            "{outer: {key: old}} # inline\n\n   # root\n",
            "outer: {key: old}\n\n   # root\n",
            "[{key: old}]\n\n   # root\n",
        ] {
            let source = source.replace('\n', newline);
            let mut document = EditableDocument::parse(&source).unwrap();
            let expected_comments = comments(&document);
            document
                .edit(|editor| {
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
                        editor
                            .set_collection_style(&handle, astral_yaml::CollectionStyle::Block)?;
                    }
                    Ok(())
                })
                .unwrap();
            assert_eq!(comments(&document), expected_comments);
            diffs.push(edit_diff(&source, document.source()));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,5 @@
    -{outer: {key: old}} # inline
    +--- # inline
    +outer:
    +  key: old
     ⏎
    -   # root
    +# root

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
    -outer: {key: old}
    +outer:
    +  key: old
     ⏎
    -   # root
    +# root

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
    -[{key: old}]
    +-
    +  key: old
     ⏎
    -   # root
    +# root

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,5 @@
    -{outer: {key: old}} # inline␍
    +--- # inline␍
    +outer:␍
    +  key: old␍
     ␍
    -   # root␍
    +# root␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
    -outer: {key: old}␍
    +outer:␍
    +  key: old␍
     ␍
    -   # root␍
    +# root␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
    -[{key: old}]␍
    +-␍
    +  key: old␍
     ␍
    -   # root␍
    +# root␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,5 @@
    -{outer: {key: old}} # inline␍+--- # inline␍+outer:␍+  key: old␍ ␍-   # root␍+# root␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
    -outer: {key: old}␍+outer:␍+  key: old␍ ␍-   # root␍+# root␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
    -[{key: old}]␍+-␍+  key: old␍ ␍-   # root␍+# root␍
    ");
}
