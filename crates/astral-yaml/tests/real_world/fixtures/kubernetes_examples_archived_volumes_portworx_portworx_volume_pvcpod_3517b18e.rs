#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("kubernetes-examples-archived-volumes-portworx-portworx-volume-pvcpod-3517b18e"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "v1" [plain @1:13..1:15]
      "kind" [plain @2:1..2:5]: "Pod" [plain @2:7..2:10]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "pvpod" [plain @4:9..4:14]
      "spec" [plain @5:1..5:5]: mapping [block]
        "containers" [plain @6:3..6:13]: sequence [block]
          - mapping [block]
            "name" [plain @7:5..7:9]: "test-container" [plain @7:11..7:25]
            "image" [plain @8:5..8:10]: "registry.k8s.io/test-webserver" [plain @8:12..8:42]
            "volumeMounts" [plain @9:5..9:17]: sequence [block]
              - mapping [block]
                "name" [plain @10:7..10:11]: "test-volume" [plain @10:13..10:24]
                "mountPath" [plain @11:7..11:16]: "/test-portworx-volume" [plain @11:18..11:39]
        "volumes" [plain @12:3..12:10]: sequence [block]
          - mapping [block]
            "name" [plain @13:5..13:9]: "test-volume" [plain @13:11..13:22]
            "persistentVolumeClaim" [plain @14:5..14:26]: mapping [block]
              "claimName" [plain @15:7..15:16]: "pvc0001" [plain @15:18..15:25]
    "#);
}
