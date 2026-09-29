#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("ansible-awx-tools-grafana-datasources-prometheus-source-028973be"), @r#"
    document 1 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @2:1..2:11]: "1" [plain @2:13..2:14]
      "datasources" [plain @4:1..4:12]: sequence [block]
        - mapping [block]
          "name" [plain @5:5..5:9]: "Prometheus" [plain @5:11..5:21]
          "type" [plain @6:5..6:9]: "prometheus" [plain @6:11..6:21]
          "isDefault" [plain @7:5..7:14]: "true" [plain @7:16..7:20]
          "access" [plain @8:5..8:11]: "proxy" [plain @8:13..8:18]
          "url" [plain @9:5..9:8]: "http://prometheus:9090" [plain @9:10..9:32]
          "editable" [plain @10:5..10:13]: "true" [plain @10:15..10:19]
          "jsonData" [plain @11:5..11:13]: mapping [block]
            "timeInterval" [plain @12:7..12:19]: "5s" [plain @12:21..12:23]
          "uid" [plain @13:5..13:8]: "awx_prometheus" [plain @13:10..13:24]
    "#);
}
