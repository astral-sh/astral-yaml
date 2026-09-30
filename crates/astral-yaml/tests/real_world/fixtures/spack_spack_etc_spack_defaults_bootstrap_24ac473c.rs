#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("spack-spack-etc-spack-defaults-bootstrap-24ac473c"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "bootstrap" [plain @1:1..1:10]: mapping [block]
        "enable" [plain @4:3..4:9]: "true" [plain @4:11..4:15]
        "root" [plain @7:3..7:7]: "$user_cache_path/bootstrap" [plain @7:9..7:35]
        "sources" [plain @11:3..11:10]: sequence [block]
          - mapping [block]
            "name" [plain @12:5..12:9]: "github-actions-v2" [plain @12:11..12:28]
            "metadata" [plain @13:5..13:13]: "$spack/share/spack/bootstrap/github-actions-v2" [plain @13:15..13:61]
          - mapping [block]
            "name" [plain @14:5..14:9]: "github-actions-v0.6" [plain @14:11..14:30]
            "metadata" [plain @15:5..15:13]: "$spack/share/spack/bootstrap/github-actions-v0.6" [plain @15:15..15:63]
          - mapping [block]
            "name" [plain @16:5..16:9]: "spack-install" [plain @16:11..16:24]
            "metadata" [plain @17:5..17:13]: "$spack/share/spack/bootstrap/spack-install" [plain @17:15..17:57]
        "trusted" [plain @18:3..18:10]: mapping [block]
          "github-actions-v2" [plain @21:5..21:22]: "true" [plain @21:24..21:28]
          "github-actions-v0.6" [plain @22:5..22:24]: "true" [plain @22:26..22:30]
          "spack-install" [plain @23:5..23:18]: "true" [plain @23:20..23:24]
    "#);
}
