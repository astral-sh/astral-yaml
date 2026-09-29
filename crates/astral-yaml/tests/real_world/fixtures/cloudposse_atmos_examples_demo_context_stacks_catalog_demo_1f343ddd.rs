#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("cloudposse-atmos-examples-demo-context-stacks-catalog-demo-1f343ddd"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "components" [plain @3:1..3:11]: mapping [block]
        "terraform" [plain @4:3..4:12]: mapping [block]
          "demo" [plain @5:5..5:9]: mapping [block]
            "metadata" [plain @6:7..6:15]: mapping [block]
              "type" [plain @7:9..7:13]: "abstract" [plain @7:15..7:23]
              "component" [plain @8:9..8:18]: "petset" [plain @8:20..8:26]
            "providers" [plain @9:7..9:16]: mapping [block]
              "context" [plain @10:9..10:16]: mapping [block]
                "values" [plain @11:11..11:17]: mapping [block]
                  "name" [plain @12:13..12:17]: "petset" [plain @12:19..12:25]
            "vars" [plain @13:7..13:11]: mapping [block]
              "pet" [plain @14:9..14:12]: "dog" [double-quoted @14:14..14:19]
    "#);
}
