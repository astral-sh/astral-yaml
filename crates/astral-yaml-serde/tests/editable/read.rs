use astral_yaml_serde::edit::CommentPlacement;
use astral_yaml_serde::{Commented, Editable, Spanned, Tagged, from_str};
use serde_json::Value;

#[test]
fn updating_an_alias_reads_its_pending_target_and_replaces_only_the_alias() {
    // Reading an alias resolves to the anchored node.
    // https://yaml.org/spec/1.2.2/#71-alias-nodes
    let source = "retries: &default 3 # attempts\ncopy: *default\nother: 'keep'\n";
    let mut document = Editable::<Value>::parse(source).unwrap();
    document
        .edit(|mut root| {
            root.reborrow().field("retries")?.update(|n: u8| n + 1)?;
            root.field("copy")?.update(|n: u8| n * 2)
        })
        .unwrap();
    insta::assert_snapshot!(document.source(), @"
    retries: &default 4 # attempts
    copy: 8
    other: 'keep'
    ");
}

#[test]
fn a_failed_read_identifies_an_inserted_value_without_preventing_its_repair() {
    let mut document = Editable::<Value>::parse("settings: {}\n").unwrap();
    let error = document
        .edit_at(["settings"], |settings| {
            let mut fields = settings.into_mapping()?;
            let mut limits = fields.set("limits", ["invalid"])?.into_sequence()?;
            let error = limits.read::<Vec<u8>>().unwrap_err();
            limits.item(0)?.set(5)?;
            Ok(error)
        })
        .unwrap();
    insta::assert_snapshot!(error, @r#"$["settings"]["limits"][0]: expected an integer"#);
    assert_eq!(document.value()["settings"]["limits"][0], 5);
}

#[test]
fn a_failed_update_prevents_commit_and_does_not_run_the_transform() {
    let source = "3 # keep\n";
    let mut document = Editable::<u8>::parse(source).unwrap();
    let error = document
        .edit(|mut value| {
            let mut calls = 0;
            let mut increment = |n: u8| {
                calls += 1;
                n + 1
            };
            value.update(&mut increment)?;
            value.set("invalid")?;
            let error = value.update(&mut increment).unwrap_err();
            assert_eq!(value.update(&mut increment).unwrap_err(), error);
            assert_eq!(calls, 1);
            Ok(())
        })
        .unwrap_err();
    insta::assert_snapshot!(astral_yaml_dev::diagnostic_snapshot(source, error), @"
      |
    1 | 3 # keep
      | ^ $: expected an integer
    ");
    assert_eq!(document.source(), source);
}

#[test]
fn reads_borrow_decoded_text_and_only_offer_spans_for_existing_syntax() {
    let source = "labels: ['old'] # names\n";
    let mut document = Editable::<Value>::parse(source).unwrap();
    document
        .edit(|mut root| {
            let expected: Commented<'_, Value> = from_str(source)?;
            assert_eq!(root.read::<Commented<'_, Value>>()?, expected);

            let mut fields = root.reborrow().into_mapping()?;
            let mut added = fields.set("added", "first\nsecond")?;
            assert_eq!(added.read::<&str>()?, "first\nsecond");
            let metadata: Commented<'_, &str> = added.read()?;
            assert_eq!(metadata.comments, []);

            let error = added.read::<Spanned<&str>>().unwrap_err();
            insta::assert_snapshot!(error, @r#"$["added"]: new node has no source span until the edit commits"#);
            added.append_comment(" new label", CommentPlacement::Inline)?;
            let error = added.read::<Commented<'_, &str>>().unwrap_err();
            insta::assert_snapshot!(error, @r#"$["added"]: new comment has no source span until the edit commits"#);
            Ok(())
        })
        .unwrap();

    document
        .edit_at(["added"], |added| {
            let metadata: Commented<'_, Spanned<&str>> = added.read()?;
            assert_eq!(metadata.comments[0].comment.text, " new label");
            assert_eq!(metadata.value.value, "first\nsecond");
            Ok(())
        })
        .unwrap();
}

#[test]
fn reads_report_a_dangling_alias_without_preventing_repair() {
    // An alias needs a preceding anchor in the same document.
    // https://yaml.org/spec/1.2.2/#71-alias-nodes
    let source = "default: &value {nested: old}\ncopy: *value\n";
    let mut document = Editable::<Value>::parse(source).unwrap();
    let error = document
        .edit(|root| {
            let mut fields = root.into_mapping()?;
            // The target has this key, but navigation stops at the alias.
            assert!(fields.field("copy")?.field("nested").is_err());
            fields.remove("default")?;
            let mut copy = fields.field("copy")?;
            let error = copy.read::<&str>().unwrap_err();
            copy.set("repaired")?;
            Ok(error)
        })
        .unwrap();
    insta::assert_snapshot!(astral_yaml_dev::diagnostic_snapshot(source, error), @r#"
      |
    1 | / default: &value {nested: old}
    2 | | copy: *value
      | |____________^ $["copy"]: node is no longer in the document
    "#);
}

#[test]
fn reads_apply_tag_resolution() {
    // The explicit local tag identifies the value for the application.
    // https://yaml.org/spec/1.2.2/#691-node-tags
    let mut document = Editable::<Value>::parse("null").unwrap();
    document
        .edit(|mut root| {
            root.set(Tagged {
                tag: "!label".to_owned(),
                value: 9,
            })?;
            let tagged: Tagged<u8> = root.read()?;
            assert_eq!(tagged.tag, "!label");
            assert_eq!(tagged.value, 9);
            root.set(())
        })
        .unwrap();
}

#[test]
fn reads_bound_alias_expansion_in_an_existing_document() {
    // Load the cyclic representation graph before expanding it into a Rust value.
    // https://yaml.org/spec/1.2.2/#321-representation-graph
    struct Graph;
    impl<'de> serde::Deserialize<'de> for Graph {
        fn deserialize<D: serde::Deserializer<'de>>(_: D) -> Result<Self, D::Error> {
            Ok(Self)
        }
    }

    let mut document = Editable::<Graph>::parse("&cycle [*cycle]").unwrap();
    document
        .edit(|mut root| {
            let error = root.read::<Value>().unwrap_err();
            assert_eq!(error.message, "deserialization resource limit exceeded");
            root.set(())
        })
        .unwrap();
}
