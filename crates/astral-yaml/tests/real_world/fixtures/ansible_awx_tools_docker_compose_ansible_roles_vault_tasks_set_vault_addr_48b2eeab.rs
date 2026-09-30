#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("ansible-awx-tools-docker-compose-ansible-roles-vault-tasks-set-vault-addr-48b2eeab"), @r#"
    document 1 (start: ---, end: implicit)
    sequence [block]
      - mapping [block]
        "name" [plain @2:3..2:7]: "Detect if vault cert directory exist" [plain @2:9..2:45]
        "stat" [plain @3:3..3:7]: mapping [block]
          "path" [plain @4:5..4:9]: "{{ vault_cert_dir }}" [double-quoted @4:11..4:33]
        "register" [plain @5:3..5:11]: "vault_cert_dir_stat" [plain @5:13..5:32]
      - mapping [block]
        "name" [plain @7:3..7:7]: "Set vault_addr for http" [plain @7:9..7:32]
        "set_fact" [plain @8:3..8:11]: mapping [block]
          "vault_addr" [plain @9:5..9:15]: "http://127.0.0.1:1234" [double-quoted @9:17..9:40]
          "vault_addr_from_host" [plain @10:5..10:25]: "http://localhost:1234" [double-quoted @10:27..10:50]
          "vault_addr_from_container" [plain @11:5..11:30]: "http://tools_vault_1:1234" [double-quoted @11:32..11:59]
        "when" [plain @12:3..12:7]: "vault_cert_dir_stat.stat.exists == false" [plain @12:9..12:49]
      - mapping [block]
        "name" [plain @14:3..14:7]: "Set vault_addr for https" [plain @14:9..14:33]
        "set_fact" [plain @15:3..15:11]: mapping [block]
          "vault_addr" [plain @16:5..16:15]: "https://127.0.0.1:1234" [double-quoted @16:17..16:41]
          "vault_addr_from_host" [plain @17:5..17:25]: "https://localhost:1234" [double-quoted @17:27..17:51]
          "vault_addr_from_container" [plain @18:5..18:30]: "https://tools_vault_1:1234" [double-quoted @18:32..18:60]
        "when" [plain @19:3..19:7]: "vault_cert_dir_stat.stat.exists == true" [plain @19:9..19:48]
    "#);
}
