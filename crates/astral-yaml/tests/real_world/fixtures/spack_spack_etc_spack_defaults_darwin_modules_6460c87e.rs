#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("spack-spack-etc-spack-defaults-darwin-modules-6460c87e"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "modules" [plain @16:1..16:8]: mapping [block]
        "prefix_inspections" [plain @17:3..17:21]: mapping [block]
          "./lib" [plain @18:5..18:10]: sequence [block]
            - "DYLD_FALLBACK_LIBRARY_PATH" [plain @19:9..19:35]
          "./lib64" [plain @20:5..20:12]: sequence [block]
            - "DYLD_FALLBACK_LIBRARY_PATH" [plain @21:9..21:35]
    "#);
}
