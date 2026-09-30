#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-docs-snippets-docs-snippets-deployment-dagster-plus-deploying-code-branch-deployments-helm-141e9570"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "dagsterCloud" [plain @2:1..2:13]: mapping [block]
        "branchDeployments" [plain @3:3..3:20]: "true" [plain @3:22..3:26]
        "workspace" [plain @4:3..4:12]: mapping [block]
          "serverTTL" [plain @5:5..5:14]: mapping [block]
            "enabled" [plain @6:7..6:14]: "true" [plain @6:16..6:20]
            "ttlSeconds" [plain @7:7..7:17]: "7200" [plain @7:19..7:23]
    "#);
}
