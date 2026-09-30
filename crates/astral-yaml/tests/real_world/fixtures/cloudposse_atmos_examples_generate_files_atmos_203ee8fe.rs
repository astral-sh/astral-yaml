#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("cloudposse-atmos-examples-generate-files-atmos-203ee8fe"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "base_path" [plain @2:1..2:10]: "./" [double-quoted @2:12..2:16]
      "components" [plain @4:1..4:11]: mapping [block]
        "terraform" [plain @5:3..5:12]: mapping [block]
          "base_path" [plain @6:5..6:14]: "components/terraform" [double-quoted @6:16..6:38]
          "apply_auto_approve" [plain @7:5..7:23]: "false" [plain @7:25..7:30]
          "deploy_run_init" [plain @8:5..8:20]: "true" [plain @8:22..8:26]
          "auto_generate_files" [plain @9:5..9:24]: "true" [plain @9:26..9:30]
      "stacks" [plain @11:1..11:7]: mapping [block]
        "base_path" [plain @12:3..12:12]: "stacks" [double-quoted @12:14..12:22]
        "included_paths" [plain @13:3..13:17]: sequence [block]
          - "deploy/**/*" [double-quoted @14:7..14:20]
        "excluded_paths" [plain @15:3..15:17]: sequence [block]
          - "**/_defaults.yaml" [double-quoted @16:7..16:26]
        "name_template" [plain @17:3..17:16]: "{{ .vars.stage }}" [double-quoted @17:18..17:37]
      "logs" [plain @19:1..19:5]: mapping [block]
        "level" [plain @20:3..20:8]: "Info" [plain @20:10..20:14]
        "file" [plain @21:3..21:7]: "/dev/stderr" [double-quoted @21:9..21:22]
    "#);
}
