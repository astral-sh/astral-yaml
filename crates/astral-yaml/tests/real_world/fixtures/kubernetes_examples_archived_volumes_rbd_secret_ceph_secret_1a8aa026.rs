#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("kubernetes-examples-archived-volumes-rbd-secret-ceph-secret-1a8aa026"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "v1" [plain @1:13..1:15]
      "kind" [plain @2:1..2:5]: "Secret" [plain @2:7..2:13]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "ceph-secret" [plain @4:9..4:20]
      "type" [plain @5:1..5:5]: "kubernetes.io/rbd" [double-quoted @5:7..5:26]
      "data" [plain @6:1..6:5]: mapping [block]
        "key" [plain @7:3..7:6]: "QVFCMTZWMVZvRjVtRXhBQTVrQ1FzN2JCajhWVUxSdzI2Qzg0SEE9PQ==" [plain @7:8..7:64]
    "#);
}
