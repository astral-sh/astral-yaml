#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("open-telemetry-opentelemetry-collector-contrib-cmd-opampsupervisor-examples-supervisor-docker-1977519a"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "server" [plain @1:1..1:7]: mapping [block]
        "endpoint" [plain @2:3..2:11]: "wss://127.0.0.1:4320/v1/opamp" [plain @2:13..2:42]
        "tls" [plain @3:3..3:6]: mapping [block]
          "insecure_skip_verify" [plain @6:5..6:25]: "true" [plain @6:27..6:31]
      "capabilities" [plain @9:1..9:13]: mapping [block]
        "reports_effective_config" [plain @10:3..10:27]: "true" [plain @10:29..10:33]
        "reports_own_metrics" [plain @11:3..11:22]: "true" [plain @11:24..11:28]
        "reports_own_logs" [plain @12:3..12:19]: "true" [plain @12:21..12:25]
        "reports_own_traces" [plain @13:3..13:21]: "true" [plain @13:23..13:27]
        "reports_health" [plain @14:3..14:17]: "true" [plain @14:19..14:23]
        "accepts_remote_config" [plain @15:3..15:24]: "true" [plain @15:26..15:30]
      "agent" [plain @17:1..17:6]: mapping [block]
        "executable" [plain @18:3..18:13]: "/otelcontribcol" [plain @18:15..18:30]
      "storage" [plain @20:1..20:8]: mapping [block]
        "directory" [plain @21:3..21:12]: "/etc/otel/supervisor-data/" [plain @21:14..21:40]
    "#);
}
