#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("kubernetes-examples-archived-cpu-manager-exclusive-1-39aa6265"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "v1" [plain @1:13..1:15]
      "kind" [plain @2:1..2:5]: "Pod" [plain @2:7..2:10]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "exclusive-1" [plain @4:9..4:20]
      "spec" [plain @5:1..5:5]: mapping [block]
        "containers" [plain @6:3..6:13]: sequence [block]
          - mapping [block]
            "image" [plain @7:5..7:10]: "quay.io/connordoyle/cpuset-visualizer" [plain @7:12..7:49]
            "name" [plain @8:5..8:9]: "exclusive-1" [plain @8:11..8:22]
            "resources" [plain @9:5..9:14]: mapping [block]
              "requests" [plain @10:7..10:15]: mapping [block]
                "cpu" [plain @11:9..11:12]: "1" [plain @11:14..11:15]
                "memory" [plain @12:9..12:15]: "256M" [double-quoted @12:17..12:23]
              "limits" [plain @13:7..13:13]: mapping [block]
                "cpu" [plain @14:9..14:12]: "1" [plain @14:14..14:15]
                "memory" [plain @15:9..15:15]: "256M" [double-quoted @15:17..15:23]
    "#);
}
