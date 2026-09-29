#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("home-assistant-core-homeassistant-components-omie-services-0d6e15cd"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "get_prices_for_date" [plain @1:1..1:20]: mapping [block]
        "fields" [plain @2:3..2:9]: mapping [block]
          "date" [plain @3:5..3:9]: mapping [block]
            "required" [plain @4:7..4:15]: "true" [plain @4:17..4:21]
            "selector" [plain @5:7..5:15]: mapping [block]
              "date" [plain @6:9..6:13]: "" [plain @6:14..6:14]
          "countries" [plain @7:5..7:14]: mapping [block]
            "required" [plain @8:7..8:15]: "true" [plain @8:17..8:21]
            "default" [plain @9:7..9:14]: sequence [block]
              - "es" [plain @10:11..10:13]
              - "pt" [plain @11:11..11:13]
            "selector" [plain @12:7..12:15]: mapping [block]
              "select" [plain @13:9..13:15]: mapping [block]
                "multiple" [plain @14:11..14:19]: "true" [plain @14:21..14:25]
                "options" [plain @15:11..15:18]: sequence [block]
                  - "es" [plain @16:15..16:17]
                  - "pt" [plain @17:15..17:17]
                "translation_key" [plain @18:11..18:26]: "country" [plain @18:28..18:35]
    "#);
}
