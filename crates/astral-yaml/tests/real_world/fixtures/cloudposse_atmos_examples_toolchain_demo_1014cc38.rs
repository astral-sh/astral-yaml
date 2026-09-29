#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("cloudposse-atmos-examples-toolchain-demo-1014cc38"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "name" [plain @1:1..1:5]: "atmos" [plain @1:7..1:12]
      "kind" [plain @2:1..2:5]: "toolchain-demo" [plain @2:7..2:21]
      "features" [plain @3:1..3:9]: sequence [block]
        - "registries" [plain @4:5..4:15]
        - "aliases" [plain @5:5..5:12]
        - "pipelines" [plain @6:5..6:14]
    "#);
}
