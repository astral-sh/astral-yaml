#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("buildkite-agent-buildkite-docker-compose-release-a23f2131"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "version" [plain @1:1..1:8]: "2" [single-quoted @1:10..1:13]
      "services" [plain @3:1..3:9]: mapping [block]
        "github-release" [plain @5:3..5:17]: mapping [block]
          "build" [plain @6:5..6:10]: mapping [block]
            "context" [plain @7:7..7:14]: "." [plain @7:16..7:17]
            "dockerfile" [plain @8:7..8:17]: "Dockerfile-github-release" [plain @8:19..8:44]
          "volumes" [plain @9:5..9:12]: sequence [block]
            - "../:/app" [plain @10:9..10:17]
            - "/usr/bin/buildkite-agent:/usr/bin/buildkite-agent" [plain @11:9..11:58]
          "working_dir" [plain @12:5..12:16]: "/app" [plain @12:18..12:22]
          "environment" [plain @13:5..13:16]: sequence [block]
            - "BUILDKITE_AGENT_ACCESS_TOKEN" [plain @14:9..14:37]
            - "BUILDKITE_JOB_ID" [plain @15:9..15:25]
            - "BUILDKITE_BUILD_ID" [plain @16:9..16:27]
            - "BUILDKITE_BUILD_NUMBER" [plain @17:9..17:31]
    "#);
}
