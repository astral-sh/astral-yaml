#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("cloudposse-atmos-examples-native-terraform-stacks-dev-0baf21d3"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "name" [plain @3:1..3:5]: "dev" [plain @3:7..3:10]
      "import" [plain @5:1..5:7]: sequence [block]
        - "_defaults" [plain @6:5..6:14]
      "vars" [plain @8:1..8:5]: mapping [block]
        "environment" [plain @9:3..9:14]: "dev" [plain @9:16..9:19]
      "components" [plain @11:1..11:11]: mapping [block]
        "terraform" [plain @12:3..12:12]: mapping [block]
          "vpc" [plain @13:5..13:8]: mapping [block]
            "vars" [plain @14:7..14:11]: "../components/terraform/vpc/envs/dev.tfvars" [plain @14:22..14:65] !<!include>
            "flags" [plain @17:7..17:12]: mapping [block]
              "parallelism" [plain @18:9..18:20]: "4" [plain @18:22..18:23]
    "#);
}
