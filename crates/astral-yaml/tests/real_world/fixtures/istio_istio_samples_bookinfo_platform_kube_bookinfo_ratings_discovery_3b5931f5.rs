#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("istio-istio-samples-bookinfo-platform-kube-bookinfo-ratings-discovery-3b5931f5"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @18:1..18:11]: "v1" [plain @18:13..18:15]
      "kind" [plain @19:1..19:5]: "Service" [plain @19:7..19:14]
      "metadata" [plain @20:1..20:9]: mapping [block]
        "name" [plain @21:3..21:7]: "ratings" [plain @21:9..21:16]
        "labels" [plain @22:3..22:9]: mapping [block]
          "app" [plain @23:5..23:8]: "ratings" [plain @23:10..23:17]
          "service" [plain @24:5..24:12]: "ratings" [plain @24:14..24:21]
      "spec" [plain @25:1..25:5]: mapping [block]
        "ports" [plain @26:3..26:8]: sequence [block]
          - mapping [block]
            "port" [plain @27:5..27:9]: "9080" [plain @27:11..27:15]
            "name" [plain @28:5..28:9]: "http" [plain @28:11..28:15]
        "selector" [plain @29:3..29:11]: mapping [block]
          "app" [plain @30:5..30:8]: "ratings" [plain @30:10..30:17]

    document 2 (start: ---, end: implicit)
    "" [plain @32:1..32:1]
    "#);
}
