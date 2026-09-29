#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("istio-istio-samples-bookinfo-demo-profile-no-gateways-2e635fb6"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @2:1..2:11]: "install.istio.io/v1alpha1" [plain @2:13..2:38]
      "kind" [plain @3:1..3:5]: "IstioOperator" [plain @3:7..3:20]
      "spec" [plain @4:1..4:5]: mapping [block]
        "profile" [plain @5:3..5:10]: "demo" [plain @5:12..5:16]
        "components" [plain @6:3..6:13]: mapping [block]
          "ingressGateways" [plain @7:5..7:20]: sequence [block]
            - mapping [block]
              "name" [plain @8:7..8:11]: "istio-ingressgateway" [plain @8:13..8:33]
              "enabled" [plain @9:7..9:14]: "false" [plain @9:16..9:21]
          "egressGateways" [plain @10:5..10:19]: sequence [block]
            - mapping [block]
              "name" [plain @11:7..11:11]: "istio-egressgateway" [plain @11:13..11:32]
              "enabled" [plain @12:7..12:14]: "false" [plain @12:16..12:21]
    "#);
}
