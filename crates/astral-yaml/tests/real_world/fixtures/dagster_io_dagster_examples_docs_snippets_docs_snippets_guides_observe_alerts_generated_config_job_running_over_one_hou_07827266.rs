#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-docs-snippets-docs-snippets-guides-observe-alerts-generated-config-job-running-over-one-hou-07827266"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "alert_policies" [plain @3:1..3:15]: sequence [block]
        - mapping [block]
          "alert_targets" [plain @4:3..4:16]: sequence [block]
            - mapping [block]
              "long_running_job_threshold_target" [plain @5:5..5:38]: mapping [block]
                "threshold_seconds" [plain @6:7..6:24]: "3600" [plain @6:26..6:30]
          "description" [plain @7:3..7:14]: "Sends a Slack message when a run is taking too long to complete." [plain @7:16..7:80]
          "event_types" [plain @8:3..8:14]: sequence [block]
            - "JOB_LONG_RUNNING" [plain @9:5..9:21]
          "name" [plain @10:3..10:7]: "job-running-over-one-hour-slack" [plain @10:9..10:40]
          "notification_service" [plain @11:3..11:23]: mapping [block]
            "slack" [plain @12:5..12:10]: mapping [block]
              "slack_channel_name" [plain @13:7..13:25]: "notifications" [plain @13:27..13:40]
              "slack_workspace_name" [plain @14:7..14:27]: "hooli" [plain @14:29..14:34]
          "tags" [plain @15:3..15:7]: mapping [block]
            "important" [plain @16:5..16:14]: "true" [single-quoted @16:16..16:22]
    "#);
}
