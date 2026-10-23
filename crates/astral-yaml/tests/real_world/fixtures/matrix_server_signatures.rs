#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_rejection("matrix-server-signatures"), @r#"
       |
    17 |     "ed25519:0": "these86bytesofbase64signaturecoveressentialfieldsincludinghashessocancheckredactedpdus"
    18 |   }
    19 | }
       | ^ flow continuation must be indented
    "#);
}
