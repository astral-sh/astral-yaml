#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-docs-projects-project-dbt-src-project-dbt-analytics-packages-080afe4d"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "packages" [plain @1:1..1:9]: sequence [block]
        - mapping [block]
          "package" [plain @2:5..2:12]: "dbt-labs/dbt_utils" [plain @2:14..2:32]
          "version" [plain @3:5..3:12]: "1.1.1" [plain @3:14..3:19]
    "#);
}
