#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("home-assistant-core-homeassistant-components-remember-the-milk-services-0f8c17e4"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "create_task" [plain @3:1..3:12]: mapping [block]
        "fields" [plain @4:3..4:9]: mapping [block]
          "name" [plain @5:5..5:9]: mapping [block]
            "required" [plain @6:7..6:15]: "true" [plain @6:17..6:21]
            "example" [plain @7:7..7:14]: "do this ^today #from_hass" [double-quoted @7:16..7:43]
            "selector" [plain @8:7..8:15]: mapping [block]
              "text" [plain @9:9..9:13]: "" [plain @9:14..9:14]
          "id" [plain @10:5..10:7]: mapping [block]
            "example" [plain @11:7..11:14]: "myid" [plain @11:16..11:20]
            "selector" [plain @12:7..12:15]: mapping [block]
              "text" [plain @13:9..13:13]: "" [plain @13:14..13:14]
      "complete_task" [plain @15:1..15:14]: mapping [block]
        "fields" [plain @16:3..16:9]: mapping [block]
          "id" [plain @17:5..17:7]: mapping [block]
            "required" [plain @18:7..18:15]: "true" [plain @18:17..18:21]
            "example" [plain @19:7..19:14]: "myid" [plain @19:16..19:20]
            "selector" [plain @20:7..20:15]: mapping [block]
              "text" [plain @21:9..21:13]: "" [plain @21:14..21:14]
    "#);
}
