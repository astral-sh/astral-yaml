#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-experimental-external-assets-dagster-cloud-171d73af"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "locations" [plain @1:1..1:10]: sequence [block]
        - mapping [block]
          "location_name" [plain @2:5..2:18]: "pipes_external_airflow_demo" [plain @2:20..2:47]
          "code_source" [plain @3:5..3:16]: mapping [block]
            "module_name" [plain @4:7..4:18]: "pipes_external_airflow_demo.definitions" [plain @4:20..4:59]
    "#);
}
