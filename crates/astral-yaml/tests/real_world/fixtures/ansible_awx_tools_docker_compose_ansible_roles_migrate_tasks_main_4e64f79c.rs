#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("ansible-awx-tools-docker-compose-ansible-roles-migrate-tasks-main-4e64f79c"), @r#"
    document 1 (start: ---, end: implicit)
    sequence [block]
      - mapping [block]
        "import_tasks" [plain @4:3..4:15]: "migrate-from-local-docker.yml" [plain @4:17..4:46]
        "when" [plain @5:3..5:7]: "migrate_local_docker" [plain @5:9..5:29]
    "#);
}
