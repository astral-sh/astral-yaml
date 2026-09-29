#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("cloudposse-atmos-examples-version-tracker-stacks-deploy-prod-0305af67"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "version" [plain @3:1..3:8]: mapping [block]
        "track" [plain @4:3..4:8]: "prod" [plain @4:10..4:14]
      "vars" [plain @6:1..6:5]: mapping [block]
        "stage" [plain @7:3..7:8]: "prod" [plain @7:10..7:14]
      "components" [plain @9:1..9:11]: mapping [block]
        "terraform" [plain @10:3..10:12]: mapping [block]
          "app" [plain @11:5..11:8]: mapping [block]
            "vars" [plain @12:7..12:11]: mapping [block]
              "environment" [plain @13:9..13:20]: "prod" [plain @13:22..13:26]
              "kubectl_version" [plain @14:9..14:24]: "kubectl" [plain @14:35..14:42] !<!version>
              "redis_image" [plain @15:9..15:20]: "redis:{{ .version.redis }}" [double-quoted @15:22..15:50]
              "ci_action_ref" [plain @16:9..16:22]: "setup_node" [plain @16:33..16:43] !<!version>
    "#);
}
