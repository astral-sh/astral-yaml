//! Flow collection suffix comments belong before their closing delimiter.

use astral_yaml::edit::EditableDocument;
use astral_yaml_dev::edit_diff;

#[test]
#[allow(clippy::too_many_lines, reason = "inline edit snapshots")]
fn normalization_keeps_flow_suffixes_inside_their_owner() {
    let mut diffs = Vec::new();
    for newline in ["\n", "\r\n", "\r"] {
        for source in [
            "outer: {key: old\n  # trailing\n  }\n",
            "outer: [old\n\n  # detached\n  ]\n",
            "[one, {key: old\n  # trailing\n\n  # detached\n  }]\n",
            "outer: [ # opening\n  old\n  # trailing\n  ] # closing\n",
            "outer: {\n  # detached\n\n  }\n",
        ] {
            let source = source.replace('\n', newline);
            let document = EditableDocument::parse(&source).unwrap();
            let output = document.normalized_yaml().unwrap();
            assert_eq!(
                EditableDocument::parse(&output)
                    .unwrap()
                    .normalized_yaml()
                    .unwrap(),
                output
            );
            diffs.push(edit_diff(&source, &output));
        }
    }
    insta::assert_snapshot!(diffs.join("\n"), @"

    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
     outer: [old
    -
    +  ⏎
       # detached
       ]

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     [one, {key: old
    -  # trailing
    -
    -  # detached
    -  }]
    +    # trailing
    +    ⏎
    +    # detached
    +    }]


    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
     outer: {
    +  ⏎
       # detached
    -
       }


    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
     outer: [old␍
    -␍
    +  ␍
       # detached␍
       ]␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     [one, {key: old␍
    -  # trailing␍
    -␍
    -  # detached␍
    -  }]␍
    +    # trailing␍
    +    ␍
    +    # detached␍
    +    }]␍


    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
     outer: {␍
    +  ␍
       # detached␍
    -␍
       }␍


    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
     outer: [old␍-␍+  ␍   # detached␍   ]␍

    --- before.yaml
    +++ after.yaml
    @@ -1,5 +1,5 @@
     [one, {key: old␍-  # trailing␍-␍-  # detached␍-  }]␍+    # trailing␍+    ␍+    # detached␍+    }]␍


    --- before.yaml
    +++ after.yaml
    @@ -1,4 +1,4 @@
     outer: {␍+  ␍   # detached␍-␍   }␍
    ");
}

#[test]
fn flow_comment_layouts_normalize_idempotently() {
    // Delimiters enclose comments independently of block indentation:
    // https://yaml.org/spec/1.2.2/#74-flow-collection-styles
    for newline in ["\n", "\r\n", "\r"] {
        for (open, body, close) in [
            ("[", "one", "]"),
            ("{", "key: value", "}"),
            ("[", "", "]"),
            ("{", "", "}"),
        ] {
            for prefix in [
                "",
                " # open\n  ",
                "\n  # leading\n  ",
                "\n\n  # detached\n\n  ",
            ] {
                for suffix in [
                    "",
                    "\n  # trailing\n  ",
                    "\n\n  # detached\n  ",
                    "\n  # trailing\n\n  # detached\n  ",
                ] {
                    for end in ["", " # close"] {
                        let inner = format!("{open}{prefix}{body}{suffix}{close}{end}");
                        for source in [
                            format!("{inner}\n"),
                            format!("outer: {inner}\n"),
                            format!("- {inner}\n"),
                            format!("[{inner}\n]\n"),
                            format!("{{outer: {inner}\n}}\n"),
                        ] {
                            let source = source.replace('\n', newline);
                            let document = EditableDocument::parse(&source).unwrap();
                            let output = document
                                .normalized_yaml()
                                .unwrap_or_else(|error| panic!("{source:?}: {error}"));
                            assert_eq!(
                                EditableDocument::parse(&output)
                                    .unwrap()
                                    .normalized_yaml()
                                    .unwrap(),
                                output
                            );
                        }
                    }
                }
            }
        }
    }
}
