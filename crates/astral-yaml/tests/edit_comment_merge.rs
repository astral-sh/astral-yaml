//! Comment placement across fragment replacement and presentation changes.

use astral_yaml::{CollectionStyle, edit::EditableDocument};
use astral_yaml_dev::edit_diff;

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn replacement_merges_comments_at_their_structural_positions() {
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        for (source, replacement) in [
            ("old # tail\n", "# leading\nnew\n"),
            ("a: old\n\n# tail\n", "a: one\n\n# middle\n\nb: two\n"),
            (
                "a: old\nb: old\n\n# retained\n\nc: old\n",
                "a: one\n\n# imported\n\nb: two\nc: three\n",
            ),
            (
                "- old\n\n# retained\n\n- old\n",
                "# leading\n- one\n\n# imported\n\n- two\n",
            ),
            ("old\n# trailing\n", "new # inline\n"),
        ] {
            let source = source.replace('\n', newline);
            let replacement = replacement.replace('\n', newline);
            let mut document = EditableDocument::parse(&source).unwrap();
            document
                .edit(|editor| editor.replace(&editor.root(), &replacement))
                .unwrap();
            diffs.push(edit_diff(&source, document.source()));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -old # tail
    +# leading
    +new # tail

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,7 @@
    -a: old
    +a: one
    +
    +# middle
    +
    +b: two
     ⏎
     # tail

    --- before.yaml
    +++ after.yaml
    @@ -1,6 +1,9 @@
    -a: old
    -b: old
    +a: one
    +
    +# imported
    +
    +b: two
     ⏎
     # retained
     ⏎
    -c: old
    +c: three

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,9 @@
    -- old
    +# leading
    +- one
     ⏎
     # retained
     ⏎
    -- old
    +
    +# imported
    +
    +- two

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
    -old
    +new # inline
     # trailing

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -old # tail␍
    +# leading␍
    +new # tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,7 @@
    -a: old␍
    +a: one␍
    +␍
    +# middle␍
    +␍
    +b: two␍
     ␍
     # tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,6 +1,9 @@
    -a: old␍
    -b: old␍
    +a: one␍
    +␍
    +# imported␍
    +␍
    +b: two␍
     ␍
     # retained␍
     ␍
    -c: old␍
    +c: three␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,9 @@
    -- old␍
    +# leading␍
    +- one␍
     ␍
     # retained␍
     ␍
    -- old␍
    +␍
    +# imported␍
    +␍
    +- two␍

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
    -old␍
    +new # inline␍
     # trailing␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -old # tail␍+# leading␍+new # tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,3 +1,7 @@
    -a: old␍+a: one␍+␍+# middle␍+␍+b: two␍ ␍ # tail␍

    --- before.yaml
    +++ after.yaml
    @@ -1,6 +1,9 @@
    -a: old␍-b: old␍+a: one␍+␍+# imported␍+␍+b: two␍ ␍ # retained␍ ␍-c: old␍+c: three␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,9 @@
    -- old␍+# leading␍+- one␍ ␍ # retained␍ ␍-- old␍+␍+# imported␍+␍+- two␍

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,2 @@
    -old␍+new # inline␍ # trailing␍
    ");
}

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn root_collection_headers_retain_inline_comments() {
    // Node properties: https://yaml.org/spec/1.2.2/#69-node-properties
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        for source in [
            "&root [old] # inline\n",
            "&root {a: old} # inline\n",
            "!!seq [old] # inline\n",
            "!!seq # tag\n&root [old] # inline\n",
            "# leading\n%YAML 1.2 # directive\n--- &root [old] # inline\n... # end\n# trailing\n",
        ] {
            let source = source.replace('\n', newline);
            let mut document = EditableDocument::parse(&source).unwrap();
            document
                .edit(|editor| editor.set_collection_style(&editor.root(), CollectionStyle::Block))
                .unwrap();
            let once = document.source().to_owned();
            document
                .edit(|editor| editor.set_collection_style(&editor.root(), CollectionStyle::Block))
                .unwrap();
            assert_eq!(document.source(), once);
            diffs.push(edit_diff(&source, document.source()));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @"
    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -&root [old] # inline
    +&root # inline
    +- old

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -&root {a: old} # inline
    +&root # inline
    +a: old

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -!!seq [old] # inline
    +!<tag:yaml.org,2002:seq> # inline
    +- old

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
    -!!seq # tag
    -&root [old] # inline
    +!<tag:yaml.org,2002:seq> # tag
    +&root # inline
    +- old

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,7 @@
     # leading
     %YAML 1.2 # directive
    ---- &root [old] # inline
    +--- ⏎
    +&root # inline
    +- old
     ... # end
     # trailing

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -&root [old] # inline␍
    +&root # inline␍
    +- old␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -&root {a: old} # inline␍
    +&root # inline␍
    +a: old␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -!!seq [old] # inline␍
    +!<tag:yaml.org,2002:seq> # inline␍
    +- old␍

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
    -!!seq # tag␍
    -&root [old] # inline␍
    +!<tag:yaml.org,2002:seq> # tag␍
    +&root # inline␍
    +- old␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,7 @@
     # leading␍
     %YAML 1.2 # directive␍
    ---- &root [old] # inline␍
    +--- ␍
    +&root # inline␍
    +- old␍
     ... # end␍
     # trailing␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -&root [old] # inline␍+&root # inline␍+- old␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -&root {a: old} # inline␍+&root # inline␍+a: old␍

    --- before.yaml
    +++ after.yaml
    @@ -1 +1,2 @@
    -!!seq [old] # inline␍+!<tag:yaml.org,2002:seq> # inline␍+- old␍

    --- before.yaml
    +++ after.yaml
    @@ -1,2 +1,3 @@
    -!!seq # tag␍-&root [old] # inline␍+!<tag:yaml.org,2002:seq> # tag␍+&root # inline␍+- old␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,7 @@
     # leading␍ %YAML 1.2 # directive␍---- &root [old] # inline␍+--- ␍+&root # inline␍+- old␍ ... # end␍ # trailing␍
    ");
}
