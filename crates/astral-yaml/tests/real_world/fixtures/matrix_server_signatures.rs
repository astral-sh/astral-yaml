#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_rejection("matrix-server-signatures"), @r#"
    matrix-server-signatures.yaml:19:1: flow continuation must be indented
    bytes 731..732
      17 |     "ed25519:0": "these86bytesofbase64signaturecoveressentialfieldsincludinghashessocancheckredactedpdus"
      18 |   }
      19 | }
         | ^
      20 | additionalProperties:
    "#);
}
