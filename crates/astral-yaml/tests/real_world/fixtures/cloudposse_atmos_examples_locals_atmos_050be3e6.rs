#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("cloudposse-atmos-examples-locals-atmos-050be3e6"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "base_path" [plain @3:1..3:10]: "." [double-quoted @3:12..3:15]
      "components" [plain @5:1..5:11]: mapping [block]
        "terraform" [plain @6:3..6:12]: mapping [block]
          "base_path" [plain @7:5..7:14]: "components/terraform" [double-quoted @7:16..7:38]
          "apply_auto_approve" [plain @8:5..8:23]: "false" [plain @8:25..8:30]
          "deploy_run_init" [plain @9:5..9:20]: "true" [plain @9:22..9:26]
          "init_run_reconfigure" [plain @10:5..10:25]: "true" [plain @10:27..10:31]
          "auto_generate_backend_file" [plain @11:5..11:31]: "false" [plain @11:33..11:38]
      "stacks" [plain @13:1..13:7]: mapping [block]
        "base_path" [plain @14:3..14:12]: "stacks" [double-quoted @14:14..14:22]
        "included_paths" [plain @15:3..15:17]: sequence [block]
          - "deploy/**/*" [double-quoted @16:7..16:20]
        "name_template" [plain @17:3..17:16]: "{{ .vars.stage }}" [double-quoted @17:18..17:37]
      "templates" [plain @19:1..19:10]: mapping [block]
        "settings" [plain @20:3..20:11]: mapping [block]
          "enabled" [plain @21:5..21:12]: "true" [plain @21:14..21:18]
          "sprig" [plain @22:5..22:10]: mapping [block]
            "enabled" [plain @23:7..23:14]: "true" [plain @23:16..23:20]
          "gomplate" [plain @24:5..24:13]: mapping [block]
            "enabled" [plain @25:7..25:14]: "true" [plain @25:16..25:20]
    "#);
}
