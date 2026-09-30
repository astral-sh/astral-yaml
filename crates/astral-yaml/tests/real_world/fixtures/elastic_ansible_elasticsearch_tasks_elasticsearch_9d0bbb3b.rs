#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("elastic-ansible-elasticsearch-tasks-elasticsearch-9d0bbb3b"), @r#"
    document 1 (start: ---, end: implicit)
    sequence [block]
      - mapping [block]
        "name" [plain @3:3..3:7]: "Include specific Elasticsearch" [plain @3:9..3:39]
        "include" [plain @4:3..4:10]: "elasticsearch-Debian.yml" [plain @4:12..4:36]
        "when" [plain @5:3..5:7]: "ansible_os_family == 'Debian'" [plain @5:9..5:38]
      - mapping [block]
        "name" [plain @7:3..7:7]: "Include specific Elasticsearch" [plain @7:9..7:39]
        "include" [plain @8:3..8:10]: "elasticsearch-RedHat.yml" [plain @8:12..8:36]
        "when" [plain @9:3..9:7]: "ansible_os_family == 'RedHat'" [plain @9:9..9:38]
    "#);
}
