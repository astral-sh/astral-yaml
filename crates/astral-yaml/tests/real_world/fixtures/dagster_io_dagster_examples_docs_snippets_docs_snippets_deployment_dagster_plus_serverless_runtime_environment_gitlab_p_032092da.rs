#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-docs-snippets-docs-snippets-deployment-dagster-plus-serverless-runtime-environment-gitlab-p-032092da"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "variables" [plain @1:1..1:10]: mapping [block]
        "DISABLE_FAST_DEPLOYS" [plain @2:3..2:23]: "" [plain @2:24..2:24]
        "DAGSTER_CLOUD_URL" [plain @3:3..3:20]: "$DAGSTER_CLOUD_URL" [plain @3:22..3:40]
        "DAGSTER_CLOUD_API_TOKEN" [plain @4:3..4:26]: "$DAGSTER_CLOUD_API_TOKEN" [plain @4:28..4:52]
        "PYTHON_VERSION" [plain @6:3..6:17]: "3.11" [single-quoted @6:19..6:25]
    "#);
}
