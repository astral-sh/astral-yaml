#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("ansible-awx-tools-docker-compose-logstash-override-af31f6e1"), @r#"
    document 1 (start: ---, end: implicit)
    mapping [block]
      "services" [plain @2:1..2:9]: mapping [block]
        "logstash" [plain @5:3..5:11]: mapping [block]
          "build" [plain @6:5..6:10]: mapping [block]
            "context" [plain @7:7..7:14]: "../" [plain @7:16..7:19]
            "dockerfile" [plain @8:7..8:17]: "Dockerfile-logstash" [plain @8:19..8:38]
          "container_name" [plain @9:5..9:19]: "tools_logstash_1" [plain @9:21..9:37]
          "hostname" [plain @10:5..10:13]: "logstash" [plain @10:15..10:23]
          "networks" [plain @11:5..11:13]: sequence [block]
            - "awx" [plain @12:9..12:12]
    "#);
}
