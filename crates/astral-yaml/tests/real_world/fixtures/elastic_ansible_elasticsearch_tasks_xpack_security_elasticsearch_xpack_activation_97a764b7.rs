#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("elastic-ansible-elasticsearch-tasks-xpack-security-elasticsearch-xpack-activation-97a764b7"), @r#"
    document 1 (start: ---, end: implicit)
    sequence [block]
      - mapping [block]
        "name" [plain @2:3..2:7]: "Activate ES license (with security authentication)" [plain @2:9..2:59]
        "uri" [plain @3:3..3:6]: mapping [block]
          "method" [plain @4:5..4:11]: "PUT" [plain @4:13..4:16]
          "url" [plain @5:5..5:8]: "{{ es_api_uri }}/{{ es_license_api }}?acknowledge=true" [double-quoted @5:10..5:66]
          "user" [plain @6:5..6:9]: "{{es_api_basic_auth_username  | default(omit)}}" [double-quoted @6:11..6:60]
          "password" [plain @7:5..7:13]: "{{es_api_basic_auth_password | default(omit)}}" [double-quoted @7:15..7:63]
          "body_format" [plain @8:5..8:16]: "json" [plain @8:18..8:22]
          "body" [plain @9:5..9:9]: "{{ es_xpack_license }}" [double-quoted @9:11..9:35]
          "return_content" [plain @10:5..10:19]: "yes" [plain @10:21..10:24]
          "force_basic_auth" [plain @11:5..11:21]: "yes" [plain @11:23..11:26]
          "validate_certs" [plain @12:5..12:19]: "{{ es_validate_certs }}" [double-quoted @12:21..12:46]
        "register" [plain @13:3..13:11]: "license_activated" [plain @13:13..13:30]
        "no_log" [plain @14:3..14:9]: "True" [plain @14:11..14:15]
        "failed_when" [plain @15:3..15:14]: folded [@15:16..20:1]
          "license_activated.status != 200 or license_activated.json.license_status is not defined or license_activated.json.license_status != 'valid'\n"
      - mapping [block]
        "name" [plain @20:3..20:7]: "License" [plain @20:9..20:16]
        "debug" [plain @21:3..21:8]: "msg={{ license_activated }}" [plain @21:10..21:37]
    "#);
}
