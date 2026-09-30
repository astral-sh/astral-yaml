#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("argoproj-argo-workflows-examples-synchronization-mutex-wf-level-22ae0edf"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @3:1..3:11]: "argoproj.io/v1alpha1" [plain @3:13..3:33]
      "kind" [plain @4:1..4:5]: "Workflow" [plain @4:7..4:15]
      "metadata" [plain @5:1..5:9]: mapping [block]
        "generateName" [plain @6:3..6:15]: "synchronization-wf-level-" [plain @6:17..6:42]
      "spec" [plain @7:1..7:5]: mapping [block]
        "entrypoint" [plain @8:3..8:13]: "hello-world" [plain @8:15..8:26]
        "synchronization" [plain @9:3..9:18]: mapping [block]
          "mutexes" [plain @10:5..10:12]: sequence [block]
            - mapping [block]
              "name" [plain @11:9..11:13]: "test" [plain @11:16..11:20]
        "templates" [plain @14:3..14:12]: sequence [block]
          - mapping [block]
            "name" [plain @15:7..15:11]: "hello-world" [plain @15:13..15:24]
            "container" [plain @16:7..16:16]: mapping [block]
              "image" [plain @17:9..17:14]: "busybox" [plain @17:16..17:23]
              "command" [plain @18:9..18:16]: sequence [flow]
                - "echo" [plain @18:19..18:23]
              "args" [plain @19:9..19:13]: sequence [flow]
                - "hello world" [double-quoted @19:16..19:29]
    "#);
}
