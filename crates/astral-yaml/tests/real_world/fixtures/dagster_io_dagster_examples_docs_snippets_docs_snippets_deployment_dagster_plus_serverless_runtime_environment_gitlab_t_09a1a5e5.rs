#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-docs-snippets-docs-snippets-deployment-dagster-plus-serverless-runtime-environment-gitlab-t-09a1a5e5"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "include" [plain @1:1..1:8]: mapping [block]
        "remote" [plain @2:3..2:9]: "https://raw.githubusercontent.com/dagster-io/dagster-cloud-action/v0.1.29/gitlab/dbt/serverless-ci-dbt.yml" [plain @2:11..2:117]
    "#);
}
