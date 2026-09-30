#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("ansible-awx-tools-docker-compose-ansible-initialize-containers-6177b074"), @r#"
    document 1 (start: ---, end: implicit)
    sequence [block]
      - mapping [block]
        "name" [plain @2:3..2:7]: "Run any pre-hooks for other container" [plain @2:9..2:46]
        "hosts" [plain @3:3..3:8]: "localhost" [plain @3:10..3:19]
        "gather_facts" [plain @4:3..4:15]: "false" [plain @4:17..4:22]
        "tasks" [plain @5:3..5:8]: sequence [block]
          - mapping [block]
            "name" [plain @6:7..6:11]: "Initialize vault" [plain @6:13..6:29]
            "include_role" [plain @7:7..7:19]: mapping [block]
              "name" [plain @8:9..8:13]: "vault" [plain @8:15..8:20]
              "tasks_from" [plain @9:9..9:19]: "initialize" [plain @9:21..9:31]
            "when" [plain @10:7..10:11]: "enable_vault | bool" [plain @10:13..10:32]
    "#);
}
