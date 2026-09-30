#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("buildkite-agent-internal-e2e-fixtures-job-update-timeout-70b89976"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "agents" [plain @1:1..1:7]: mapping [block]
        "queue" [plain @2:3..2:8]: "{{.queue}}" [double-quoted @2:10..2:22]
      "steps" [plain @3:1..3:6]: sequence [block]
        - mapping [block]
          "command" [plain @4:5..4:12]: literal [@4:14..7:1]
            "{{.buildkite_agent_binary}} job update timeout 1\n"
            "sleep 300\n"
          "timeout_in_minutes" [plain @7:5..7:23]: "10" [plain @7:25..7:27]
    "#);
}
