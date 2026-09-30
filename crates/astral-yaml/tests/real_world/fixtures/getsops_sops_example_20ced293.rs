#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("getsops-sops-example-20ced293"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "myapp1" [plain @1:1..1:7]: "ENC[AES256_GCM,data:zlGNmhTYX5xol4ZZFsiaoGkD73nn,iv:ql9mkhoU1I64E/FJi3iA0HaAe2U3kQVFee2ZLwPnBik=,tag:SqVSfu/JkRrwqidAT/i0pg==,type:str]" [plain @1:9..1:144]
      "app2" [plain @2:1..2:5]: mapping [block]
        "db" [plain @3:5..3:7]: mapping [block]
          "user" [plain @4:9..4:13]: "ENC[AES256_GCM,data:tQ1l,iv:o9MiMveNYO7T82yDab+4pAt17DO6B4wl8yGy3oFbDb8=,tag:0RjvUUtSQwaUc9SF3RSTZQ==,type:str]" [plain @4:15..4:126]
          "password" [plain @5:9..5:17]: "ENC[AES256_GCM,data:8Ll+TDCgzQ==,iv:aao0OSVdFwaB9EGZ0O+Wn5bRJ6do7hgiQioqiwDi1w0=,tag:WKj0nuSSPkm2dBOBwinYvg==,type:str]" [plain @5:19..5:138]
        "key" [plain @7:5..7:8]: "ENC[AES256_GCM,data:xRjmLiX4BCoSUElToUs5twDq1WNWQNvNMi8yitXly43iGQiwltIs0FsY5u+7fzLepS66oLTGdL0NfWwCGxksYYzUKz5OXRLEatacZP40D71861zu+njmGdXepY0q0q5VOG7ObgIAMMMElVKRIFdjpVgmgUa+/h6R77mEbDztk6lqb28r15XyR6GmdubierTE7aialFzNoC+XO/yk7bnMi0XA/aomj1H5RdZ37LBR2k+rhqXmhkPdGTdJ39t4Ou1Q2Oc4RHRGvgs/EeFqQHcq0AilqXsFIv/PE0bQP564LaOcXK33B+PnoV1D+lXZ7mLOjKlq04c+UojwXjpZeXBr6Ip4H3dAGkPAoQKMtyGwHYzuLCNesxwPv2tPqbxkbVev0AKezGhnPjvCNvRN3S1Y0LPf1atfwzOCBQBhdUTpmXtxCdTNG0cUUZjeIKAyJXDWJooYHlstzDti/dSGMEedOnKq6648Yp7tLNDjAg5CGbDEWjTWehtqgvixUoRdYc2/r/ie3t09XB9h70BzLFDnbNdTNHhg0/aivHeDf8LJ2Co8YvIjLTwlm7GV0mSwzJIoY/2YxXFtw+XFqt+BCt1G8wq3R6OdXZK/+6fUKH/D4EVym5nrGkZIuOCiTB+wpzy09QmZ80Fo+ba0x/1g9ZTMKHk=,iv:ZtzvrO7QSHEOCnKCrIYcaesKnyScV8KaHZr22tUMLlU=,tag:2A3nJBIPF2Q3FlwMYLvG2w==,type:str]" [plain @7:10..7:781]
      "number" [plain @8:1..8:7]: "ENC[AES256_GCM,data:DX0qiTOWhQvG/w==,iv:ouWsby8JoFwCRj/mLVCnNcYhP2sdyf4h6nwZuGksE7Q=,tag:lPU6AId2JrlquHnYRw+E8Q==,type:float]" [plain @8:9..8:134]
      "an_array" [plain @9:1..9:9]: sequence [block]
        - "ENC[AES256_GCM,data:vyczE8EQr9qHkaM=,iv:sT5jKk3LZ61Zq/neTli5tcnDFxCxY5RuGr2k5oGQWJQ=,tag:1HgaHWfyh6EJLkI1V2kOrw==,type:str]" [plain @10:7..10:130]
        - "ENC[AES256_GCM,data:XtBinnYXR7bx1GY=,iv:KvT9smKVmgMNrab+RzfuWscyvJav2r8j1P08ucNmhgQ=,tag:IllAEvIPPeKOfqi1XbmT8w==,type:str]" [plain @11:7..11:130]
        - "ENC[AES256_GCM,data:gpZ7nwWTGaI+Ti+lk+CPQOoM0ypwK7UMMBUiZAniQHDNJelipqc8hyhNeV+tpJLNaRt74OHs04EX8g==,iv:mKcwVelqLvwVDPjR8NeyMZ7AhsjRgmnYmyEuwPNPrQ8=,tag:vrkoccUfJs105yLCmXYYCw==,type:str]" [plain @12:7..12:194]
        - "ENC[AES256_GCM,data:L9jPh+7+XsdqEpUnFcD4nA==,iv:xyfKjOXVrBDCIQG5786pSu5yvHdl/PK8eVxkIUWoCIw=,tag:Q0wlTV2e2vZeU6eTF5Oacg==,type:str]" [plain @13:7..13:138]
      "somebooleans" [plain @14:1..14:13]: sequence [block]
        - "ENC[AES256_GCM,data:ExiXxg==,iv:K7FUwomqdA7o9lzvNoAMH/wbXs08FextTGGeJKnaatU=,tag:A9UntgvPIcappmeM3jsbdA==,type:bool]" [plain @15:7..15:123]
        - "ENC[AES256_GCM,data:3I0AVdM=,iv:q4YKnRIKufREPmwT4sz8plcsOD6iem/tY3NMUV0STBE=,tag:0w4OMKClWTzjKhqsJZT8JA==,type:bool]" [plain @16:7..16:123]
      "this" [plain @17:1..17:5]: mapping [block]
        "is" [plain @18:5..18:7]: mapping [block]
          "a" [plain @19:9..19:10]: mapping [block]
            "nested" [plain @20:13..20:19]: mapping [block]
              "value" [plain @21:17..21:22]: "ENC[AES256_GCM,data:oFn5fJS5+slb2sCdLY5SxZ+iWeowWtf4wn9g,iv:MZ7i4tZnfCQhQRUwXV2fYQPIJ0tTUFLiD9xuB+765e8=,tag:ZEjE5jvxE5HkN3mma84pKw==,type:str]" [plain @21:24..21:167]
      "somelist_unencrypted" [plain @25:1..25:21]: sequence [block]
        - "all elements of this list" [plain @26:7..26:32]
        - "remain in clear text" [plain @27:7..27:27]
        - "because of the _unencrypted suffix in the key" [plain @28:7..28:52]
      "nested_unencrypted" [plain @29:1..29:19]: mapping [block]
        "this" [plain @30:5..30:9]: mapping [block]
          "is" [plain @31:9..31:11]: mapping [block]
            "all" [plain @32:13..32:16]: "going to remain in clear text" [plain @32:18..32:47]
      "sops" [plain @33:1..33:5]: mapping [block]
        "lastmodified" [plain @34:5..34:17]: "2025-08-06T18:55:05Z" [double-quoted @34:19..34:41]
        "mac" [plain @35:5..35:8]: "ENC[AES256_GCM,data:GCWOyMJo56xAfC4he5zJUuQr+uYXpSPX61twiGf+bi6E9Mb6JNEWGz8GhnxtlXDNinTPrTrmjeep0H1B9EzhSjxkjXuvTGrIUZmO3paGqpkyszU9pm4APyPtoq6ggCo+rqIMG+EDYFN4wMKPU+Nyn2EjmrorAg1uHq2ORKN40vk=,iv:V1kH6xaXDIb8NHYNm4A5ilTInizW7gw8nWhINZh9OJs=,tag:u+JwZ5osXC6FHdytwvRwvw==,type:str]" [plain @35:10..35:289]
        "pgp" [plain @36:5..36:8]: sequence [block]
          - mapping [block]
            "created_at" [plain @37:11..37:21]: "2019-12-10T22:45:53Z" [double-quoted @37:23..37:45]
            "enc" [plain @38:11..38:14]: literal [@38:16..51:1]
              "-----BEGIN PGP MESSAGE-----\n"
              "\n"
              "wcBMAyUpShfNkFB/AQgAE0MaWAQGbTKY7Xg3fDNtzlvnVBkkQRHsLt5kUTu2nAy4\n"
              "sPX0NRXPVF/tAMxr9mI2fRjKnNBXKpOAecNis85D/QEkfflG8/syGkqiJqy9Nqon\n"
              "WSm1bNriPfD4PL850688EJe49Xrsz6rVbW5FYZCHMbnPxvmoheMJRxLonW3/eWPy\n"
              "IjJ9i6Z7W175mv1y7FELOimdQeelynp4r8bOuuq1BhePB4+wJXihw9n0ovuLklpl\n"
              "Kr2iCmIUibSywlEO/LGQT/VXo6R7xgSN2Xg0RwWfflajYHNhVkHlnNYzkACjvdhj\n"
              "ph1M5fLGDqPu+ySSe93EyahNhdwKgQ7R9yF8/13+QdLgAeRmrBgh0N54mQQAQyL7\n"
              "mH/i4SWi4BjgmeHgUeBL4qcm2avg6OUasVJQHlTeA8D+c3TwKSTRVDijN4GBadYJ\n"
              "Z5/vXKOseeBk5JWCnIHC/MtjOkuPt53nvGzi3lYvW+F0FQA=\n"
              "=RDyn\n"
              "-----END PGP MESSAGE-----"
            "fp" [plain @51:11..51:13]: "FBC7B9E2A4F9289AC0C1D4843D16CEE4A27381B4" [plain @51:15..51:55]
          - mapping [block]
            "created_at" [plain @52:11..52:21]: "2019-12-10T22:45:53Z" [double-quoted @52:23..52:45]
            "enc" [plain @53:11..53:14]: literal [@53:16..64:1]
              "-----BEGIN PGP MESSAGE-----\n"
              "\n"
              "wYwDXFUltYFwV4MBBABpm+tFhFhv3A7A/L/p6nL3HXKKhONrgguYgXA/hhSg4/bD\n"
              "1Po5pQhCM4yb3gqWewxgVpGNKFr/Gl+kN9eZ3LXp5nEdhei/aQn7BbWkhph5PKt6\n"
              "faiEZAL5PNHvktvEQwPsfNJvxe8QT2Z9oFmlueP0n3mCZz3UV9LZHNwOP7XfzdLg\n"
              "AeRzUGrZK43KavmIjdgPXcd/4ZW14NPgNeGm8+BO4nD26Fvgh+UiCb0TpBiI7WsX\n"
              "HQGUlFjuR6Vd7Q+vg8B/1Ovm3fUw7uCG5Gc2WeB0M+pXaLYS9bCDL9jisGOkD+Ef\n"
              "ZAA=\n"
              "=mqGc\n"
              "-----END PGP MESSAGE-----"
            "fp" [plain @64:11..64:13]: "D7229043384BCC60326C6FB9D8720D957C3D3074" [plain @64:15..64:55]
        "unencrypted_suffix" [plain @65:5..65:23]: "_unencrypted" [plain @65:25..65:37]
        "version" [plain @66:5..66:12]: "3.10.2" [plain @66:14..66:20]
    "#);
}
