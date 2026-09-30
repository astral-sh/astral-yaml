#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("open-telemetry-opentelemetry-collector-contrib-cmd-opampsupervisor-examples-supervisor-fallback-b0908fb6"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "server" [plain @9:1..9:7]: mapping [block]
        "endpoint" [plain @10:3..10:11]: "wss://opamp.example.com/v1/opamp" [plain @10:13..10:45]
        "tls" [plain @11:3..11:6]: "" [plain @11:7..11:7]
      "capabilities" [plain @15:1..15:13]: mapping [block]
        "reports_effective_config" [plain @16:3..16:27]: "true" [plain @16:29..16:33]
        "reports_own_metrics" [plain @17:3..17:22]: "true" [plain @17:24..17:28]
        "reports_own_logs" [plain @18:3..18:19]: "true" [plain @18:21..18:25]
        "reports_own_traces" [plain @19:3..19:21]: "true" [plain @19:23..19:27]
        "reports_health" [plain @20:3..20:17]: "true" [plain @20:19..20:23]
        "accepts_remote_config" [plain @21:3..21:24]: "true" [plain @21:26..21:30]
        "accepts_restart_command" [plain @22:3..22:26]: "true" [plain @22:28..22:32]
      "agent" [plain @24:1..24:6]: mapping [block]
        "executable" [plain @26:3..26:13]: "/opt/otelcol/bin/otelcol" [plain @26:15..26:39]
        "startup_fallback_configs" [plain @31:3..31:27]: sequence [block]
          - "/etc/otelcol/fallback.yaml" [plain @32:5..32:31]
      "storage" [plain @34:1..34:8]: mapping [block]
        "directory" [plain @35:3..35:12]: "/var/lib/otelcol/supervisor" [plain @35:14..35:41]
    "#);
}
