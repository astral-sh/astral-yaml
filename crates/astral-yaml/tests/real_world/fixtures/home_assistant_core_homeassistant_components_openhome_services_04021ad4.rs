#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("home-assistant-core-homeassistant-components-openhome-services-04021ad4"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "invoke_pin" [plain @3:1..3:11]: mapping [block]
        "target" [plain @4:3..4:9]: mapping [block]
          "entity" [plain @5:5..5:11]: mapping [block]
            "integration" [plain @6:7..6:18]: "openhome" [plain @6:20..6:28]
            "domain" [plain @7:7..7:13]: "media_player" [plain @7:15..7:27]
        "fields" [plain @8:3..8:9]: mapping [block]
          "pin" [plain @9:5..9:8]: mapping [block]
            "required" [plain @10:7..10:15]: "true" [plain @10:17..10:21]
            "selector" [plain @11:7..11:15]: mapping [block]
              "number" [plain @12:9..12:15]: mapping [block]
                "min" [plain @13:11..13:14]: "0" [plain @13:16..13:17]
                "max" [plain @14:11..14:14]: "1000" [plain @14:16..14:20]
    "#);
}
