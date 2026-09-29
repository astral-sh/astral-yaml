#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("apache-airflow-scripts-ci-docker-compose-grafana-volume-provisioning-dashboards-default-74758e08"), @r#"
    document 1 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @18:1..18:11]: "1" [plain @18:13..18:14]
      "providers" [plain @19:1..19:10]: sequence [block]
        - mapping [block]
          "name" [plain @20:5..20:9]: "dashboards" [plain @20:11..20:21]
          "folder" [plain @21:5..21:11]: "General" [plain @21:13..21:20]
          "type" [plain @22:5..22:9]: "file" [plain @22:11..22:15]
          "editable" [plain @23:5..23:13]: "true" [plain @23:15..23:19]
          "updateIntervalSeconds" [plain @24:5..24:26]: "10" [plain @24:28..24:30]
          "allowUiUpdates" [plain @25:5..25:19]: "true" [plain @25:21..25:25]
          "options" [plain @26:5..26:12]: mapping [block]
            "path" [plain @27:7..27:11]: "/grafana/dashboards" [plain @27:13..27:32]
            "foldersFromFilesStructure" [plain @28:7..28:32]: "true" [plain @28:34..28:38]
    "#);
}
