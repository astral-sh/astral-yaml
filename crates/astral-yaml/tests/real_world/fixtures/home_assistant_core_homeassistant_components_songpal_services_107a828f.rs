#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("home-assistant-core-homeassistant-components-songpal-services-107a828f"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "set_sound_setting" [plain @1:1..1:18]: mapping [block]
        "target" [plain @2:3..2:9]: mapping [block]
          "entity" [plain @3:5..3:11]: mapping [block]
            "integration" [plain @4:7..4:18]: "songpal" [plain @4:20..4:27]
            "domain" [plain @5:7..5:13]: "media_player" [plain @5:15..5:27]
        "fields" [plain @6:3..6:9]: mapping [block]
          "name" [plain @7:5..7:9]: mapping [block]
            "required" [plain @8:7..8:15]: "true" [plain @8:17..8:21]
            "example" [plain @9:7..9:14]: "nightMode" [double-quoted @9:16..9:27]
            "selector" [plain @10:7..10:15]: mapping [block]
              "text" [plain @11:9..11:13]: "" [plain @11:14..11:14]
          "value" [plain @12:5..12:10]: mapping [block]
            "required" [plain @13:7..13:15]: "true" [plain @13:17..13:21]
            "example" [plain @14:7..14:14]: "on" [double-quoted @14:16..14:20]
            "selector" [plain @15:7..15:15]: mapping [block]
              "text" [plain @16:9..16:13]: "" [plain @16:14..16:14]
    "#);
}
