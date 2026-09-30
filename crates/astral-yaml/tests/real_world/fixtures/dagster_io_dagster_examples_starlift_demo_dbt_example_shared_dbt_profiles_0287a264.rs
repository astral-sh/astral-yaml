#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-starlift-demo-dbt-example-shared-dbt-profiles-0287a264"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "jaffle_shop" [plain @1:1..1:12]: mapping [block]
        "target" [plain @3:3..3:9]: "dev" [plain @3:11..3:14]
        "outputs" [plain @4:3..4:10]: mapping [block]
          "dev" [plain @5:5..5:8]: mapping [block]
            "type" [plain @6:7..6:11]: "duckdb" [plain @6:13..6:19]
            "path" [plain @7:7..7:11]: "{{ env_var('DBT_PROJECT_DIR') }}/../jaffle_shop.duckdb" [double-quoted @7:13..7:69]
            "threads" [plain @8:7..8:14]: "24" [plain @8:16..8:18]
    "#);
}
