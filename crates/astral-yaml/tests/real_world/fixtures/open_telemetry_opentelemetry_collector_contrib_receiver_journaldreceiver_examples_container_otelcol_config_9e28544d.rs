#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("open-telemetry-opentelemetry-collector-contrib-receiver-journaldreceiver-examples-container-otelcol-config-9e28544d"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "receivers" [plain @1:1..1:10]: mapping [block]
        "journald" [plain @2:3..2:11]: mapping [block]
          "directory" [plain @3:5..3:14]: "/opt/journal" [plain @3:16..3:28]
      "exporters" [plain @5:1..5:10]: mapping [block]
        "debug" [plain @6:3..6:8]: mapping [block]
          "verbosity" [plain @7:5..7:14]: "normal" [plain @7:16..7:22]
      "service" [plain @9:1..9:8]: mapping [block]
        "pipelines" [plain @10:3..10:12]: mapping [block]
          "logs" [plain @11:5..11:9]: mapping [block]
            "receivers" [plain @12:7..12:16]: sequence [flow]
              - "journald" [plain @12:19..12:27]
            "exporters" [plain @13:7..13:16]: sequence [flow]
              - "debug" [plain @13:19..13:24]
    "#);
}
