#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("kubernetes-examples-archived-volumes-azure-disk-claim-blob-based-disk-account-specified-hdd-storageclass-account-hdd-192eae65"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "kind" [plain @1:1..1:5]: "StorageClass" [plain @1:7..1:19]
      "apiVersion" [plain @2:1..2:11]: "storage.k8s.io/v1beta1" [plain @2:13..2:35]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "accounthdd" [plain @4:9..4:19]
      "provisioner" [plain @5:1..5:12]: "kubernetes.io/azure-disk" [plain @5:14..5:38]
      "parameters" [plain @6:1..6:11]: mapping [block]
        "skuname" [plain @7:3..7:10]: "Standard_LRS" [plain @7:12..7:24]
        "location" [plain @8:3..8:11]: "westus" [plain @8:13..8:19]
        "storageAccount" [plain @9:3..9:17]: "azuretestx" [plain @9:19..9:29]
    "#);
}
