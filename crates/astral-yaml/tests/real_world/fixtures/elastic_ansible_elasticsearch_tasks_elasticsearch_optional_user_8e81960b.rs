#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("elastic-ansible-elasticsearch-tasks-elasticsearch-optional-user-8e81960b"), @r#"
    document 1 (start: ---, end: implicit)
    sequence [block]
      - mapping [block]
        "name" [plain @3:3..3:7]: "Ensure optional elasticsearch group is created with the correct id." [plain @3:9..3:76]
        "become" [plain @4:3..4:9]: "yes" [plain @4:11..4:14]
        "notify" [plain @6:3..6:9]: "restart elasticsearch" [plain @6:11..6:32]
        "group" [plain @7:3..7:8]: mapping [block]
          "state" [plain @8:5..8:10]: "present" [plain @8:12..8:19]
          "name" [plain @9:5..9:9]: "{{ es_group }}" [double-quoted @9:11..9:27]
          "system" [plain @10:5..10:11]: "yes" [plain @10:13..10:16]
          "gid" [plain @11:5..11:8]: "{{ es_group_id }}" [double-quoted @11:10..11:29]
      - mapping [block]
        "name" [plain @13:3..13:7]: "Ensure optional elasticsearch user is created with the correct id." [plain @13:9..13:75]
        "become" [plain @14:3..14:9]: "yes" [plain @14:11..14:14]
        "notify" [plain @16:3..16:9]: "restart elasticsearch" [plain @16:11..16:32]
        "user" [plain @17:3..17:7]: mapping [block]
          "state" [plain @18:5..18:10]: "present" [plain @18:12..18:19]
          "name" [plain @19:5..19:9]: "{{ es_user }}" [double-quoted @19:11..19:26]
          "comment" [plain @20:5..20:12]: "elasticsearch system user" [plain @20:14..20:39]
          "system" [plain @21:5..21:11]: "yes" [plain @21:13..21:16]
          "createhome" [plain @22:5..22:15]: "no" [plain @22:17..22:19]
          "uid" [plain @23:5..23:8]: "{{ es_user_id }}" [double-quoted @23:10..23:28]
          "group" [plain @24:5..24:10]: "{{ es_group }}" [double-quoted @24:12..24:28]
    "#);
}
