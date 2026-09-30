#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("spack-spack-etc-spack-include-3619e12f"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "include" [plain @1:1..1:8]: sequence [block]
        - mapping [block]
          "name" [plain @3:5..3:9]: "user" [double-quoted @3:11..3:17]
          "path_override_env_var" [plain @4:5..4:26]: "SPACK_USER_CONFIG_PATH" [plain @4:28..4:50]
          "path" [plain @5:5..5:9]: "~/.spack" [double-quoted @5:11..5:21]
          "optional" [plain @6:5..6:13]: "true" [plain @6:15..6:19]
          "prefer_modify" [plain @7:5..7:18]: "true" [plain @7:20..7:24]
          "when" [plain @8:5..8:9]: "\"SPACK_DISABLE_LOCAL_CONFIG\" not in env" [single-quoted @8:11..8:52]
        - mapping [block]
          "name" [plain @11:5..11:9]: "site" [double-quoted @11:11..11:17]
          "path" [plain @12:5..12:9]: "$spack/etc/spack/site" [double-quoted @12:11..12:34]
          "optional" [plain @13:5..13:13]: "true" [plain @13:15..13:19]
        - mapping [block]
          "name" [plain @16:5..16:9]: "system" [double-quoted @16:11..16:19]
          "path_override_env_var" [plain @17:5..17:26]: "SPACK_SYSTEM_CONFIG_PATH" [plain @17:28..17:52]
          "path" [plain @18:5..18:9]: "/etc/spack" [double-quoted @18:11..18:23]
          "optional" [plain @19:5..19:13]: "true" [plain @19:15..19:19]
          "when" [plain @20:5..20:9]: "\"SPACK_DISABLE_LOCAL_CONFIG\" not in env" [single-quoted @20:11..20:52]
    "#);
}
