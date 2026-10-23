#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("espanso-uuid"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "matches" [plain @1:1..1:8]: sequence [block]
        - mapping [block]
          "trigger" [plain @3:5..3:12]: ":uuid" [double-quoted @3:14..3:21]
          "replace" [plain @4:5..4:12]: "{{uuid}}" [double-quoted @4:14..4:24]
          "propagate_case" [plain @5:5..5:19]: "true" [plain @5:21..5:25]
          "uppercase_style" [plain @6:5..6:20]: "uppercase" [plain @6:22..6:31]
          "vars" [plain @7:5..7:9]: sequence [block]
            - mapping [block]
              "name" [plain @8:9..8:13]: "uuid" [plain @8:15..8:19]
              "type" [plain @9:9..9:13]: "script" [plain @9:15..9:21]
              "params" [plain @10:9..10:15]: mapping [block]
                "args" [plain @11:11..11:15]: sequence [block]
                  - "python" [plain @12:15..12:21]
                  - "-c" [plain @13:15..13:17]
                  - literal [@14:15..18:1]
                    "import uuid\n"
                    "print(uuid.uuid4())\n"
        - mapping [block]
          "trigger" [plain @19:5..19:12]: ":nuuid" [double-quoted @19:14..19:22]
          "replace" [plain @20:5..20:12]: "{{uuid_nodash}}" [double-quoted @20:14..20:31]
          "propagate_case" [plain @21:5..21:19]: "true" [plain @21:21..21:25]
          "uppercase_style" [plain @22:5..22:20]: "uppercase" [plain @22:22..22:31]
          "vars" [plain @23:5..23:9]: sequence [block]
            - mapping [block]
              "name" [plain @24:9..24:13]: "uuid_nodash" [plain @24:15..24:26]
              "type" [plain @25:9..25:13]: "script" [plain @25:15..25:21]
              "params" [plain @26:9..26:15]: mapping [block]
                "args" [plain @27:11..27:15]: sequence [block]
                  - "python" [plain @28:15..28:21]
                  - "-c" [plain @29:15..29:17]
                  - literal [@30:15..32:56]
                    "import uuid\n"
                    "print(str(uuid.uuid4()).replace('-', ''))\n"
    "#);
}
