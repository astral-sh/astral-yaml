#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("cloudposse-atmos-examples-terraform-tests-stacks-deploy-fixtures-app-1d2f5177"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "import" [plain @3:1..3:7]: sequence [block]
        - "catalog/emulator/aws" [plain @4:5..4:25]
        - "catalog/app" [plain @5:5..5:16]
      "vars" [plain @7:1..7:5]: mapping [block]
        "stage" [plain @8:3..8:8]: "fixtures" [plain @8:10..8:18]
      "components" [plain @10:1..10:11]: mapping [block]
        "terraform" [plain @11:3..11:12]: mapping [block]
          "vpc" [plain @12:5..12:8]: mapping [block]
            "metadata" [plain @13:7..13:15]: mapping [block]
              "type" [plain @14:9..14:13]: "real" [plain @14:15..14:19]
          "app" [plain @15:5..15:8]: mapping [block]
            "metadata" [plain @16:7..16:15]: mapping [block]
              "type" [plain @17:9..17:13]: "real" [plain @17:15..17:19]
            "vars" [plain @18:7..18:11]: mapping [block]
              "environment" [plain @19:9..19:20]: "fixtures" [plain @19:22..19:30]
            "test" [plain @20:7..20:11]: mapping [block]
              "vars" [plain @21:9..21:13]: mapping [block]
                "expected_bucket_name" [plain @22:11..22:31]: "atmos-demo-test" [plain @22:33..22:48]
                "fixture_vpc_id" [plain @23:11..23:25]: "vpc fixtures vpc_id" [plain @23:44..23:63] !<!terraform.state>
    "#);
}
