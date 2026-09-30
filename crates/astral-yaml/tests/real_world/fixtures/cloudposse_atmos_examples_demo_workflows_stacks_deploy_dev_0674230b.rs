#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("cloudposse-atmos-examples-demo-workflows-stacks-deploy-dev-0674230b"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "import" [plain @3:1..3:7]: sequence [block]
        - "catalog/myapp" [plain @4:5..4:18]
      "vars" [plain @6:1..6:5]: mapping [block]
        "stage" [plain @7:3..7:8]: "dev" [plain @7:10..7:13]
    "#);
}
