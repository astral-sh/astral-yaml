//! Comments and separation: <https://yaml.org/spec/1.2.2/#66-comments>.

use astral_yaml::{
    CollectionStyle,
    edit::{EditableDocument, Editor, Handle, NodeKind},
};
use astral_yaml_dev::edit_diff;

fn collection(editor: &Editor<'_>, nested: bool) -> Handle {
    if nested {
        editor.mapping_entries(&editor.root()).unwrap()[0].1.clone()
    } else {
        editor.root()
    }
}

#[test]
fn flow_delimiter_comments_survive_collection_edits() {
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        for (source, nested) in [
            ("[ # open\n  old\n] # close\n", false),
            ("{ # open\n  key: old\n} # close\n", false),
            ("outer: [ # open\n  old\n  ] # close\n", true),
            ("outer: { # open\n  key: old\n  } # close\n", true),
            ("&a [ # open\n  *a\n] # close\n", false),
        ] {
            let source = source.replace('\n', newline);
            let mut document = EditableDocument::parse(&source).unwrap();
            document
                .edit(|editor| {
                    let target = collection(editor, nested);
                    match editor.node(&target)?.kind() {
                        NodeKind::Sequence { .. } => editor.insert_item(&target, 1, "new"),
                        _ => editor.insert_entry(&target, 1, "added", "new"),
                    }
                })
                .unwrap();
            let committed = document.to_yaml();
            document
                .edit(|editor| {
                    let target = collection(editor, nested);
                    editor.set_collection_style(&target, CollectionStyle::Flow)
                })
                .unwrap();
            assert_eq!(document.source(), committed);
            diffs.push(edit_diff(&source, document.source()));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     [ # open
    -  old
    -] # close
    +  old, new] # close

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     { # open
    -  key: old
    -} # close
    +  key: old, added: new} # close

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     outer: [ # open
    -  old
    -  ] # close
    +  old, new] # close

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     outer: { # open
    -  key: old
    -  } # close
    +  key: old, added: new} # close

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     &a [ # open
    -  *a
    -] # close
    +  *a, new] # close

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     [ # open␍
    -  old␍
    -] # close␍
    +  old, new] # close␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     { # open␍
    -  key: old␍
    -} # close␍
    +  key: old, added: new} # close␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     outer: [ # open␍
    -  old␍
    -  ] # close␍
    +  old, new] # close␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     outer: { # open␍
    -  key: old␍
    -  } # close␍
    +  key: old, added: new} # close␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     &a [ # open␍
    -  *a␍
    -] # close␍
    +  *a, new] # close␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     [ # open␍-  old␍-] # close␍+  old, new] # close␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     { # open␍-  key: old␍-} # close␍+  key: old, added: new} # close␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     outer: [ # open␍-  old␍-  ] # close␍+  old, new] # close␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     outer: { # open␍-  key: old␍-  } # close␍+  key: old, added: new} # close␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     &a [ # open␍-  *a␍-] # close␍+  *a, new] # close␍
    ");
}

#[test]
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
                    let target = collection(editor, true);
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

#[test]
fn shrinking_collections_retain_their_comment_boundaries() {
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        for source in [
            "outer: [ # open\n  old\n  ] # close\n",
            "outer: { # open\n  key: old\n  } # close\n",
            "outer:\n# leading\n  {key: old}\n",
            "outer:\n# detached\n  key: old\n",
        ] {
            let source = source.replace('\n', newline);
            let mut document = EditableDocument::parse(&source).unwrap();
            document
                .edit(|editor| {
                    let target = collection(editor, true);
                    match editor.node(&target)?.kind() {
                        NodeKind::Sequence { .. } => editor.remove_item(&target, 0),
                        _ => editor.remove_entry(&target, 0),
                    }
                })
                .unwrap();
            diffs.push(edit_diff(&source, document.source()));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     outer: [ # open
    -  old
       ] # close

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     outer: { # open
    -  key: old
       } # close

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1 @@
    -outer:
    -# leading
    -  {key: old}
    +outer: {}

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     outer:
    -# detached
    -  key: old
    +  # detached
    +
    +  {}

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     outer: [ # open␍
    -  old␍
       ] # close␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     outer: { # open␍
    -  key: old␍
       } # close␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1 @@
    -outer:␍
    -# leading␍
    -  {key: old}␍
    +outer: {}␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     outer:␍
    -# detached␍
    -  key: old␍
    +  # detached␍
    +␍
    +  {}␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     outer: [ # open␍-  old␍   ] # close␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,2 @@
     outer: { # open␍-  key: old␍   } # close␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1 @@
    -outer:␍-# leading␍-  {key: old}␍+outer: {}␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
     outer:␍-# detached␍-  key: old␍+  # detached␍+␍+  {}␍
    ");
}

#[test]
fn rewrite_context_is_selected_after_all_mutations() {
    let source = "outer:   [old]\nuntouched:    keep\n";
    let mut document = EditableDocument::parse(source).unwrap();
    document
        .edit(|editor| {
            let target = collection(editor, true);
            // The transient block layout needs its parent. The final flow layout
            // fits in the original value region and keeps the surrounding source.
            editor.set_collection_style(&target, CollectionStyle::Block)?;
            editor.insert_item(&target, 1, "new")?;
            editor.set_collection_style(&target, CollectionStyle::Flow)
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff(source, document.source()), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
    -outer:   [old]
    +outer:   [old, new]
     untouched:    keep
    ");
}

#[test]
fn retained_opening_comments_merge_with_fragment_prefixes() {
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        let source = "[ # open\n  old\n] # close\n".replace('\n', newline);
        let mut document = EditableDocument::parse(&source).unwrap();
        document
            .edit(|editor| {
                editor.replace(&editor.root(), "# detached\n\n[new]\n")?;
                editor.replace_comment(&editor.root(), 1, " updated")
            })
            .unwrap();
        diffs.push(edit_diff(&source, document.source()));
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
    -[ # open
    -  old
    -] # close
    +# detached
    +
    +[ # updated
    +  new] # close

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
    -[ # open␍
    -  old␍
    -] # close␍
    +# detached␍
    +␍
    +[ # updated␍
    +  new] # close␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,4 @@
    -[ # open␍-  old␍-] # close␍+# detached␍+␍+[ # updated␍+  new] # close␍
    ");
}
