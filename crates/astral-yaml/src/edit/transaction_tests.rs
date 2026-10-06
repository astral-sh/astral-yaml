use crate::CollectionStyle;
use astral_yaml_dev::edit_diff;

use super::{EditableDocument, Editor, NodeKind, Span, same_style};

#[test]
fn overlapping_rewrite_regions_expand_to_their_common_ancestor() {
    let mut document = EditableDocument::parse("[[a, b], c]").unwrap();
    let first = crate::NodeId(
        document
            .nodes
            .iter()
            .position(|node| matches!(&node.kind, NodeKind::Scalar { value, .. } if value == "a"))
            .unwrap(),
    );
    let last = crate::NodeId(
        document
            .nodes
            .iter()
            .position(|node| matches!(&node.kind, NodeKind::Scalar { value, .. } if value == "c"))
            .unwrap(),
    );
    // The patch planner must coalesce overlapping retained ranges, including
    // ranges expanded by trivia ownership, before slicing the original source.
    document.nodes[first.0].span = Span::new(2, 10).unwrap();
    let mut editor = Editor::new(&document);
    editor.rewrites.extend([first, last]);
    assert_eq!(
        editor.rewrite_regions().unwrap(),
        [(document.root, document.nodes[document.root.0].span)]
    );
}

#[test]
fn empty_transactions_do_not_require_rewrite_work() {
    let document = EditableDocument::parse("one").unwrap();
    let editor = Editor::new(&document);
    editor.work.set(0);
    assert_eq!(editor.rewrite_regions().unwrap(), []);
}

#[test]
fn outer_comment_plans_obey_work_and_output_limits() {
    let source = "        # old\nvalue\n";
    let mut document = EditableDocument::parse(source).unwrap();
    document.options.max_input_bytes = source.len();
    let mut editor = Editor::new(&document);
    for index in 0..3 {
        editor
            .insert_comment(
                &editor.root(),
                index,
                " new",
                crate::comments::CommentPlacement::Leading,
            )
            .unwrap();
    }
    let region = document.nodes[document.root.0].span;
    assert_eq!(
        editor.root_comments(Some(region)).unwrap_err().message,
        "emitted YAML exceeds input byte limit"
    );
    editor.work.set(0);
    assert_eq!(
        editor.root_comments(Some(region)).unwrap_err().message,
        "editing resource limit exceeded"
    );
}

#[test]
fn requested_style_comparison_checks_kind_and_collection_contents() {
    for (left, right, expected) in [
        ("one", "one", true),
        ("one", "'one'", false),
        ("one", "[one]", false),
        ("[]", "[]", true),
        ("[one]", "- one", false),
        ("{}", "{}", true),
        ("{one: two}", "one: two", false),
    ] {
        let left = EditableDocument::parse(left).unwrap();
        let right = EditableDocument::parse(right).unwrap();
        assert_eq!(
            same_style(
                &left.nodes[left.root.0].kind,
                &right.nodes[right.root.0].kind
            ),
            expected
        );
    }
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn collection_rewrites_include_their_comment_context() {
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        for source in [
            "outer:\n# detached\n  key: old\n",
            "outer:\n  # detached\n    key: old\n",
            "outer:\n# leading\n  {key: old}\n",
            "outer:\n# leading\n  [old]\n",
            "outer:\n  [old] # inline\n",
            "outer:\n# detached\n  - old\n",
        ] {
            let source = source.replace('\n', newline);
            let mut document = EditableDocument::parse(&source).unwrap();
            document
                .edit(|editor| {
                    let target = editor.mapping_entries(&editor.root())?[0].1.clone();
                    editor.set_collection_style(&target, CollectionStyle::Flow)?;
                    match editor.node(&target)?.kind() {
                        NodeKind::Sequence { .. } => editor.insert_item(&target, 1, "new"),
                        _ => editor.insert_entry(&target, 1, "added", "new"),
                    }
                })
                .unwrap();
            diffs.push(edit_diff(&source, document.source()));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     outer:
    -# detached
    -  key: old
    +  # detached
    +
    +  {key: old, added: new}

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     outer:
       # detached
    -    key: old
    +
    +  {key: old, added: new}

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
    -outer:
    -# leading
    -  {key: old}
    +outer: {
    +  # leading
    +  key: old, added: new}

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
    -outer:
    -# leading
    -  [old]
    +outer: [
    +  # leading
    +  old, new]

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1 @@
    -outer:
    -  [old] # inline
    +outer: [old, new] # inline

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     outer:
    -# detached
    -  - old
    +  # detached
    +
    +  [old, new]

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     outer:␍
    -# detached␍
    -  key: old␍
    +  # detached␍
    +␍
    +  {key: old, added: new}␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     outer:␍
       # detached␍
    -    key: old␍
    +␍
    +  {key: old, added: new}␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
    -outer:␍
    -# leading␍
    -  {key: old}␍
    +outer: {␍
    +  # leading␍
    +  key: old, added: new}␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
    -outer:␍
    -# leading␍
    -  [old]␍
    +outer: [␍
    +  # leading␍
    +  old, new]␍

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1 @@
    -outer:␍
    -  [old] # inline␍
    +outer: [old, new] # inline␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     outer:␍
    -# detached␍
    -  - old␍
    +  # detached␍
    +␍
    +  [old, new]␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     outer:␍-# detached␍-  key: old␍+  # detached␍+␍+  {key: old, added: new}␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     outer:␍   # detached␍-    key: old␍+␍+  {key: old, added: new}␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
    -outer:␍-# leading␍-  {key: old}␍+outer: {␍+  # leading␍+  key: old, added: new}␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,3 @@
    -outer:␍-# leading␍-  [old]␍+outer: [␍+  # leading␍+  old, new]␍

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1 @@
    -outer:␍-  [old] # inline␍+outer: [old, new] # inline␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     outer:␍-# detached␍-  - old␍+  # detached␍+␍+  [old, new]␍
    ");
}
