#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("ansible-awx-awx-collection-tools-generate-48f48ea1"), @r#"
    document 1 (start: ---, end: implicit)
    sequence [block]
      - mapping [block]
        "name" [plain @2:3..2:7]: "Generate the awx.awx collection" [plain @2:9..2:40]
        "hosts" [plain @3:3..3:8]: "localhost" [plain @3:10..3:19]
        "connection" [plain @4:3..4:13]: "local" [plain @4:15..4:20]
        "gather_facts" [plain @5:3..5:15]: "false" [plain @5:17..5:22]
        "vars" [plain @6:3..6:7]: mapping [block]
          "api_url" [plain @7:5..7:12]: "{{ lookup('env', 'TOWER_HOST') }}" [double-quoted @7:14..7:49]
        "vars_files" [plain @8:3..8:13]: sequence [block]
          - "vars/generate_for.yml" [plain @9:7..9:28]
          - "vars/aliases.yml" [plain @10:7..10:23]
          - "vars/associations.yml" [plain @11:7..11:28]
          - "vars/resolution.yml" [plain @12:7..12:26]
          - "vars/examples.yml" [plain @13:7..13:24]
        "module_defaults" [plain @14:3..14:18]: mapping [block]
          "uri" [plain @15:5..15:8]: mapping [block]
            "validate_certs" [plain @16:7..16:21]: "false" [plain @16:23..16:28]
            "force_basic_auth" [plain @17:7..17:23]: "true" [plain @17:25..17:29]
            "url_username" [plain @18:7..18:19]: "{{ lookup('env', 'TOWER_USERNAME') }}" [double-quoted @18:21..18:60]
            "url_password" [plain @19:7..19:19]: "{{ lookup('env', 'TOWER_PASSWORD') }}" [double-quoted @19:21..19:60]
        "roles" [plain @20:3..20:8]: sequence [block]
          - "generate" [plain @21:7..21:15]
    "#);
}
