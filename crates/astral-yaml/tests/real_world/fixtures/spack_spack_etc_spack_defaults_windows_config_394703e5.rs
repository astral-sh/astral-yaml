#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("spack-spack-etc-spack-defaults-windows-config-394703e5"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "config" [plain @1:1..1:7]: mapping [block]
        "locks" [plain @2:3..2:8]: "false" [plain @2:10..2:15]
        "build_stage:" [plain @3:3..3:15]: sequence [block]
          - "$user_cache_path/stage" [single-quoted @4:7..4:31]
        "stage_name" [plain @5:3..5:13]: "{name}-{version}-{hash:7}" [single-quoted @5:15..5:42]
        "concurrent_packages" [plain @6:3..6:22]: "1" [plain @6:24..6:25]
    "#);
}
