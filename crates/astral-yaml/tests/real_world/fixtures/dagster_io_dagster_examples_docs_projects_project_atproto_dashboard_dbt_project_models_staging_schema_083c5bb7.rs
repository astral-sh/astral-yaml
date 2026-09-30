#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-docs-projects-project-atproto-dashboard-dbt-project-models-staging-schema-083c5bb7"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "version" [plain @1:1..1:8]: "2" [plain @1:10..1:11]
      "models" [plain @3:1..3:7]: sequence [block]
        - mapping [block]
          "name" [plain @4:5..4:9]: "stg_profiles" [plain @4:11..4:23]
          "description" [plain @5:5..5:16]: "raw data from r2 bucket" [double-quoted @5:18..5:43]
        - mapping [block]
          "name" [plain @6:5..6:9]: "stg_feed_snapshots" [plain @6:11..6:29]
          "description" [plain @7:5..7:16]: "raw posts data from r2 bucket" [double-quoted @7:18..7:49]
    "#);
}
