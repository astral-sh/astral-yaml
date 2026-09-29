#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("getsops-sops-functional-tests-res-no-mac-05eebe6c"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "myapp1" [plain @1:1..1:7]: "ENC[AES256_GCM,data:NjYyOQ1GyFUhy25XX7cz7CZo4tLt,iv:WeWmnZCG7/pm71VMd37ua3LsZ6NnqQzLAyN8ylfYHb8=,tag:zh2hOpxHZ8ON+sJgB+OnTg==,type:str]" [plain @1:9..1:144]
      "sops" [plain @2:1..2:5]: mapping [block]
        "kms" [plain @3:5..3:8]: sequence [flow]
        "gcp_kms" [plain @4:5..4:12]: sequence [flow]
        "azure_kv" [plain @5:5..5:13]: sequence [flow]
        "lastmodified" [plain @6:5..6:17]: "2019-08-29T22:23:01Z" [single-quoted @6:19..6:41]
        "pgp" [plain @7:5..7:8]: sequence [block]
          - mapping [block]
            "created_at" [plain @8:9..8:19]: "2019-08-29T22:23:00Z" [single-quoted @8:21..8:43]
            "enc" [plain @9:9..9:12]: literal [@9:14..22:1]
              "-----BEGIN PGP MESSAGE-----\n"
              "\n"
              "wcBMAyUpShfNkFB/AQgABuzcPMJSK+DvnPKZVyYlmDLCM9BobLz/7/F8zyX4O00T\n"
              "UN7/0Lc+MwZWUOe7lmTVgorDLcfxLONbLg9nxXO+apbb2gRABRYdggosMFaRUn3+\n"
              "m+maKQ4yDTIjIvblVa/olN8reD4Bt037mT52IxTZBLDAnyVw45dRTe9mzJ73eTi7\n"
              "1Mk5s6s1ZH52SQHtbY4TSqCnOKodn3UkTjnFmDXMSAzCwBDmK/wv+fDufXSkLpeR\n"
              "AGufxo697as2bQ8mpqgPPTvB04MgXwnTngUQ8BfagyOndgTCrU9YMA1gewJnJJj4\n"
              "azRP9SOukqtn0c/LpewiOHHk/w00dMLEk0E8DpxMItLgAeQQGaX95PJfl3vuPeRT\n"
              "2CA24Qbb4DbgseGimeBG4oS0oCLg4OVDd1tvVtu4+6VGKJr8YvgDk7WpDAK33LoO\n"
              "QBWfQ5mPIeAN5GVWJ2Mk1qcSQTlO9LmM1vPiWojLjOEvZAA=\n"
              "=5Gax\n"
              "-----END PGP MESSAGE-----"
            "fp" [plain @22:9..22:11]: "FBC7B9E2A4F9289AC0C1D4843D16CEE4A27381B4" [plain @22:13..22:53]
        "unencrypted_suffix" [plain @23:5..23:23]: "_unencrypted" [plain @23:25..23:37]
        "version" [plain @24:5..24:12]: "3.3.1" [plain @24:14..24:19]
    "#);
}
