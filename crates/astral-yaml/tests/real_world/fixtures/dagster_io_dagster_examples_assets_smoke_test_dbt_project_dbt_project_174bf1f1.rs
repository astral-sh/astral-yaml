#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-assets-smoke-test-dbt-project-dbt-project-174bf1f1"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "name" [plain @1:1..1:5]: "dbt_project" [double-quoted @1:7..1:20]
      "version" [plain @2:1..2:8]: "1.0.0" [double-quoted @2:10..2:17]
      "config-version" [plain @3:1..3:15]: "2" [plain @3:17..3:18]
      "profile" [plain @5:1..5:8]: "dbt_project" [double-quoted @5:10..5:23]
      "analysis-paths" [plain @7:1..7:15]: sequence [flow]
        - "analyses" [double-quoted @7:18..7:28]
      "test-paths" [plain @8:1..8:11]: sequence [flow]
        - "tests" [double-quoted @8:14..8:21]
      "macro-paths" [plain @9:1..9:12]: sequence [flow]
        - "macros" [double-quoted @9:15..9:23]
      "snapshot-paths" [plain @10:1..10:15]: sequence [flow]
        - "snapshots" [double-quoted @10:18..10:29]
      "target-path" [plain @12:1..12:12]: "target" [double-quoted @12:14..12:22]
      "clean-targets" [plain @13:1..13:14]: sequence [block]
        - "target" [double-quoted @14:5..14:13]
        - "dbt_packages" [double-quoted @15:5..15:19]
      "models" [plain @17:1..17:7]: mapping [block]
        "+materialized" [plain @18:3..18:16]: "table" [plain @18:18..18:23]
    "#);
}
