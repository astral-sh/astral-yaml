#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("argoproj-argo-workflows-examples-hello-world-2da01182"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "argoproj.io/v1alpha1" [plain @1:13..1:33]
      "kind" [plain @2:1..2:5]: "Workflow" [plain @2:7..2:15]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "generateName" [plain @4:3..4:15]: "hello-world-" [plain @4:17..4:29]
        "labels" [plain @5:3..5:9]: mapping [block]
          "workflows.argoproj.io/archive-strategy" [plain @6:5..6:43]: "false" [double-quoted @6:45..6:52]
        "annotations" [plain @7:3..7:14]: mapping [block]
          "workflows.argoproj.io/description" [plain @8:5..8:38]: literal [@8:40..10:1]
            "This is a simple hello world example.\n"
      "spec" [plain @10:1..10:5]: mapping [block]
        "entrypoint" [plain @11:3..11:13]: "hello-world" [plain @11:15..11:26]
        "templates" [plain @12:3..12:12]: sequence [block]
          - mapping [block]
            "name" [plain @13:5..13:9]: "hello-world" [plain @13:11..13:22]
            "container" [plain @14:5..14:14]: mapping [block]
              "image" [plain @15:7..15:12]: "busybox" [plain @15:14..15:21]
              "command" [plain @16:7..16:14]: sequence [flow]
                - "echo" [plain @16:17..16:21]
              "args" [plain @17:7..17:11]: sequence [flow]
                - "hello world" [double-quoted @17:14..17:27]
    "#);
}
