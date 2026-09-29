#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("cloudposse-atmos-examples-demo-env-atmos-141e43d4"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "base_path" [plain @1:1..1:10]: "./" [double-quoted @1:12..1:16]
      "components" [plain @3:1..3:11]: mapping [block]
        "terraform" [plain @4:3..4:12]: mapping [block]
          "base_path" [plain @5:5..5:14]: "components/terraform" [double-quoted @5:16..5:38]
          "apply_auto_approve" [plain @6:5..6:23]: "false" [plain @6:25..6:30]
          "deploy_run_init" [plain @7:5..7:20]: "true" [plain @7:22..7:26]
          "init_run_reconfigure" [plain @8:5..8:25]: "true" [plain @8:27..8:31]
          "auto_generate_backend_file" [plain @9:5..9:31]: "false" [plain @9:33..9:38]
      "stacks" [plain @11:1..11:7]: mapping [block]
        "base_path" [plain @12:3..12:12]: "stacks" [double-quoted @12:14..12:22]
        "included_paths" [plain @13:3..13:17]: sequence [block]
          - "deploy/**/*" [double-quoted @14:7..14:20]
        "excluded_paths" [plain @15:3..15:17]: sequence [block]
          - "**/_defaults.yaml" [double-quoted @16:7..16:26]
        "name_template" [plain @17:3..17:16]: "{{.vars.stage}}" [double-quoted @17:18..17:35]
      "env" [plain @21:1..21:4]: mapping [block]
        "GITHUB_TOKEN" [plain @23:3..23:15]: "gh auth token" [plain @23:23..23:36] !<!exec>
        "DEMO_ENVIRONMENT" [plain @25:3..25:19]: "local-development" [plain @25:21..25:38]
      "logs" [plain @27:1..27:5]: mapping [block]
        "level" [plain @28:3..28:8]: "Info" [plain @28:10..28:14]
        "file" [plain @29:3..29:7]: "/dev/stderr" [double-quoted @29:9..29:22]
    "#);
}
