#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("kubernetes-examples-archived-openshift-origin-openshift-service-3204be8c"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "v1" [plain @1:13..1:15]
      "kind" [plain @2:1..2:5]: "Service" [plain @2:7..2:14]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "openshift" [plain @4:9..4:18]
      "spec" [plain @5:1..5:5]: mapping [block]
        "ports" [plain @6:3..6:8]: sequence [block]
          - mapping [block]
            "name" [plain @7:7..7:11]: "openshift" [plain @7:13..7:22]
            "port" [plain @8:7..8:11]: "8443" [plain @8:13..8:17]
            "targetPort" [plain @9:7..9:17]: "8443" [plain @9:19..9:23]
        "selector" [plain @10:3..10:11]: mapping [block]
          "name" [plain @11:5..11:9]: "openshift" [plain @11:11..11:20]
        "type" [plain @12:3..12:7]: "LoadBalancer" [plain @12:9..12:21]
    "#);
}
