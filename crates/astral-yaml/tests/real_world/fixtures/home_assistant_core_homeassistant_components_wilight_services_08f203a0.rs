#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("home-assistant-core-homeassistant-components-wilight-services-08f203a0"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "set_watering_time" [plain @1:1..1:18]: mapping [block]
        "target" [plain @2:3..2:9]: "" [plain @2:10..2:10]
        "fields" [plain @3:3..3:9]: mapping [block]
          "watering_time" [plain @4:5..4:18]: mapping [block]
            "example" [plain @5:7..5:14]: "30" [plain @5:16..5:18]
      "set_pause_time" [plain @6:1..6:15]: mapping [block]
        "target" [plain @7:3..7:9]: "" [plain @7:10..7:10]
        "fields" [plain @8:3..8:9]: mapping [block]
          "pause_time" [plain @9:5..9:15]: mapping [block]
            "example" [plain @10:7..10:14]: "24" [plain @10:16..10:18]
      "set_trigger" [plain @11:1..11:12]: mapping [block]
        "target" [plain @12:3..12:9]: "" [plain @12:10..12:10]
        "fields" [plain @13:3..13:9]: mapping [block]
          "trigger_index" [plain @14:5..14:18]: mapping [block]
            "example" [plain @15:7..15:14]: "1" [double-quoted @15:16..15:19]
          "trigger" [plain @16:5..16:12]: mapping [block]
            "example" [plain @17:7..17:14]: "'12707001'" [double-quoted @17:16..17:28]
    "#);
}
