#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("cloudposse-atmos-examples-kustomize-stacks-catalog-emulator-kubernetes-1d370952"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "components" [plain @9:1..9:11]: mapping [block]
        "emulator" [plain @10:3..10:11]: mapping [block]
          "kubernetes" [plain @11:5..11:15]: mapping [block]
            "driver" [plain @12:7..12:13]: "k3s" [plain @12:15..12:18]
            "ephemeral" [plain @18:7..18:16]: "true" [plain @18:18..18:22]
    "#);
}
