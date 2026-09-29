#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("ansible-awx-tools-docker-compose-ansible-roles-migrate-defaults-main-68de52d9"), @r#"
    document 1 (start: ---, end: implicit)
    mapping [block]
      "postgres_data_dir" [plain @2:1..2:18]: "~/.awx/pgdocker" [double-quoted @2:20..2:37]
      "migrate_local_docker" [plain @3:1..3:21]: "false" [plain @3:23..3:28]
      "old_docker_compose_dir" [plain @4:1..4:23]: "~/.awx/awxcompose" [double-quoted @4:25..4:44]
      "pg_volume_name" [plain @5:1..5:15]: "postgres_data" [double-quoted @5:17..5:32]
    "#);
}
