#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("ansible-awx-tools-grafana-dashboards-awx-dashboard-46e80af1"), @r#"
    document 1 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @2:1..2:11]: "1" [plain @2:13..2:14]
      "providers" [plain @4:1..4:10]: sequence [block]
        - mapping [block]
          "name" [plain @5:5..5:9]: "awx-dashboards" [plain @5:11..5:25]
          "type" [plain @6:5..6:9]: "file" [plain @6:11..6:15]
          "allowUiUpdates" [plain @7:5..7:19]: "true" [plain @7:21..7:25]
          "options" [plain @8:5..8:12]: mapping [block]
            "foldersFromFilesStructure" [plain @9:7..9:32]: "true" [plain @9:34..9:38]
            "path" [plain @10:7..10:11]: "/etc/grafana/provisioning/dashboards/" [plain @10:13..10:50]
    "#);
}
