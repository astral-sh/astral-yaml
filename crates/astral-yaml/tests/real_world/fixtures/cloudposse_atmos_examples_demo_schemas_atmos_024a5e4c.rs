#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("cloudposse-atmos-examples-demo-schemas-atmos-024a5e4c"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "schemas" [plain @1:1..1:8]: mapping [block]
        "schemaFromFile" [plain @2:3..2:17]: mapping [block]
          "manifest" [plain @3:5..3:13]: "./manifest.json" [plain @3:15..3:30]
          "matches" [plain @4:5..4:12]: sequence [block]
            - "config.yaml" [plain @5:9..5:20]
        "schemaFromInternet" [plain @6:3..6:21]: mapping [block]
          "manifest" [plain @7:5..7:13]: "https://json.schemastore.org/bower.json" [plain @7:15..7:54]
          "matches" [plain @8:5..8:12]: sequence [block]
            - "bower.yaml" [plain @9:9..9:19]
        "schemaFromInline" [plain @10:3..10:19]: mapping [block]
          "manifest" [plain @11:5..11:13]: literal [@11:15..20:1]
            "{\n"
            "  \"type\": \"object\",\n"
            "  \"properties\": {\n"
            "    \"name\": { \"type\": \"string\" },\n"
            "    \"age\": { \"type\": \"integer\" }\n"
            "  },\n"
            "  \"required\": [\"name\"]\n"
            "}\n"
          "matches" [plain @20:5..20:12]: sequence [block]
            - "inline.yaml" [plain @21:9..21:20]
    "#);
}
