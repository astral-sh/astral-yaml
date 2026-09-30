#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("ansible-awx-tools-docker-compose-ansible-roles-vault-defaults-main-66cfc87a"), @r#"
    document 1 (start: ---, end: implicit)
    mapping [block]
      "vault_file" [plain @2:1..2:11]: "{{ sources_dest }}/secrets/vault_init.yml" [double-quoted @2:13..2:56]
      "admin_password_file" [plain @3:1..3:20]: "{{ sources_dest }}/secrets/admin_password.yml" [double-quoted @3:22..3:69]
      "vault_cert_dir" [plain @4:1..4:15]: "{{ sources_dest }}/vault_certs" [double-quoted @4:17..4:49]
      "vault_server_cert" [plain @5:1..5:18]: "{{ vault_cert_dir }}/server.crt" [double-quoted @5:20..5:53]
      "vault_client_cert" [plain @6:1..6:18]: "{{ vault_cert_dir }}/client.crt" [double-quoted @6:20..6:53]
      "vault_client_key" [plain @7:1..7:17]: "{{ vault_cert_dir }}/client.key" [double-quoted @7:19..7:52]
      "vault_userpass_username" [plain @8:1..8:24]: "awx_userpass_admin" [double-quoted @8:26..8:46]
      "vault_userpass_password" [plain @9:1..9:24]: "userpass123" [double-quoted @9:26..9:39]
    "#);
}
