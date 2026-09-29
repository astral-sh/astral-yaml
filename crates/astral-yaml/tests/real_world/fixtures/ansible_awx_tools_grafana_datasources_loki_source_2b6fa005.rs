#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("ansible-awx-tools-grafana-datasources-loki-source-2b6fa005"), @r#"
    document 1 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @2:1..2:11]: "1" [plain @2:13..2:14]
      "datasources" [plain @4:1..4:12]: sequence [block]
        - mapping [block]
          "name" [plain @5:5..5:9]: "Loki" [plain @5:11..5:15]
          "type" [plain @6:5..6:9]: "loki" [plain @6:11..6:15]
          "access" [plain @7:5..7:11]: "proxy" [plain @7:13..7:18]
          "url" [plain @8:5..8:8]: "http://loki:3100" [plain @8:10..8:26]
          "jsonData" [plain @9:5..9:13]: mapping [block]
            "timeout" [plain @10:7..10:14]: "60" [plain @10:16..10:18]
            "maxLines" [plain @11:7..11:15]: "100000" [plain @11:17..11:23]
    "#);
}
