#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("cloudposse-atmos-examples-demo-env-stacks-deploy-demo-18f743cb"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "vars" [plain @1:1..1:5]: mapping [block]
        "stage" [plain @2:3..2:8]: "demo" [plain @2:10..2:14]
      "components" [plain @4:1..4:11]: mapping [block]
        "terraform" [plain @5:3..5:12]: mapping [block]
          "github-repo" [plain @6:5..6:16]: mapping [block]
            "vars" [plain @7:7..7:11]: mapping [block]
              "repository" [plain @8:9..8:19]: "cloudposse/atmos" [plain @8:21..8:37]
    "#);
}
