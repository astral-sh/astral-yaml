#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("buildkite-agent-internal-e2e-fixtures-env-precedence-21f99449"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "agents" [plain @1:1..1:7]: mapping [block]
        "queue" [plain @2:3..2:8]: mapping [flow]
          ? mapping [flow]
            ".queue" [plain @2:13..2:19]: "" [plain @2:19..2:19]
          : "" [plain @2:21..2:21]
      "env" [plain @3:1..3:4]: mapping [block]
        "GLOBAL_ENV" [plain @4:3..4:13]: "llamas" [plain @4:15..4:21]
        "ENV_ORDER" [plain @5:3..5:12]: "global" [plain @5:14..5:20]
      "steps" [plain @6:1..6:6]: sequence [block]
        - mapping [block]
          "label" [plain @7:5..7:10]: "test env" [double-quoted @7:12..7:22]
          "env" [plain @8:5..8:8]: mapping [block]
            "LOCAL_ENV" [plain @9:7..9:16]: "alpacas" [plain @9:18..9:25]
            "ENV_ORDER" [plain @10:7..10:16]: "local" [plain @10:18..10:23]
          "commands" [plain @11:5..11:13]: sequence [block]
            - literal [@12:9..16:1]
              "[[ \"$${GLOBAL_ENV?}\" == \"llamas\" ]]\n"
              "[[ \"$${LOCAL_ENV?}\" == \"alpacas\" ]]\n"
              "[[ \"$${ENV_ORDER?}\" == \"local\" ]]\n"
    "#);
}
