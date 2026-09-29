#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("elastic-ansible-elasticsearch-tasks-xpack-elasticsearch-xpack-05909c15"), @r#"
    document 1 (start: ---, end: implicit)
    sequence [block]
      - mapping [block]
        "name" [plain @4:3..4:7]: "include security/elasticsearch-security.yml" [plain @4:9..4:52]
        "include" [plain @5:3..5:10]: "security/elasticsearch-security.yml" [plain @5:12..5:47]
        "when" [plain @6:3..6:7]: "not oss_version" [plain @6:9..6:24]
      - mapping [block]
        "name" [plain @9:3..9:7]: "Set elasticsearch.keystore Permissions" [plain @9:9..9:47]
        "become" [plain @10:3..10:9]: "yes" [plain @10:11..10:14]
        "file" [plain @11:3..11:7]: mapping [block]
          "state" [plain @12:5..12:10]: "file" [plain @12:12..12:16]
          "path" [plain @13:5..13:9]: "{{ es_conf_dir }}/elasticsearch.keystore" [double-quoted @13:11..13:53]
          "owner" [plain @14:5..14:10]: "root" [plain @14:12..14:16]
          "group" [plain @15:5..15:10]: "{{ es_group }}" [double-quoted @15:12..15:28]
          "mode" [plain @16:5..16:9]: "0660" [double-quoted @16:11..16:17]
    "#);
}
