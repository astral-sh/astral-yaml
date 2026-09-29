#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("open-telemetry-opentelemetry-collector-contrib-examples-couchbase-prometheus-config-74c5a47c"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "scrape_configs" [plain @3:1..3:15]: sequence [block]
        - mapping [block]
          "job_name" [plain @5:5..5:13]: "otel-collector" [double-quoted @5:15..5:31]
          "scrape_interval" [plain @8:5..8:20]: "5s" [plain @8:22..8:24]
          "static_configs" [plain @9:5..9:19]: sequence [block]
            - mapping [block]
              "targets" [plain @10:9..10:16]: sequence [flow]
                - "opentelemetry-collector-contrib:9123" [double-quoted @10:19..10:57]
    "#);
}
