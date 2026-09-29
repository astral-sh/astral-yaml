#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("getsops-sops-sops-a566a585"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "creation_rules" [plain @1:1..1:15]: sequence [block]
        - mapping [block]
          "pgp" [plain @2:7..2:10]: "FBC7B9E2A4F9289AC0C1D4843D16CEE4A27381B4, D7229043384BCC60326C6FB9D8720D957C3D3074" [folded @2:12..5:1]
    "#);
}
