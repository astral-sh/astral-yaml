#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("ansible-awx-tools-docker-compose-ansible-unseal-vault-92fad8fb"), @r#"
    document 1 (start: ---, end: implicit)
    sequence [block]
      - mapping [block]
        "name" [plain @2:3..2:7]: "Run tasks post startup" [plain @2:9..2:31]
        "hosts" [plain @3:3..3:8]: "localhost" [plain @3:10..3:19]
        "gather_facts" [plain @4:3..4:15]: "False" [plain @4:17..4:22]
        "tasks" [plain @5:3..5:8]: sequence [block]
          - mapping [block]
            "name" [plain @6:7..6:11]: "Unseal the vault" [plain @6:13..6:29]
            "ansible.builtin.include_role" [plain @7:7..7:35]: mapping [block]
              "name" [plain @8:9..8:13]: "vault" [plain @8:15..8:20]
              "tasks_from" [plain @9:9..9:19]: "unseal" [plain @9:21..9:27]
          - mapping [block]
            "name" [plain @11:7..11:11]: "Display root token" [plain @11:13..11:31]
            "ansible.builtin.debug" [plain @12:7..12:28]: mapping [block]
              "var" [plain @13:9..13:12]: "Initial_Root_Token" [plain @13:14..13:32]
    "#);
}
