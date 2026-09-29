#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("spack-spack-etc-spack-defaults-windows-packages-1f46f2a8"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "packages" [plain @16:1..16:9]: mapping [block]
        "all" [plain @17:3..17:6]: mapping [block]
          "providers" [plain @18:5..18:14]: mapping [block]
            "c" [plain @19:7..19:8]: sequence [flow]
              - "msvc" [plain @19:12..19:16]
            "cxx" [plain @20:7..20:10]: sequence [flow]
              - "msvc" [plain @20:13..20:17]
            "mpi" [plain @21:7..21:10]: sequence [flow]
              - "msmpi" [plain @21:13..21:18]
            "gl" [plain @22:7..22:9]: sequence [flow]
              - "wgl" [plain @22:12..22:15]
        "mpi" [plain @23:3..23:6]: mapping [block]
          "require" [plain @24:5..24:12]: sequence [block]
            - mapping [block]
              "one_of" [plain @25:7..25:13]: sequence [flow]
                - "msmpi" [plain @25:16..25:21]
        "msvc" [plain @26:3..26:7]: mapping [block]
          "buildable" [plain @27:5..27:14]: "false" [plain @27:16..27:21]
    "#);
}
