#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("kubernetes-examples-archived-volumes-storageos-storageos-pv-38ba7d48"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "v1" [plain @1:13..1:15]
      "kind" [plain @2:1..2:5]: "PersistentVolume" [plain @2:7..2:23]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "pv0001" [plain @4:9..4:15]
      "spec" [plain @5:1..5:5]: mapping [block]
        "capacity" [plain @6:3..6:11]: mapping [block]
          "storage" [plain @7:5..7:12]: "5Gi" [plain @7:14..7:17]
        "accessModes" [plain @8:3..8:14]: sequence [block]
          - "ReadWriteOnce" [plain @9:7..9:20]
        "persistentVolumeReclaimPolicy" [plain @10:3..10:32]: "Delete" [plain @10:34..10:40]
        "storageClassName" [plain @11:3..11:19]: "fast" [plain @11:21..11:25]
        "storageos" [plain @12:3..12:12]: mapping [block]
          "volumeName" [plain @14:5..14:15]: "pv0001" [plain @14:17..14:23]
          "fsType" [plain @20:5..20:11]: "ext4" [plain @20:13..20:17]
          "secretName" [plain @22:5..22:15]: "storageos-secret" [plain @22:17..22:33]
    "#);
}
