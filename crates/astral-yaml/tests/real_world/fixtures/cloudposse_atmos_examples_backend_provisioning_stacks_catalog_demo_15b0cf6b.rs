#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("cloudposse-atmos-examples-backend-provisioning-stacks-catalog-demo-15b0cf6b"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "components" [plain @3:1..3:11]: mapping [block]
        "terraform" [plain @4:3..4:12]: mapping [block]
          "demo" [plain @5:5..5:9]: mapping [block]
            "metadata" [plain @6:7..6:15]: mapping [block]
              "type" [plain @7:9..7:13]: "abstract" [plain @7:15..7:23]
              "component" [plain @8:9..8:18]: "bucket" [plain @8:20..8:26]
            "backend_type" [plain @13:7..13:19]: "s3" [plain @13:21..13:23]
            "backend" [plain @14:7..14:14]: mapping [block]
              "s3" [plain @15:9..15:11]: mapping [block]
                "key" [plain @16:11..16:14]: "demo/terraform.tfstate" [plain @16:16..16:38]
                "region" [plain @17:11..17:17]: "us-east-1" [plain @17:19..17:28]
                "use_path_style" [plain @21:11..21:25]: "true" [plain @21:27..21:31]
            "provision" [plain @22:7..22:16]: mapping [block]
              "backend" [plain @23:9..23:16]: mapping [block]
                "enabled" [plain @24:11..24:18]: "true" [plain @24:20..24:24]
            "vars" [plain @25:7..25:11]: mapping [flow]
    "#);
}
