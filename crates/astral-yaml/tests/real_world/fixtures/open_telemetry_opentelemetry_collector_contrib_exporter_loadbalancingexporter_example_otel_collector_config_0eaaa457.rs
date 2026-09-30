#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("open-telemetry-opentelemetry-collector-contrib-exporter-loadbalancingexporter-example-otel-collector-config-0eaaa457"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "receivers" [plain @1:1..1:10]: mapping [block]
        "otlp" [plain @2:3..2:7]: mapping [block]
          "protocols" [plain @3:5..3:14]: mapping [block]
            "grpc" [plain @4:7..4:11]: "" [plain @4:12..4:12]
      "exporters" [plain @6:1..6:10]: mapping [block]
        "debug" [plain @7:3..7:8]: mapping [block]
          "verbosity" [plain @8:5..8:14]: "detailed" [plain @8:16..8:24]
      "extensions" [plain @10:1..10:11]: mapping [block]
        "health_check" [plain @11:3..11:15]: "" [plain @11:16..11:16]
        "pprof" [plain @12:3..12:8]: "" [plain @12:9..12:9]
        "zpages" [plain @13:3..13:9]: "" [plain @13:10..13:10]
      "service" [plain @15:1..15:8]: mapping [block]
        "extensions" [plain @16:3..16:13]: sequence [flow]
          - "pprof" [plain @16:16..16:21]
          - "zpages" [plain @16:23..16:29]
          - "health_check" [plain @16:31..16:43]
        "pipelines" [plain @17:3..17:12]: mapping [block]
          "logs" [plain @18:5..18:9]: mapping [block]
            "receivers" [plain @19:7..19:16]: sequence [flow]
              - "otlp" [plain @19:19..19:23]
            "exporters" [plain @20:7..20:16]: sequence [flow]
              - "debug" [plain @20:19..20:24]
    "#);
}
