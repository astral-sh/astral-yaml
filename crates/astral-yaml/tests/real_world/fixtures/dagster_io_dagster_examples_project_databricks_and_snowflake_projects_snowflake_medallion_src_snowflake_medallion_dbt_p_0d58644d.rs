#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-project-databricks-and-snowflake-projects-snowflake-medallion-src-snowflake-medallion-dbt-p-0d58644d"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "snowflake_medallion" [plain @1:1..1:20]: mapping [block]
        "target" [plain @2:3..2:9]: "duckdb" [plain @2:11..2:17]
        "outputs" [plain @3:3..3:10]: mapping [block]
          "duckdb" [plain @4:5..4:11]: mapping [block]
            "type" [plain @5:7..5:11]: "duckdb" [plain @5:13..5:19]
            "path" [plain @6:7..6:11]: ":memory:" [double-quoted @6:13..6:23]
          "snowflake" [plain @7:5..7:14]: mapping [block]
            "type" [plain @8:7..8:11]: "snowflake" [plain @8:13..8:22]
            "account" [plain @9:7..9:14]: "{{ env_var('SNOWFLAKE_ACCOUNT') }}" [double-quoted @9:16..9:52]
            "user" [plain @10:7..10:11]: "{{ env_var('SNOWFLAKE_USER') }}" [double-quoted @10:13..10:46]
            "password" [plain @11:7..11:15]: "{{ env_var('SNOWFLAKE_PASSWORD') }}" [double-quoted @11:17..11:54]
            "warehouse" [plain @12:7..12:16]: "{{ env_var('SNOWFLAKE_WAREHOUSE') }}" [double-quoted @12:18..12:56]
            "database" [plain @13:7..13:15]: "{{ env_var('SNOWFLAKE_DATABASE') }}" [double-quoted @13:17..13:54]
            "schema" [plain @14:7..14:13]: "{{ env_var('SNOWFLAKE_SCHEMA', 'PUBLIC') }}" [double-quoted @14:15..14:60]
    "#);
}
