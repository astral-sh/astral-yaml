#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("home-assistant-core-homeassistant-components-wemo-services-0d193927"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "set_humidity" [plain @1:1..1:13]: mapping [block]
        "target" [plain @2:3..2:9]: mapping [block]
          "entity" [plain @3:5..3:11]: mapping [block]
            "integration" [plain @4:7..4:18]: "wemo" [plain @4:20..4:24]
            "domain" [plain @5:7..5:13]: "fan" [plain @5:15..5:18]
        "fields" [plain @6:3..6:9]: mapping [block]
          "target_humidity" [plain @7:5..7:20]: mapping [block]
            "required" [plain @8:7..8:15]: "true" [plain @8:17..8:21]
            "selector" [plain @9:7..9:15]: mapping [block]
              "number" [plain @10:9..10:15]: mapping [block]
                "min" [plain @11:11..11:14]: "0" [plain @11:16..11:17]
                "max" [plain @12:11..12:14]: "100" [plain @12:16..12:19]
                "step" [plain @13:11..13:15]: "5" [plain @13:17..13:18]
                "unit_of_measurement" [plain @14:11..14:30]: "%" [double-quoted @14:32..14:35]
      "reset_filter_life" [plain @16:1..16:18]: mapping [block]
        "target" [plain @17:3..17:9]: mapping [block]
          "entity" [plain @18:5..18:11]: mapping [block]
            "integration" [plain @19:7..19:18]: "wemo" [plain @19:20..19:24]
            "domain" [plain @20:7..20:13]: "fan" [plain @20:15..20:18]
    "#);
}
