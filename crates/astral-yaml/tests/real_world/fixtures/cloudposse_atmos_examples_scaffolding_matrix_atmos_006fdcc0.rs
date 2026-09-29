#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("cloudposse-atmos-examples-scaffolding-matrix-atmos-006fdcc0"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "base_path" [plain @5:1..5:10]: "." [double-quoted @5:12..5:15]
      "stacks" [plain @8:1..8:7]: mapping [block]
        "base_path" [plain @10:3..10:12]: "stacks" [double-quoted @10:14..10:22]
        "name_template" [plain @12:3..12:16]: "{{.vars.environment}}" [double-quoted @12:18..12:41]
      "templates" [plain @15:1..15:10]: mapping [block]
        "settings" [plain @16:3..16:11]: mapping [block]
          "enabled" [plain @17:5..17:12]: "true" [plain @17:14..17:18]
          "sprig" [plain @18:5..18:10]: mapping [block]
            "enabled" [plain @19:7..19:14]: "true" [plain @19:16..19:20]
          "gomplate" [plain @20:5..20:13]: mapping [block]
            "enabled" [plain @21:7..21:14]: "true" [plain @21:16..21:20]
      "scaffold" [plain @26:1..26:9]: mapping [block]
        "templates" [plain @27:3..27:12]: mapping [block]
          "example" [plain @28:5..28:12]: mapping [block]
            "description" [plain @29:7..29:18]: "Example scaffold template for demonstration" [plain @29:20..29:63]
            "source" [plain @30:7..30:13]: "." [double-quoted @30:15..30:18]
      "logs" [plain @33:1..33:5]: mapping [block]
        "level" [plain @34:3..34:8]: "info" [plain @34:10..34:14]
    "#);
}
