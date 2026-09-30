#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-docs-snippets-docs-snippets-guides-components-post-processing-components-my-assets-defs-3-0152228d"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @1:1..1:5]: "dagster.DefsFolderComponent" [plain @1:7..1:34]
      "post_processing" [plain @3:1..3:16]: mapping [block]
        "assets" [plain @4:3..4:9]: sequence [block]
          - mapping [block]
            "attributes" [plain @5:7..5:17]: mapping [block]
              "kinds" [plain @6:9..6:14]: sequence [block]
                - "some_kind" [double-quoted @7:11..7:22]
          - mapping [block]
            "target" [plain @8:7..8:13]: "foo" [plain @8:15..8:18]
            "attributes" [plain @9:7..9:17]: mapping [block]
              "group_name" [plain @10:9..10:19]: "foo_group" [double-quoted @10:21..10:32]
          - mapping [block]
            "target" [plain @11:7..11:13]: "bar" [plain @11:15..11:18]
            "attributes" [plain @12:7..12:17]: mapping [block]
              "group_name" [plain @13:9..13:19]: "bar_group" [double-quoted @13:21..13:32]
    "#);
}
