#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("spack-spack-share-spack-bootstrap-spack-install-metadata-2c9ee462"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @4:1..4:5]: "install" [plain @4:7..4:14]
      "description" [plain @5:1..5:12]: literal [@5:14..7:1]
        "Specs built from sources downloaded from the Spack public mirror.\n"
      "info" [plain @7:1..7:5]: mapping [block]
        "url" [plain @8:3..8:6]: "https://mirror.spack.io" [plain @8:8..8:31]
    "#);
}
