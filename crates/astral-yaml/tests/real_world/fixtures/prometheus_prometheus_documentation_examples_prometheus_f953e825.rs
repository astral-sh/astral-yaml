#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("prometheus-prometheus-documentation-examples-prometheus-f953e825"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "global" [plain @2:1..2:7]: mapping [block]
        "scrape_interval" [plain @3:3..3:18]: "15s" [plain @3:20..3:23]
        "evaluation_interval" [plain @4:3..4:22]: "15s" [plain @4:24..4:27]
      "alerting" [plain @8:1..8:9]: mapping [block]
        "alertmanagers" [plain @9:3..9:16]: sequence [block]
          - mapping [block]
            "static_configs" [plain @10:7..10:21]: sequence [block]
              - mapping [block]
                "targets" [plain @11:11..11:18]: "" [plain @11:19..11:19]
      "rule_files" [plain @15:1..15:11]: "" [plain @15:12..15:12]
      "scrape_configs" [plain @21:1..21:15]: sequence [block]
        - mapping [block]
          "job_name" [plain @23:5..23:13]: "prometheus" [double-quoted @23:15..23:27]
          "static_configs" [plain @28:5..28:19]: sequence [block]
            - mapping [block]
              "targets" [plain @29:9..29:16]: sequence [flow]
                - "localhost:9090" [double-quoted @29:19..29:35]
              "labels" [plain @31:9..31:15]: mapping [block]
                "app" [plain @32:11..32:14]: "prometheus" [double-quoted @32:16..32:28]
          "scrape_native_histograms" [plain @33:5..33:29]: "true" [plain @33:31..33:35]
    "#);
}
