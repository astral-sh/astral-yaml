#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("getsops-sops-functional-tests-res-multiple-keys-a1ea1ab4"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "message" [plain @1:1..1:8]: "ENC[AES256_GCM,data:4soxy1Q=,iv:GGOFFLu8aqMEyoOBdMGdevug7E0R8H0iDyXIKQ5ufzM=,tag:954eipkbSu2U64tx2usiBA==,type:str]" [plain @1:10..1:125]
      "sops" [plain @2:1..2:5]: mapping [block]
        "kms" [plain @3:5..3:8]: sequence [flow]
        "gcp_kms" [plain @4:5..4:12]: sequence [flow]
        "azure_kv" [plain @5:5..5:13]: sequence [flow]
        "lastmodified" [plain @6:5..6:17]: "2019-08-29T22:22:04Z" [single-quoted @6:19..6:41]
        "mac" [plain @7:5..7:8]: "ENC[AES256_GCM,data:Rcyh6DKgpd8WNvLqnPhMRYDGztK/WjNJ+SisCD4qL50wxiYUgwRGs5S9I8kOFm4+8pruOcQyCVIDzyjQHZfM1bKs+CnMJTrp1HDz2vhIKkE8O6EdE5+JbNxwc6ZFQE6YThmx6rz93gMHnDOcrO0XG2guGxTumM5xPkZsEBfpIyQ=,iv:YI0Yc2khjy+q1BxiEBDejgymyAQmBkdyl3fQUm76/1Y=,tag:pf4ZbMy9TrpTA/B2qVEkmw==,type:str]" [plain @7:10..7:289]
        "pgp" [plain @8:5..8:8]: sequence [block]
          - mapping [block]
            "created_at" [plain @9:9..9:19]: "2019-08-29T22:22:04Z" [single-quoted @9:21..9:43]
            "enc" [plain @10:9..10:12]: literal [@10:14..23:1]
              "-----BEGIN PGP MESSAGE-----\n"
              "\n"
              "wcBMAyUpShfNkFB/AQgAQH8KFrXipAGaHsKjQecpZ4R5PsWg4nqetgsy6FTcAI3j\n"
              "KwvBoXapI+2m5aXjAdO4k76ZxGyyXXdwe6ebbuikxCuDl28m86o9xNxbBv5SxbaO\n"
              "gTxtUR+swcnYRyfVmkd07HhATXQw7b2q4ERNJAVH9zxJcmzNzP6q9eRjePRWXSjn\n"
              "xxzpnS9kQkevuzJXg5dhVZ9HnsuVEthK88c9H3ZZQt/kg3GLaj2XDhFUt9Xy3POl\n"
              "3Q2dE8xzVXzg5rF+0n8H0WpiRJ7hbvW6kvAanKNUDeNLY2shmkKiIzh4W0f2Vl5Q\n"
              "W1nw23Tga6IQ6mSqLItZB5bYX/C9K33kceV1DYyW+tLgAeTofEdQuueh2myAYmmV\n"
              "Wjap4eAq4F/gi+EpuOAS4jDaEhPgY+UkojNLl9ow2EkyrIMMJCKDSzuZcmZhwTYl\n"
              "7/RSqGHDHuDX5GvXVgnRiqnoDiPsImCDN5/iNmoSVuG9cgA=\n"
              "=wVy4\n"
              "-----END PGP MESSAGE-----"
            "fp" [plain @23:9..23:11]: "FBC7B9E2A4F9289AC0C1D4843D16CEE4A27381B4" [plain @23:13..23:53]
          - mapping [block]
            "created_at" [plain @24:9..24:19]: "2019-08-29T22:22:04Z" [single-quoted @24:21..24:43]
            "enc" [plain @25:9..25:12]: literal [@25:14..35:1]
              "-----BEGIN PGP MESSAGE-----\n"
              "\n"
              "hIwDXFUltYFwV4MBA/9/qqKDEbT9oZJqB7Z7R1RAz+TStVUzlE9+M0ScFuguBUpd\n"
              "7doLhhgBYa9ygti5pnAN4E1SI97xPhflGZLTp4OtdTg5UGOUwh6BM0BZI1ai6dlK\n"
              "u4ra0BKOmDzv9SkbTngStXfPPLC9msc6sZe9TER88puqNAZ4DBrTyTtEAmg5wtJe\n"
              "AecV7hxDzAGleAdmaTC+WHkvg/EgwY/9PaU1IMf6tCx8O/bS8JQhxmptO1aAm5Vr\n"
              "dO/SV2m9QStMafO6OG1aGpy3zkVytlxGavkUK+dPp5fYbRYGQVfUiGpgRRZEZg==\n"
              "=NlDf\n"
              "-----END PGP MESSAGE-----\n"
            "fp" [plain @35:9..35:11]: "D7229043384BCC60326C6FB9D8720D957C3D3074" [plain @35:13..35:53]
        "unencrypted_suffix" [plain @36:5..36:23]: "_unencrypted" [plain @36:25..36:37]
        "version" [plain @37:5..37:12]: "3.3.1" [plain @37:14..37:19]
    "#);
}
