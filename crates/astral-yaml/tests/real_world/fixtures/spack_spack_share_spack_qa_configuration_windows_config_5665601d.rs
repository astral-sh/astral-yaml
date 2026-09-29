#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("spack-spack-share-spack-qa-configuration-windows-config-5665601d"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "config" [plain @1:1..1:7]: mapping [block]
        "locks" [plain @2:3..2:8]: "false" [plain @2:10..2:15]
        "install_tree" [plain @3:3..3:15]: mapping [block]
          "root" [plain @4:5..4:9]: "$spack\\opt\\spack" [plain @4:11..4:27]
          "projections" [plain @5:5..5:16]: mapping [block]
            "all" [plain @6:7..6:10]: "{architecture}\\\\{compiler.name}-{compiler.version}\\\\{name}-{version}-{hash}" [single-quoted @6:12..6:89]
        "build_stage" [plain @7:3..7:14]: sequence [block]
          - "~/.spack/stage" [plain @8:7..8:21]
    "#);
}
