#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("argoproj-argo-workflows-examples-pod-spec-patch-0d034d5a"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "argoproj.io/v1alpha1" [plain @1:13..1:33]
      "kind" [plain @2:1..2:5]: "Workflow" [plain @2:7..2:15]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "generateName" [plain @4:3..4:15]: "pod-spec-patch-" [plain @4:17..4:32]
      "spec" [plain @5:1..5:5]: mapping [block]
        "entrypoint" [plain @6:3..6:13]: "hello-world" [plain @6:15..6:26]
        "arguments" [plain @7:3..7:12]: mapping [block]
          "parameters" [plain @8:5..8:15]: sequence [block]
            - mapping [block]
              "name" [plain @9:9..9:13]: "cpu-limit" [plain @9:15..9:24]
              "value" [plain @10:9..10:14]: "100m" [plain @10:16..10:20]
        "templates" [plain @11:3..11:12]: sequence [block]
          - mapping [block]
            "name" [plain @12:5..12:9]: "hello-world" [plain @12:11..12:22]
            "podSpecPatch" [plain @13:5..13:17]: "{\"containers\":[{\"name\":\"main\", \"resources\":{\"limits\":{\"cpu\": \"{{workflow.parameters.cpu-limit}}\" }}}]}" [single-quoted @13:19..13:123]
            "container" [plain @14:5..14:14]: mapping [block]
              "image" [plain @15:7..15:12]: "busybox" [plain @15:14..15:21]
              "command" [plain @16:7..16:14]: sequence [flow]
                - "echo" [plain @16:17..16:21]
              "args" [plain @17:7..17:11]: sequence [flow]
                - "hello world" [double-quoted @17:14..17:27]
    "#);
}
