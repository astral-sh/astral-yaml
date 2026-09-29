#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("kubernetes-examples-archived-podsecuritypolicy-rbac-pod-25043dd6"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "v1" [plain @1:13..1:15]
      "kind" [plain @2:1..2:5]: "Pod" [plain @2:7..2:10]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "nginx" [plain @4:9..4:14]
        "labels" [plain @5:3..5:9]: mapping [block]
          "name" [plain @6:5..6:9]: "nginx" [plain @6:11..6:16]
      "spec" [plain @7:1..7:5]: mapping [block]
        "containers" [plain @8:3..8:13]: sequence [block]
          - mapping [block]
            "name" [plain @9:5..9:9]: "nginx" [plain @9:11..9:16]
            "image" [plain @10:5..10:10]: "nginx" [plain @10:12..10:17]
            "ports" [plain @11:5..11:10]: sequence [block]
              - mapping [block]
                "containerPort" [plain @12:7..12:20]: "80" [plain @12:22..12:24]
    "#);
}
