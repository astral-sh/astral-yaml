#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-docs-snippets-docs-snippets-guides-components-integrations-dlt-component-6-defs-01c86cc9"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @1:1..1:5]: "dagster_dlt.DltLoadCollectionComponent" [plain @1:7..1:45]
      "attributes" [plain @3:1..3:11]: mapping [block]
        "loads" [plain @4:3..4:8]: sequence [block]
          - mapping [block]
            "source" [plain @5:7..5:13]: ".loads.my_load_source" [plain @5:15..5:36]
            "pipeline" [plain @6:7..6:15]: ".loads.my_load_pipeline" [plain @6:17..6:40]
    "#);
}
