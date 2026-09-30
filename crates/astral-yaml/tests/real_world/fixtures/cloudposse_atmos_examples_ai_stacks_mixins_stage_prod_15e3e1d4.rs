#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("cloudposse-atmos-examples-ai-stacks-mixins-stage-prod-15e3e1d4"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "vars" [plain @2:1..2:5]: mapping [block]
        "stage" [plain @3:3..3:8]: "prod" [plain @3:10..3:14]
    "#);
}
