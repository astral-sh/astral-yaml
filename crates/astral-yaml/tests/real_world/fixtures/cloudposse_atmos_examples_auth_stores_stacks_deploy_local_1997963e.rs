#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("cloudposse-atmos-examples-auth-stores-stacks-deploy-local-1997963e"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "import" [plain @1:1..1:7]: sequence [block]
        - "catalog/emulator/aws" [plain @2:5..2:25]
      "vars" [plain @4:1..4:5]: mapping [block]
        "stage" [plain @5:3..5:8]: "local" [plain @5:10..5:15]
    "#);
}
