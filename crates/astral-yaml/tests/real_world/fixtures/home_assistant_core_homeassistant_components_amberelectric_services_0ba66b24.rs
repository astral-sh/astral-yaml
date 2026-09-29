#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("home-assistant-core-homeassistant-components-amberelectric-services-0ba66b24"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "get_forecasts" [plain @1:1..1:14]: mapping [block]
        "fields" [plain @2:3..2:9]: mapping [block]
          "config_entry_id" [plain @3:5..3:20]: mapping [block]
            "required" [plain @4:7..4:15]: "true" [plain @4:17..4:21]
            "selector" [plain @5:7..5:15]: mapping [block]
              "config_entry" [plain @6:9..6:21]: mapping [block]
                "integration" [plain @7:11..7:22]: "amberelectric" [plain @7:24..7:37]
          "channel_type" [plain @8:5..8:17]: mapping [block]
            "required" [plain @9:7..9:15]: "true" [plain @9:17..9:21]
            "selector" [plain @10:7..10:15]: mapping [block]
              "select" [plain @11:9..11:15]: mapping [block]
                "options" [plain @12:11..12:18]: sequence [block]
                  - "general" [plain @13:15..13:22]
                  - "controlled_load" [plain @14:15..14:30]
                  - "feed_in" [plain @15:15..15:22]
                "translation_key" [plain @16:11..16:26]: "channel_type" [plain @16:28..16:40]
    "#);
}
