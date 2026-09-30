#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("kubernetes-examples-archived-openshift-origin-openshift-origin-namespace-03bbea5a"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "kind" [plain @1:1..1:5]: "Namespace" [plain @1:7..1:16]
      "apiVersion" [plain @2:1..2:11]: "v1" [plain @2:13..2:15]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "openshift-origin" [double-quoted @4:9..4:27]
        "labels" [plain @5:3..5:9]: mapping [block]
          "name" [plain @6:5..6:9]: "openshift-origin" [double-quoted @6:11..6:29]
    "#);
}
