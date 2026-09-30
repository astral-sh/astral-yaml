#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("ansible-awx-tools-ansible-roles-dockerfile-tasks-main-78bb087c"), @r#"
    document 1 (start: ---, end: implicit)
    sequence [block]
      - mapping [block]
        "name" [plain @2:3..2:7]: "Create _build directory" [plain @2:9..2:32]
        "file" [plain @3:3..3:7]: mapping [block]
          "path" [plain @4:5..4:9]: "{{ dockerfile_dest }}/{{ template_dest }}" [double-quoted @4:11..4:54]
          "state" [plain @5:5..5:10]: "directory" [plain @5:12..5:21]
      - mapping [block]
        "name" [plain @7:3..7:7]: "Render supervisor configs" [plain @7:9..7:34]
        "template" [plain @8:3..8:11]: mapping [block]
          "src" [plain @9:5..9:8]: "{{ item }}.j2" [double-quoted @9:10..9:25]
          "dest" [plain @10:5..10:9]: "{{ dockerfile_dest }}/{{ template_dest }}/{{ item }}" [double-quoted @10:11..10:65]
        "with_items" [plain @11:3..11:13]: sequence [block]
          - "supervisor_web.conf" [double-quoted @12:7..12:28]
          - "supervisor_task.conf" [double-quoted @13:7..13:29]
          - "supervisor_rsyslog.conf" [double-quoted @14:7..14:32]
      - mapping [block]
        "name" [plain @16:3..16:7]: "Render Dockerfile" [plain @16:9..16:26]
        "template" [plain @17:3..17:11]: mapping [block]
          "src" [plain @18:5..18:8]: "Dockerfile.j2" [plain @18:10..18:23]
          "dest" [plain @19:5..19:9]: "{{ dockerfile_dest }}/{{ dockerfile_name }}" [double-quoted @19:11..19:56]
    "#);
}
