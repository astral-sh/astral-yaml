#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("istio-istio-samples-external-pypi-1b762dba"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @8:1..8:11]: "networking.istio.io/v1" [plain @8:13..8:35]
      "kind" [plain @9:1..9:5]: "ServiceEntry" [plain @9:7..9:19]
      "metadata" [plain @10:1..10:9]: mapping [block]
        "name" [plain @11:3..11:7]: "python-https" [plain @11:9..11:21]
      "spec" [plain @12:1..12:5]: mapping [block]
        "hosts" [plain @13:3..13:8]: sequence [block]
          - "pypi.python.org" [plain @14:5..14:20]
        "ports" [plain @15:3..15:8]: sequence [block]
          - mapping [block]
            "number" [plain @16:5..16:11]: "443" [plain @16:13..16:16]
            "name" [plain @17:5..17:9]: "https" [plain @17:11..17:16]
            "protocol" [plain @18:5..18:13]: "HTTPS" [plain @18:15..18:20]

    document 2 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @21:1..21:11]: "networking.istio.io/v1" [plain @21:13..21:35]
      "kind" [plain @22:1..22:5]: "ServiceEntry" [plain @22:7..22:19]
      "metadata" [plain @23:1..23:9]: mapping [block]
        "name" [plain @24:3..24:7]: "pypi-https" [plain @24:9..24:19]
      "spec" [plain @25:1..25:5]: mapping [block]
        "hosts" [plain @26:3..26:8]: sequence [block]
          - "pypi.org" [plain @27:5..27:13]
        "ports" [plain @28:3..28:8]: sequence [block]
          - mapping [block]
            "number" [plain @29:5..29:11]: "443" [plain @29:13..29:16]
            "name" [plain @30:5..30:9]: "https" [plain @30:11..30:16]
            "protocol" [plain @31:5..31:13]: "HTTPS" [plain @31:15..31:20]

    document 3 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @34:1..34:11]: "networking.istio.io/v1" [plain @34:13..34:35]
      "kind" [plain @35:1..35:5]: "ServiceEntry" [plain @35:7..35:19]
      "metadata" [plain @36:1..36:9]: mapping [block]
        "name" [plain @37:3..37:7]: "pythonhosted-https" [plain @37:9..37:27]
      "spec" [plain @38:1..38:5]: mapping [block]
        "hosts" [plain @39:3..39:8]: sequence [block]
          - "files.pythonhosted.org" [plain @40:5..40:27]
        "ports" [plain @41:3..41:8]: sequence [block]
          - mapping [block]
            "number" [plain @42:5..42:11]: "443" [plain @42:13..42:16]
            "name" [plain @43:5..43:9]: "https" [plain @43:11..43:16]
            "protocol" [plain @44:5..44:13]: "HTTPS" [plain @44:15..44:20]
    "#);
}
