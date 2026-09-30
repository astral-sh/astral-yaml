#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("argoproj-argo-workflows-examples-testvolume-04c8c1db"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "kind" [plain @5:1..5:5]: "PersistentVolumeClaim" [plain @5:7..5:28]
      "apiVersion" [plain @6:1..6:11]: "v1" [plain @6:13..6:15]
      "metadata" [plain @7:1..7:9]: mapping [block]
        "name" [plain @8:3..8:7]: "my-existing-volume" [plain @8:9..8:27]
      "spec" [plain @9:1..9:5]: mapping [block]
        "accessModes" [plain @10:3..10:14]: sequence [flow]
          - "ReadWriteOnce" [double-quoted @10:18..10:33]
        "resources" [plain @11:3..11:12]: mapping [block]
          "requests" [plain @12:5..12:13]: mapping [block]
            "storage" [plain @13:7..13:14]: "1Gi" [plain @13:16..13:19]
    "#);
}
