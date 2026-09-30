#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("oai-openapi-specification-proposals-overlays-mergepatch-aac000a6"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "overlay" [plain @1:1..1:8]: "1.0.0" [plain @1:10..1:15]
      "info" [plain @2:1..2:5]: mapping [block]
        "title" [plain @3:3..3:8]: "An example of an overlay that performs a Merge Patch" [plain @3:10..3:62]
        "version" [plain @4:3..4:10]: "1.0.0" [plain @4:12..4:17]
      "updates" [plain @5:1..5:8]: sequence [block]
        - mapping [block]
          "target" [plain @6:3..6:9]: "@" [double-quoted @6:11..6:14]
          "merge" [plain @7:3..7:8]: mapping [block]
            "openapi" [plain @8:5..8:12]: "3.0.3" [plain @8:14..8:19]
            "paths" [plain @9:5..9:10]: mapping [flow]
    "#);
}
