#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-tutorial-notebook-assets-workspace-1de22c97"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "load_from" [plain @1:1..1:10]: sequence [block]
        - mapping [block]
          "python_package" [plain @2:5..2:19]: mapping [block]
            "package_name" [plain @3:7..3:19]: "tutorial_template.definitions" [plain @3:21..3:50]
            "location_name" [plain @4:7..4:20]: "tutorial_template" [plain @4:22..4:39]
        - mapping [block]
          "python_package" [plain @5:5..5:19]: mapping [block]
            "package_name" [plain @6:7..6:19]: "tutorial_finished.definitions" [plain @6:21..6:50]
            "location_name" [plain @7:7..7:20]: "tutorial_finished" [plain @7:22..7:39]
    "#);
}
