#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("elastic-ansible-elasticsearch-handlers-main-67da3219"), @r#"
    document 1 (start: implicit, end: implicit)
    sequence [block]
      - mapping [block]
        "name" [plain @2:3..2:7]: "reload systemd configuration" [plain @2:9..2:37]
        "become" [plain @3:3..3:9]: "yes" [plain @3:11..3:14]
        "systemd" [plain @4:3..4:10]: mapping [block]
          "daemon_reload" [plain @5:5..5:18]: "true" [plain @5:20..5:24]
      - mapping [block]
        "name" [plain @9:3..9:7]: "restart elasticsearch" [plain @9:9..9:30]
        "become" [plain @10:3..10:9]: "yes" [plain @10:11..10:14]
        "service" [plain @11:3..11:10]: "name=elasticsearch state=restarted enabled=yes" [plain @11:12..11:58]
        "when" [plain @12:3..12:7]: sequence [block]
          - "es_restart_on_change" [plain @13:7..13:27]
          - "es_start_service" [plain @14:7..14:23]
        "register" [plain @15:3..15:11]: "es_restarted" [plain @15:13..15:25]
    "#);
}
