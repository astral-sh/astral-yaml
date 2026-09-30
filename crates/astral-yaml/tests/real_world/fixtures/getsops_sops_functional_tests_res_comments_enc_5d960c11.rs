#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("getsops-sops-functional-tests-res-comments-enc-5d960c11"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "lorem" [plain @2:1..2:6]: "ENC[AES256_GCM,data:PhmSdTs=,iv:J5ugEWq6RfyNx+5zDXvcTdoQ18YYZkqesDED7LNzou4=,tag:0Qrom6J6aUnZMZzGz5XCxw==,type:str]" [plain @2:8..2:123]
      "dolor" [plain @4:1..4:6]: "ENC[AES256_GCM,data:IgvT,iv:wtPNYbDTARFE810PH6ldOLzCDcAjkB/dzPsZjpgHcko=,tag:zwE8P+AwO1hrHkgF6pTbZw==,type:str]" [plain @4:8..4:119]
      "sops" [plain @5:1..5:5]: mapping [block]
        "kms" [plain @6:5..6:8]: sequence [flow]
        "gcp_kms" [plain @7:5..7:12]: sequence [flow]
        "azure_kv" [plain @8:5..8:13]: sequence [flow]
        "hc_vault" [plain @9:5..9:13]: sequence [flow]
        "age" [plain @10:5..10:8]: sequence [flow]
        "lastmodified" [plain @11:5..11:17]: "2020-10-07T15:49:13Z" [single-quoted @11:19..11:41]
        "mac" [plain @12:5..12:8]: "ENC[AES256_GCM,data:2dhyKdHYSynjXPwYrn9356wA7vRKw+T5qwBenI2vZrgthpQBOCQG4M6f7eeH3VLTxB4mN4CAchb25dsNRoGr6A38VruaSSAhPco3Rh4AlvKSvXuhgRnzZvNxE/bnHX1D4K5cdTb4FsJg/Ue1l7UcWrlrv1s3H3SwLHP/nf+suD0=,iv:6xBYURjjaQzlUOKOrs2NWOChiNFZVAGPJZQZ59MwX3o=,tag:uXD5VYme+c8eHcCc5TD2YA==,type:str]" [plain @12:10..12:289]
        "pgp" [plain @13:5..13:8]: sequence [block]
          - mapping [block]
            "created_at" [plain @14:9..14:19]: "2019-08-29T21:52:32Z" [single-quoted @14:21..14:43]
            "enc" [plain @15:9..15:12]: literal [@15:14..28:1]
              "-----BEGIN PGP MESSAGE-----\n"
              "\n"
              "hQEMAyUpShfNkFB/AQgAlvpTj0NYqF4mQyIeM7wX2SHLb4U07/flpqDpp2W/30Pz\n"
              "AHA7sYrgP0l8BrjT2kwtgCN0cdfoIHJudezrNjANp2P5TbP2b9kYYNxpehzB9PFj\n"
              "FixnCS7Zp8WIt1yXr1TX+ANZoXLopVcRbMaQ5OdH7CN1pNQtMR+R3FR3X/IqKxiU\n"
              "Do1YLaooRJICUC8LJw2Tb4K+lYnTSqd/HalLGym++ivFvdDB1Ya1GhT1FswXidXK\n"
              "IRjsOVbxV0q5VeNOR0zxsheOvuHyCje16c7NXJtATJVWtTFABJB8u7CY5HhZSgq+\n"
              "rXJHyLHqVLzJ8E4WqHQkMNUlVcrqAz7glZ6xbAhfI9JeAYk5SuBOQOQ4yvASqH4K\n"
              "b0N3+/abluBY7YPqKuRZBiEtmcYlZ+zIHuOTP1rD/7L5VY8CwE5U8SFlEqwM7nQJ\n"
              "6/vtl6qngOFjwt34WrhZzUfLPB/wRV/m1Qv2kr0RNA==\n"
              "=Ykiw\n"
              "-----END PGP MESSAGE-----\n"
            "fp" [plain @28:9..28:11]: "FBC7B9E2A4F9289AC0C1D4843D16CEE4A27381B4" [plain @28:13..28:53]
        "unencrypted_suffix" [plain @29:5..29:23]: "_unencrypted" [plain @29:25..29:37]
        "version" [plain @30:5..30:12]: "3.6.1" [plain @30:14..30:19]
    "#);
}
