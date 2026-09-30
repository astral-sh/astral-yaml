#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("home-assistant-core-homeassistant-components-local-file-services-08afb72c"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "update_file_path" [plain @1:1..1:17]: mapping [block]
        "target" [plain @2:3..2:9]: mapping [block]
          "entity" [plain @3:5..3:11]: mapping [block]
            "integration" [plain @4:7..4:18]: "local_file" [plain @4:20..4:30]
            "domain" [plain @5:7..5:13]: "camera" [plain @5:15..5:21]
        "fields" [plain @6:3..6:9]: mapping [block]
          "file_path" [plain @7:5..7:14]: mapping [block]
            "required" [plain @8:7..8:15]: "true" [plain @8:17..8:21]
            "example" [plain @9:7..9:14]: "/config/www/images/image.jpg" [double-quoted @9:16..9:46]
            "selector" [plain @10:7..10:15]: mapping [block]
              "text" [plain @11:9..11:13]: "" [plain @11:14..11:14]
    "#);
}
