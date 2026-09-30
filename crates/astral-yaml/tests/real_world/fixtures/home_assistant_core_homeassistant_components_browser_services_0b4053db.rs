#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("home-assistant-core-homeassistant-components-browser-services-0b4053db"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "browse_url" [plain @1:1..1:11]: mapping [block]
        "fields" [plain @2:3..2:9]: mapping [block]
          "url" [plain @3:5..3:8]: mapping [block]
            "required" [plain @4:7..4:15]: "true" [plain @4:17..4:21]
            "example" [plain @5:7..5:14]: "https://www.home-assistant.io" [double-quoted @5:16..5:47]
            "selector" [plain @6:7..6:15]: mapping [block]
              "text" [plain @7:9..7:13]: "" [plain @7:14..7:14]
    "#);
}
