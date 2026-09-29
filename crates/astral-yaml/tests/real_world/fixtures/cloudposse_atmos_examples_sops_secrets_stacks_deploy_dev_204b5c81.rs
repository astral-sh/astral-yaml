#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("cloudposse-atmos-examples-sops-secrets-stacks-deploy-dev-204b5c81"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "vars" [plain @3:1..3:5]: mapping [block]
        "stage" [plain @4:3..4:8]: "dev" [plain @4:10..4:13]
      "secrets" [plain @13:1..13:8]: mapping [block]
        "providers" [plain @14:3..14:12]: mapping [block]
          "dev-sops" [plain @15:5..15:13]: mapping [block]
            "kind" [plain @16:7..16:11]: "sops/age" [plain @16:13..16:21]
            "spec" [plain @17:7..17:11]: mapping [block]
              "file" [plain @18:9..18:13]: "secrets/dev.enc.yaml" [plain @18:15..18:35]
              "age_key_file" [plain @19:9..19:21]: "secrets/keys.txt" [plain @19:23..19:39]
      "import" [plain @21:1..21:7]: sequence [block]
        - "catalog/api" [plain @22:5..22:16]
    "#);
}
