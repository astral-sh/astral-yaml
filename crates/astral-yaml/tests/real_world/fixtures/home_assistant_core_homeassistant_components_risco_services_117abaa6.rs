#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("home-assistant-core-homeassistant-components-risco-services-117abaa6"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "set_time" [plain @1:1..1:9]: mapping [block]
        "fields" [plain @2:3..2:9]: mapping [block]
          "config_entry_id" [plain @3:5..3:20]: mapping [block]
            "required" [plain @4:7..4:15]: "true" [plain @4:17..4:21]
            "selector" [plain @5:7..5:15]: mapping [block]
              "config_entry" [plain @6:9..6:21]: mapping [block]
                "integration" [plain @7:11..7:22]: "risco" [plain @7:24..7:29]
          "time" [plain @8:5..8:9]: mapping [block]
            "required" [plain @9:7..9:15]: "false" [plain @9:17..9:22]
            "selector" [plain @10:7..10:15]: mapping [block]
              "datetime" [plain @11:9..11:17]: "" [plain @11:18..11:18]
    "#);
}
