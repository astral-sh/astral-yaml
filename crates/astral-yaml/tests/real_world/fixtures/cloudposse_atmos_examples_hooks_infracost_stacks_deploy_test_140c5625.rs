#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("cloudposse-atmos-examples-hooks-infracost-stacks-deploy-test-140c5625"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "vars" [plain @3:1..3:5]: mapping [block]
        "stage" [plain @4:3..4:8]: "test" [plain @4:10..4:14]
      "components" [plain @6:1..6:11]: mapping [block]
        "terraform" [plain @7:3..7:12]: mapping [block]
          "nat-gateway" [plain @8:5..8:16]: mapping [block]
            "vars" [plain @9:7..9:11]: mapping [block]
              "environment" [plain @10:9..10:20]: "test" [plain @10:22..10:26]
            "dependencies" [plain @11:7..11:19]: mapping [block]
              "tools" [plain @12:9..12:14]: mapping [block]
                "infracost" [plain @13:11..13:20]: "0.10.44" [double-quoted @13:22..13:31]
            "hooks" [plain @14:7..14:12]: mapping [block]
              "cost" [plain @15:9..15:13]: mapping [block]
                "events" [plain @16:11..16:17]: sequence [block]
                  - "after.terraform.plan" [plain @17:15..17:35]
                "kind" [plain @18:11..18:15]: "infracost" [plain @18:17..18:26]
    "#);
}
