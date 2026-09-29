#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("cloudposse-atmos-examples-custom-components-stacks-deploy-dev-1aed21d0"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "import" [plain @1:1..1:7]: sequence [block]
        - "catalog/script/deploy-app" [plain @2:5..2:30]
      "vars" [plain @4:1..4:5]: mapping [block]
        "stage" [plain @5:3..5:8]: "dev" [plain @5:10..5:13]
      "components" [plain @7:1..7:11]: mapping [block]
        "script" [plain @8:3..8:9]: mapping [block]
          "deploy-app" [plain @9:5..9:15]: mapping [block]
            "vars" [plain @10:7..10:11]: mapping [block]
              "replicas" [plain @11:9..11:17]: "1" [plain @11:19..11:20]
    "#);
}
