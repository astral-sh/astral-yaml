#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("istio-istio-samples-ambient-argo-application-reviews-waypoint-3ad6b960"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "gateway.networking.k8s.io/v1" [plain @1:13..1:41]
      "kind" [plain @2:1..2:5]: "Gateway" [plain @2:7..2:14]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "labels" [plain @4:3..4:9]: mapping [block]
          "istio.io/rev" [plain @5:5..5:17]: "rapid" [plain @5:19..5:24]
        "name" [plain @6:3..6:7]: "bookinfo-reviews" [plain @6:9..6:25]
      "spec" [plain @7:1..7:5]: mapping [block]
        "gatewayClassName" [plain @8:3..8:19]: "istio-waypoint" [plain @8:21..8:35]
        "listeners" [plain @9:3..9:12]: sequence [block]
          - mapping [block]
            "name" [plain @10:5..10:9]: "mesh" [plain @10:11..10:15]
            "port" [plain @11:5..11:9]: "15008" [plain @11:11..11:16]
            "protocol" [plain @12:5..12:13]: "HBONE" [plain @12:15..12:20]
    "#);
}
