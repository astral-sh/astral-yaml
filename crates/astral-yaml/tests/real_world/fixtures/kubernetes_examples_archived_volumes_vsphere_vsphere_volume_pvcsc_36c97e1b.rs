#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("kubernetes-examples-archived-volumes-vsphere-vsphere-volume-pvcsc-36c97e1b"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "kind" [plain @1:1..1:5]: "PersistentVolumeClaim" [plain @1:7..1:28]
      "apiVersion" [plain @2:1..2:11]: "v1" [plain @2:13..2:15]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "pvcsc001" [plain @4:9..4:17]
      "spec" [plain @5:1..5:5]: mapping [block]
        "accessModes" [plain @6:3..6:14]: sequence [block]
          - "ReadWriteOnce" [plain @7:7..7:20]
        "resources" [plain @8:3..8:12]: mapping [block]
          "requests" [plain @9:5..9:13]: mapping [block]
            "storage" [plain @10:7..10:14]: "2Gi" [plain @10:16..10:19]
        "storageClassName" [plain @11:3..11:19]: "fast" [plain @11:21..11:25]
    "#);
}
