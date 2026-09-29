#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("kubernetes-examples-archived-volumes-vsphere-vsphere-volume-spbm-policy-with-datastore-08f1ccdc"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "kind" [plain @1:1..1:5]: "StorageClass" [plain @1:7..1:19]
      "apiVersion" [plain @2:1..2:11]: "storage.k8s.io/v1" [plain @2:13..2:30]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "fast" [plain @4:9..4:13]
      "provisioner" [plain @5:1..5:12]: "kubernetes.io/vsphere-volume" [plain @5:14..5:42]
      "parameters" [plain @6:1..6:11]: mapping [block]
        "diskformat" [plain @7:5..7:15]: "zeroedthick" [plain @7:17..7:28]
        "storagePolicyName" [plain @8:5..8:22]: "gold" [plain @8:24..8:28]
        "datastore" [plain @9:5..9:14]: "VSANDatastore" [plain @9:16..9:29]
    "#);
}
