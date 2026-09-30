#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("ansible-awx-tools-docker-compose-ansible-roles-vault-tasks-unseal-612e1f58"), @r#"
    document 1 (start: ---, end: implicit)
    sequence [block]
      - mapping [block]
        "name" [plain @2:3..2:7]: "Set vault_addr" [plain @2:9..2:23]
        "include_tasks" [plain @3:3..3:16]: "set_vault_addr.yml" [plain @3:18..3:36]
      - mapping [block]
        "name" [plain @5:3..5:7]: "Load vault keys" [plain @5:9..5:24]
        "include_vars" [plain @6:3..6:15]: mapping [block]
          "file" [plain @7:5..7:9]: "{{ vault_file }}" [double-quoted @7:11..7:29]
      - mapping [block]
        "name" [plain @9:3..9:7]: "Unseal the vault" [plain @9:9..9:25]
        "flowerysong.hvault.seal" [plain @10:3..10:26]: mapping [block]
          "vault_addr" [plain @11:5..11:15]: "{{ vault_addr_from_host }}" [double-quoted @11:17..11:45]
          "validate_certs" [plain @12:5..12:19]: "false" [plain @12:21..12:26]
          "state" [plain @13:5..13:10]: "unsealed" [plain @13:12..13:20]
          "key" [plain @14:5..14:8]: "{{ item }}" [double-quoted @14:10..14:22]
        "loop" [plain @15:3..15:7]: sequence [block]
          - "{{ Unseal_Key_1 }}" [double-quoted @16:7..16:27]
          - "{{ Unseal_Key_2 }}" [double-quoted @17:7..17:27]
          - "{{ Unseal_Key_3 }}" [double-quoted @18:7..18:27]
        "register" [plain @19:3..19:11]: "unseal_result" [plain @19:13..19:26]
        "until" [plain @20:3..20:8]: "unseal_result is succeeded or unseal_result is failed and 'Connection refused' not in unseal_result.msg" [plain @20:10..20:113]
        "retries" [plain @21:3..21:10]: "5" [plain @21:12..21:13]
        "delay" [plain @22:3..22:8]: "1" [plain @22:10..22:11]
    "#);
}
