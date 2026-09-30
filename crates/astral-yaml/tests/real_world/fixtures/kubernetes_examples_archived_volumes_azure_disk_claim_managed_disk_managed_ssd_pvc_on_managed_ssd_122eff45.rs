#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("kubernetes-examples-archived-volumes-azure-disk-claim-managed-disk-managed-ssd-pvc-on-managed-ssd-122eff45"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "kind" [plain @1:1..1:5]: "PersistentVolumeClaim" [plain @1:7..1:28]
      "apiVersion" [plain @2:1..2:11]: "v1" [plain @2:13..2:15]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "dd-managed-ssd-5g" [plain @4:9..4:26]
        "annotations" [plain @5:3..5:14]: mapping [block]
          "volume.beta.kubernetes.io/storage-class" [plain @6:5..6:44]: "managedssd" [plain @6:46..6:56]
      "spec" [plain @7:1..7:5]: mapping [block]
        "accessModes" [plain @8:3..8:14]: sequence [block]
          - "ReadWriteOnce" [plain @9:5..9:18]
        "resources" [plain @10:3..10:12]: mapping [block]
          "requests" [plain @11:5..11:13]: mapping [block]
            "storage" [plain @12:7..12:14]: "5Gi" [plain @12:16..12:19]
    "#);
}
