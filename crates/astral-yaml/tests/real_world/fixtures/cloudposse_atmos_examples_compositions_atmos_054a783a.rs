#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("cloudposse-atmos-examples-compositions-atmos-054a783a"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "base_path" [plain @1:1..1:10]: "." [double-quoted @1:12..1:15]
      "stacks" [plain @3:1..3:7]: mapping [block]
        "base_path" [plain @4:3..4:12]: "stacks" [double-quoted @4:14..4:22]
        "included_paths" [plain @5:3..5:17]: sequence [block]
          - "deploy/**/*" [double-quoted @6:7..6:20]
        "name_template" [plain @7:3..7:16]: "{{ .vars.stage }}" [double-quoted @7:18..7:37]
      "components" [plain @9:1..9:11]: mapping [block]
        "container" [plain @10:3..10:12]: mapping [block]
          "base_path" [plain @11:5..11:14]: "components/container" [double-quoted @11:16..11:38]
      "container" [plain @13:1..13:10]: mapping [block]
        "runtime" [plain @14:3..14:10]: mapping [block]
          "provider" [plain @15:5..15:13]: "auto" [plain @15:15..15:19]
      "compositions" [plain @23:1..23:13]: mapping [block]
        "storefront" [plain @24:3..24:13]: mapping [block]
          "description" [plain @25:5..25:16]: "Storefront frontend, API, and database" [plain @25:18..25:56]
          "services" [plain @26:5..26:13]: sequence [block]
            - "frontend" [plain @27:9..27:17]
            - "api" [plain @28:9..28:12]
            - "database" [plain @29:9..29:17]
    "#);
}
