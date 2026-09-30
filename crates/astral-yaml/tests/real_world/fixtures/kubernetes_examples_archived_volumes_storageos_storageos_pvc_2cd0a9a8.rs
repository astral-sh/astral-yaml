#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("kubernetes-examples-archived-volumes-storageos-storageos-pvc-2cd0a9a8"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "v1" [plain @1:13..1:15]
      "kind" [plain @2:1..2:5]: "PersistentVolumeClaim" [plain @2:7..2:28]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "pvc0001" [plain @4:9..4:16]
      "spec" [plain @5:1..5:5]: mapping [block]
        "accessModes" [plain @6:3..6:14]: sequence [block]
          - "ReadWriteOnce" [plain @7:7..7:20]
        "resources" [plain @8:3..8:12]: mapping [block]
          "requests" [plain @9:5..9:13]: mapping [block]
            "storage" [plain @10:7..10:14]: "5Gi" [plain @10:16..10:19]
        "storageClassName" [plain @11:3..11:19]: "fast" [plain @11:21..11:25]
    "#);
}
