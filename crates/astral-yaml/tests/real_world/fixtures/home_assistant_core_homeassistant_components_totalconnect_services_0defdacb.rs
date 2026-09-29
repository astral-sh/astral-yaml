#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("home-assistant-core-homeassistant-components-totalconnect-services-0defdacb"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "arm_away_instant" [plain @1:1..1:17]: mapping [block]
        "target" [plain @2:3..2:9]: mapping [block]
          "entity" [plain @3:5..3:11]: mapping [block]
            "integration" [plain @4:7..4:18]: "totalconnect" [plain @4:20..4:32]
            "domain" [plain @5:7..5:13]: "alarm_control_panel" [plain @5:15..5:34]
      "arm_home_instant" [plain @7:1..7:17]: mapping [block]
        "target" [plain @8:3..8:9]: mapping [block]
          "entity" [plain @9:5..9:11]: mapping [block]
            "integration" [plain @10:7..10:18]: "totalconnect" [plain @10:20..10:32]
            "domain" [plain @11:7..11:13]: "alarm_control_panel" [plain @11:15..11:34]
    "#);
}
