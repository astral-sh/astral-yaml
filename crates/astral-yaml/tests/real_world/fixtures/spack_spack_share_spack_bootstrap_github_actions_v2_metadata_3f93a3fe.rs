#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("spack-spack-share-spack-bootstrap-github-actions-v2-metadata-3f93a3fe"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @1:1..1:5]: "buildcache" [plain @1:7..1:17]
      "description" [plain @2:1..2:12]: literal [@2:14..5:1]
        "Buildcache generated from a public workflow using GitHub Actions hosted on GitHub Packages.\n"
        "The sha256 checksum of binaries is checked before installation.\n"
      "info" [plain @5:1..5:5]: mapping [block]
        "url" [plain @6:3..6:6]: "oci://ghcr.io/spack/bootstrap-buildcache-v2.2" [plain @6:8..6:53]
        "homepage" [plain @7:3..7:11]: "https://github.com/spack/spack-bootstrap-mirrors" [plain @7:13..7:61]
        "releases" [plain @8:3..8:11]: "https://github.com/spack/spack-bootstrap-mirrors/releases" [plain @8:13..8:70]
    "#);
}
