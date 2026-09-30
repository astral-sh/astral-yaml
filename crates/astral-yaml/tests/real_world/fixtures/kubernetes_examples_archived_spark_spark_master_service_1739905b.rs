#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("kubernetes-examples-archived-spark-spark-master-service-1739905b"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "kind" [plain @1:1..1:5]: "Service" [plain @1:7..1:14]
      "apiVersion" [plain @2:1..2:11]: "v1" [plain @2:13..2:15]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "spark-master" [plain @4:9..4:21]
      "spec" [plain @5:1..5:5]: mapping [block]
        "ports" [plain @6:3..6:8]: sequence [block]
          - mapping [block]
            "port" [plain @7:7..7:11]: "7077" [plain @7:13..7:17]
            "targetPort" [plain @8:7..8:17]: "7077" [plain @8:19..8:23]
            "name" [plain @9:7..9:11]: "spark" [plain @9:13..9:18]
          - mapping [block]
            "port" [plain @10:7..10:11]: "8080" [plain @10:13..10:17]
            "targetPort" [plain @11:7..11:17]: "8080" [plain @11:19..11:23]
            "name" [plain @12:7..12:11]: "http" [plain @12:13..12:17]
        "selector" [plain @13:3..13:11]: mapping [block]
          "component" [plain @14:5..14:14]: "spark-master" [plain @14:16..14:28]
    "#);
}
