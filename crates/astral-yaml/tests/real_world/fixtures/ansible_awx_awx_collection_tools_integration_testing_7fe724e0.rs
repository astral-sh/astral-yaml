#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("ansible-awx-awx-collection-tools-integration-testing-7fe724e0"), @r#"
    document 1 (start: ---, end: implicit)
    sequence [block]
      - mapping [block]
        "hosts" [plain @2:3..2:8]: "localhost" [plain @2:10..2:19]
        "gather_facts" [plain @3:3..3:15]: "false" [plain @3:17..3:22]
        "connection" [plain @4:3..4:13]: "local" [plain @4:15..4:20]
        "collections" [plain @5:3..5:14]: sequence [block]
          - "awx.awx" [plain @6:7..6:14]
        "vars" [plain @7:3..7:7]: mapping [block]
          "collection_location" [plain @8:5..8:24]: "{{ playbook_dir }}/.." [double-quoted @8:26..8:49]
          "loc_tests" [plain @9:5..9:14]: "{{ collection_location }}/tests/integration/targets/" [double-quoted @9:16..9:70]
          "test" [plain @10:5..10:9]: "ad_hoc_command,host,role" [plain @10:11..10:35]
        "tasks" [plain @11:3..11:8]: sequence [block]
          - mapping [block]
            "name" [plain @12:7..12:11]: "DEBUG - make sure variables are what we expect" [plain @12:13..12:59]
            "ansible.builtin.debug" [plain @13:7..13:28]: mapping [block]
              "msg" [plain @14:9..14:12]: literal [@14:14..20:1]
                "Running tests at location:\n"
                "    {{ loc_tests }}\n"
                "Running tests folders:\n"
                "    {{ test | trim | split(',') }}\n"
          - mapping [block]
            "name" [plain @20:7..20:11]: "Include test targets" [double-quoted @20:13..20:35]
            "ansible.builtin.include_tasks" [plain @21:7..21:36]: "{{ loc_tests }}{{ test_name }}/tasks/main.yml" [double-quoted @21:38..21:85]
            "loop" [plain @22:7..22:11]: "{{ test | trim | split(',') }}" [double-quoted @22:13..22:45]
            "loop_control" [plain @23:7..23:19]: mapping [block]
              "loop_var" [plain @24:9..24:17]: "test_name" [plain @24:19..24:28]
    "#);
}
