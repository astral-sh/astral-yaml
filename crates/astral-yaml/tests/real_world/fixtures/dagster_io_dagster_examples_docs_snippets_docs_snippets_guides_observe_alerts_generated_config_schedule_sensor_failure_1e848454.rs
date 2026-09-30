#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("dagster-io-dagster-examples-docs-snippets-docs-snippets-guides-observe-alerts-generated-config-schedule-sensor-failure-1e848454"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "alert_policies" [plain @3:1..3:15]: sequence [block]
        - mapping [block]
          "description" [plain @4:3..4:14]: "Sends a Slack message when a schedule or sensor tick fails." [plain @4:16..4:75]
          "event_types" [plain @5:3..5:14]: sequence [block]
            - "TICK_FAILURE" [plain @6:5..6:17]
          "name" [plain @7:3..7:7]: "schedule-sensor-failure-slack" [plain @7:9..7:38]
          "notification_service" [plain @8:3..8:23]: mapping [block]
            "slack" [plain @9:5..9:10]: mapping [block]
              "slack_channel_name" [plain @10:7..10:25]: "notifications" [plain @10:27..10:40]
              "slack_workspace_name" [plain @11:7..11:27]: "hooli" [plain @11:29..11:34]
    "#);
}
