#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("ansible-awx-tools-docker-compose-ansible-plumb-vault-5b0cf4c4"), @r#"
    document 1 (start: ---, end: implicit)
    sequence [block]
      - mapping [block]
        "name" [plain @2:3..2:7]: "Plumb AWX for Vault" [plain @2:9..2:28]
        "hosts" [plain @3:3..3:8]: "localhost" [plain @3:10..3:19]
        "gather_facts" [plain @4:3..4:15]: "False" [plain @4:17..4:22]
        "vars" [plain @5:3..5:7]: mapping [block]
          "awx_host" [plain @6:5..6:13]: "https://127.0.0.1:8043" [double-quoted @6:15..6:39]
        "tasks" [plain @7:3..7:8]: sequence [block]
          - mapping [block]
            "include_role" [plain @8:7..8:19]: mapping [block]
              "name" [plain @9:9..9:13]: "vault" [plain @9:15..9:20]
              "tasks_from" [plain @10:9..10:19]: "plumb" [plain @10:21..10:26]
    "#);
}
