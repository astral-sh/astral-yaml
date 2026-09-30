#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("spack-spack-share-spack-docs-docker-module-file-tutorial-packages-633d5a4b"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "packages" [plain @1:1..1:9]: mapping [block]
        "git" [plain @2:3..2:6]: mapping [block]
          "buildable" [plain @3:5..3:14]: "False" [plain @3:16..3:21]
          "paths" [plain @4:5..4:10]: mapping [block]
            "git@2.9.4" [plain @5:7..5:16]: "/usr" [plain @5:18..5:22]
        "openmpi" [plain @6:3..6:10]: mapping [block]
          "buildable" [plain @7:5..7:14]: "False" [plain @7:16..7:21]
          "paths" [plain @8:5..8:10]: mapping [block]
            "openmpi@1.10.2" [plain @9:7..9:21]: "/usr" [plain @9:23..9:27]
    "#);
}
