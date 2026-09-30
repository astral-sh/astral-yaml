#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("open-telemetry-opentelemetry-collector-contrib-examples-secure-tracing-otel-collector-config-31a6a38b"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "receivers" [plain @1:1..1:10]: mapping [block]
        "otlp" [plain @2:3..2:7]: mapping [block]
          "protocols" [plain @3:5..3:14]: mapping [block]
            "grpc" [plain @4:7..4:11]: mapping [block]
              "endpoint" [plain @5:9..5:17]: "0.0.0.0:55690" [plain @5:19..5:32]
              "tls" [plain @6:9..6:12]: mapping [block]
                "cert_file" [plain @7:11..7:20]: "/etc/otel-collector.crt" [plain @7:22..7:45]
                "key_file" [plain @8:11..8:19]: "/etc/otel-collector.key" [plain @8:21..8:44]
      "exporters" [plain @10:1..10:10]: mapping [block]
        "debug" [plain @11:3..11:8]: mapping [block]
          "verbosity" [plain @12:5..12:14]: "detailed" [plain @12:16..12:24]
      "service" [plain @14:1..14:8]: mapping [block]
        "pipelines" [plain @15:3..15:12]: mapping [block]
          "traces" [plain @16:5..16:11]: mapping [block]
            "receivers" [plain @17:7..17:16]: sequence [flow]
              - "otlp" [plain @17:19..17:23]
            "exporters" [plain @18:7..18:16]: sequence [flow]
              - "debug" [plain @18:19..18:24]
    "#);
}
