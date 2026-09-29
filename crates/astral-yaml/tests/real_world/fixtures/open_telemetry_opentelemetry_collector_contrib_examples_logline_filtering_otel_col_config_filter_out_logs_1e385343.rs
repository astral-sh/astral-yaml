#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("open-telemetry-opentelemetry-collector-contrib-examples-logline-filtering-otel-col-config-filter-out-logs-1e385343"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "receivers" [plain @1:1..1:10]: mapping [block]
        "file_log" [plain @2:3..2:11]: mapping [block]
          "include" [plain @3:5..3:12]: sequence [flow]
            - "/var/log/busybox/simple.log" [plain @3:15..3:42]
          "operators" [plain @4:5..4:14]: sequence [block]
            - mapping [block]
              "type" [plain @5:9..5:13]: "filter" [plain @5:15..5:21]
              "expr" [plain @6:9..6:13]: "body matches \"^INFO:\"" [single-quoted @6:15..6:38]
      "service" [plain @8:1..8:8]: mapping [block]
        "pipelines" [plain @9:3..9:12]: mapping [block]
          "logs" [plain @10:5..10:9]: mapping [block]
            "receivers" [plain @11:7..11:16]: sequence [flow]
              - "file_log" [plain @11:19..11:27]
            "exporters" [plain @12:7..12:16]: sequence [flow]
              - "otlp_grpc/custom" [plain @12:19..12:35]
            "processors" [plain @13:7..13:17]: sequence [flow]
      "exporters" [plain @15:1..15:10]: mapping [block]
        "otlp_grpc/custom" [plain @16:3..16:19]: mapping [block]
          "endpoint" [plain @17:5..17:13]: "http://0.0.0.0:4242" [plain @17:15..17:34]
    "#);
}
