#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-docs-snippets-docs-snippets-guides-log-debug-logging-config-prod-job-1f27d559"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "loggers" [plain @1:1..1:8]: mapping [block]
        "cloudwatch" [plain @2:3..2:13]: mapping [block]
          "config" [plain @3:5..3:11]: mapping [block]
            "log_level" [plain @4:7..4:16]: "ERROR" [plain @4:18..4:23]
            "log_group_name" [plain @5:7..5:21]: "/my/cool/cloudwatch/log/group" [plain @5:23..5:52]
            "log_stream_name" [plain @6:7..6:22]: "very_good_log_stream" [plain @6:24..6:44]
    "#);
}
