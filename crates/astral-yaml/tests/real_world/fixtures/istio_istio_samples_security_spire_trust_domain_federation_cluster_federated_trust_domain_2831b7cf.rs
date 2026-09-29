#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("istio-istio-samples-security-spire-trust-domain-federation-cluster-federated-trust-domain-2831b7cf"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "spire.spiffe.io/v1alpha1" [plain @1:13..1:37]
      "kind" [plain @2:1..2:5]: "ClusterFederatedTrustDomain" [plain @2:7..2:34]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "${CLUSTER}" [plain @4:9..4:19]
      "spec" [plain @5:1..5:5]: mapping [block]
        "className" [plain @6:3..6:12]: "spire-spire" [plain @6:14..6:25]
        "trustDomain" [plain @7:3..7:14]: "${CLUSTER}.local" [plain @7:16..7:32]
        "bundleEndpointURL" [plain @8:3..8:20]: "https://${BUNDLE_ENDPOINT}:8443" [plain @8:22..8:53]
        "bundleEndpointProfile" [plain @9:3..9:24]: mapping [block]
          "type" [plain @10:5..10:9]: "https_spiffe" [plain @10:11..10:23]
          "endpointSPIFFEID" [plain @11:5..11:21]: "spiffe://${CLUSTER}.local/spire/server" [plain @11:23..11:61]
    "#);
}
