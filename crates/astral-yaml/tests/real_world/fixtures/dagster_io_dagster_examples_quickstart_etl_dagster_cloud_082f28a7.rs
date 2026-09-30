#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-quickstart-etl-dagster-cloud-082f28a7"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "locations" [plain @1:1..1:10]: sequence [block]
        - mapping [block]
          "location_name" [plain @2:5..2:18]: "quickstart_etl" [plain @2:20..2:34]
          "code_source" [plain @3:5..3:16]: mapping [block]
            "module_name" [plain @4:7..4:18]: "quickstart_etl.definitions" [plain @4:20..4:46]
          "working_directory" [plain @5:5..5:22]: "./src/" [plain @5:24..5:30]
    "#);
}
