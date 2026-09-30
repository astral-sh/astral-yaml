#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("istio-istio-samples-jwt-server-jwt-server-26226f99"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @17:1..17:11]: "v1" [plain @17:13..17:15]
      "kind" [plain @18:1..18:5]: "Service" [plain @18:7..18:14]
      "metadata" [plain @19:1..19:9]: mapping [block]
        "name" [plain @20:3..20:7]: "jwt-server" [plain @20:9..20:19]
        "labels" [plain @21:3..21:9]: mapping [block]
          "app" [plain @22:5..22:8]: "jwt-server" [plain @22:10..22:20]
      "spec" [plain @23:1..23:5]: mapping [block]
        "ports" [plain @24:3..24:8]: sequence [block]
          - mapping [block]
            "name" [plain @25:5..25:9]: "http" [plain @25:11..25:15]
            "port" [plain @26:5..26:9]: "8000" [plain @26:11..26:15]
            "targetPort" [plain @27:5..27:15]: "8000" [plain @27:17..27:21]
          - mapping [block]
            "name" [plain @28:5..28:9]: "https" [plain @28:11..28:16]
            "port" [plain @29:5..29:9]: "8443" [plain @29:11..29:15]
            "targetPort" [plain @30:5..30:15]: "8443" [plain @30:17..30:21]
        "selector" [plain @31:3..31:11]: mapping [block]
          "app" [plain @32:5..32:8]: "jwt-server" [plain @32:10..32:20]

    document 2 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @34:1..34:11]: "v1" [plain @34:13..34:15]
      "kind" [plain @35:1..35:5]: "Secret" [plain @35:7..35:13]
      "metadata" [plain @36:1..36:9]: mapping [block]
        "name" [plain @37:3..37:7]: "jwt-cert-key-secret" [plain @37:9..37:28]
      "stringData" [plain @40:1..40:11]: mapping [block]
        "server.crt" [plain @41:3..41:13]: literal [@41:15..64:1]
          "-----BEGIN CERTIFICATE-----\n"
          "MIIDjzCCAnegAwIBAgIUfIuuQDfWakIpZ7bZAuuLUWhSm2AwDQYJKoZIhvcNAQEL\n"
          "BQAwRjELMAkGA1UEBhMCVVMxCzAJBgNVBAgMAkFaMRMwEQYDVQQKDApBY21lLCBJ\n"
          "bmMuMRUwEwYDVQQDDAxBY21lIFJvb3QgQ0EwHhcNMjExMTE3MDQ0NDE2WhcNMzEx\n"
          "MTE1MDQ0NDE2WjA/MQswCQYDVQQGEwJVUzELMAkGA1UECAwCQVoxEzARBgNVBAoM\n"
          "CkFjbWUsIEluYy4xDjAMBgNVBAMMBSouY29tMIIBIjANBgkqhkiG9w0BAQEFAAOC\n"
          "AQ8AMIIBCgKCAQEAv3qsDI+3Fc65EuuPnKG4BN0dLZZy+wFNxruszYRg0foP9kUQ\n"
          "rUUv12uu/y2Rguf09y9mXXzGc51kwU5TIhVarYPBIa46MLMBBroF908VX9ng4Q9M\n"
          "ta+rU10e9xugRRnCDf1ZMlQJB/7pmnF21vw6gdmRt7vMLKiHQuN9BI+042Z/NiiF\n"
          "T7xCDDz+HvhGnn+vDv53h6LPzwNM2zGLSIPaV5xkYs0fYvs5Y2pUGonrra5hGoRq\n"
          "JzOZ3SNfKtaQ3AXrf/+kikJGFA/GmzZuhW26Nygl/kYgx7l+g3uTXOz0hN434nF6\n"
          "Cc7EyuvD37lAsgw1w48poTnDUijV5Cx6yA8FHQIDAQABo3wwejB4BgNVHREEcTBv\n"
          "ggpqd3Qtc2VydmVygglsb2NhbGhvc3SCF2p3dC1zZXJ2ZXIuaXN0aW8tc3lzdGVt\n"
          "ghJqd3Qtc2VydmVyLmRlZmF1bHSCKWp3dC1zZXJ2ZXIuaXN0aW8tc3lzdGVtLnN2\n"
          "Yy5jbHVzdGVyLmxvY2FsMA0GCSqGSIb3DQEBCwUAA4IBAQC26tlBXaF+chVS3f8w\n"
          "Tv1D1lgXgJ/ROozqlSMe5BGDuOgsVtWQeqpMIXxEx8w6fXUF0TaMYxp3sC4D3Ql9\n"
          "W+PALf9Zpy+vv6vxoKkrnKiXvOiuYkLJhaVDkzvj6j6yMjxUk5a9ehDZ0gKwXf+m\n"
          "Ei35D8xKtPdz/FaB7qgN2mu7V47oFizon7jLLqAvlWIIQ7Pku+XfjraDPtjxUj4u\n"
          "5qSrIfSWAeuJSEsSlGPyYJCFvqFNQYW0y8y7fCCQo7FObHfBmpp7kG2BViuLxebW\n"
          "zfi4K3gDCpR8lWiNEjm4NamQ07OpCtmLZfaueZH/vSXXVVbs6VCsb6nJqJrGDc5t\n"
          "K/xK\n"
          "-----END CERTIFICATE-----\n"
        "server.key" [plain @64:3..64:13]: literal [@64:15..93:1]
          "-----BEGIN PRIVATE KEY-----\n"
          "MIIEvAIBADANBgkqhkiG9w0BAQEFAASCBKYwggSiAgEAAoIBAQC/eqwMj7cVzrkS\n"
          "64+cobgE3R0tlnL7AU3Gu6zNhGDR+g/2RRCtRS/Xa67/LZGC5/T3L2ZdfMZznWTB\n"
          "TlMiFVqtg8EhrjowswEGugX3TxVf2eDhD0y1r6tTXR73G6BFGcIN/VkyVAkH/uma\n"
          "cXbW/DqB2ZG3u8wsqIdC430Ej7TjZn82KIVPvEIMPP4e+Eaef68O/neHos/PA0zb\n"
          "MYtIg9pXnGRizR9i+zljalQaieutrmEahGonM5ndI18q1pDcBet//6SKQkYUD8ab\n"
          "Nm6Fbbo3KCX+RiDHuX6De5Nc7PSE3jficXoJzsTK68PfuUCyDDXDjymhOcNSKNXk\n"
          "LHrIDwUdAgMBAAECggEAPtk99ZWKa58BwkMNTUULiJUnCZqTPO4NoEhjjMWBngot\n"
          "CRFcSvMlo9iVhO5pD4WhMy0ctVzKKpKjyosx4EMQE1nmn253bRqkIJgYczdC9cYm\n"
          "+NgzvoLdgixTiJpJvcSZnEvm5g0NNdGmzWmmryP09D/8g0kh2Bqs4viWRVQB9I1T\n"
          "eKbPOwatlDBiqitHDPQ7ZAGBvXfHbRc8PePQ2biJJWN/JzOrhBMG9tKpAqduQgbW\n"
          "u/DR06JI5Gpp9LiOXcThDbSB/XLdwxLY64MAU4ihWRsQ2k+FNrnuOLDM3YNCrWF5\n"
          "MRKRVrUhAwDs3V4my4uVu65QjDWURTg7mnbzzwKAgQKBgQDopj9ZKuNXw5T7Yuj2\n"
          "BnYDd4h9gz2BAtQR0ACXoeFhRipmfX3TZdrfklbE5IryRZObSGBxMw/Jf+jseyT3\n"
          "9nhE8dRrR2yxvlN/SMNP+uW3wziSRriGM8+WB2mkBEhxPrbIPyAQFupkeE6iuY0c\n"
          "14cNiKRXPrz4lE5tBZPCECEtIQKBgQDSspYwXuakP6jeiZOym1rRfj58Xi6Hfra3\n"
          "4e4elTsgj+iKvw/5vqn+/axqmZzymxY6vOECSlxKDee+inxHvZxr9de7DXv8rr8x\n"
          "w+nna/hnKUzqiplbDEQCqMH0US3k9fbNX/AknGccYQO9kiYj23Gi6cnRZAVrm7oy\n"
          "MEQIFgB8fQKBgHPLQx5zbUIic4WHnmHNp3FkTkgCSVtr9/eBqrnN9ap/zNzEOxs7\n"
          "x+udH5jSE6IwJR6VsILHImVtR5ZkWGsefo/6OXrHyv7QtyhUI/or66hB/2c20eLh\n"
          "6MFIoTjkdNYAm+MhIClB7pnhE2qEpgqj73E6AGn4LQAgeMRkkT1237xhAoGAJoPW\n"
          "yIjQiH3KlMN5aFDVzS3SplFhGAulwv9d0+FbqZwk2hgLB5A+6wncFrB17DNFYP9d\n"
          "8lk9fZwFHOObzFFw4ptSEDNq0snu0V4Kx+8IvXLjSIyFdAtN81599fdQ+GWt8+Tx\n"
          "tP+SKbHiSSkKJ8vZffpWlhw+kWkqJDqGdSPwetECgYBzekGqr0MrrnK1nsXwd1pe\n"
          "Y+KypdjOfu7SI9I1ujosSTo3XZ9+EJo2vJYy1acCLFrp1s8eaUhc/NTT57R/EIOL\n"
          "8mpQUbVH8l8h6gRs6izoPFhtOKJZgkPrx7OCs08CCmYIr9qUvWFhcnsxnW7B5hic\n"
          "LEAqdR15WVMSx/Fw8dACEw==\n"
          "-----END PRIVATE KEY-----\n"

    document 3 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @94:1..94:11]: "apps/v1" [plain @94:13..94:20]
      "kind" [plain @95:1..95:5]: "Deployment" [plain @95:7..95:17]
      "metadata" [plain @96:1..96:9]: mapping [block]
        "name" [plain @97:3..97:7]: "jwt-server" [plain @97:9..97:19]
      "spec" [plain @98:1..98:5]: mapping [block]
        "replicas" [plain @99:3..99:11]: "1" [plain @99:13..99:14]
        "selector" [plain @100:3..100:11]: mapping [block]
          "matchLabels" [plain @101:5..101:16]: mapping [block]
            "app" [plain @102:7..102:10]: "jwt-server" [plain @102:12..102:22]
        "template" [plain @103:3..103:11]: mapping [block]
          "metadata" [plain @104:5..104:13]: mapping [block]
            "labels" [plain @105:7..105:13]: mapping [block]
              "app" [plain @106:9..106:12]: "jwt-server" [plain @106:14..106:24]
          "spec" [plain @107:5..107:9]: mapping [block]
            "containers" [plain @108:7..108:17]: sequence [block]
              - mapping [block]
                "image" [plain @109:9..109:14]: "registry.istio.io/testing/jwt-server:0.8" [plain @109:16..109:56]
                "imagePullPolicy" [plain @110:9..110:24]: "IfNotPresent" [plain @110:26..110:38]
                "name" [plain @111:9..111:13]: "jwt-server" [plain @111:15..111:25]
                "volumeMounts" [plain @112:9..112:21]: sequence [block]
                  - mapping [block]
                    "name" [plain @113:13..113:17]: "certkeysecretresource" [plain @113:19..113:40]
                    "mountPath" [plain @114:13..114:22]: "/app/https/secretresources" [double-quoted @114:24..114:52]
                    "readOnly" [plain @115:13..115:21]: "true" [plain @115:23..115:27]
                "args" [plain @116:9..116:13]: sequence [flow]
                  - "-https" [double-quoted @116:16..116:24]
                  - "8443" [double-quoted @116:26..116:32]
                  - "-cert" [double-quoted @116:34..116:41]
                  - "/app/https/secretresources/server.crt" [double-quoted @116:43..116:82]
                  - "-key" [double-quoted @116:84..116:90]
                  - "/app/https/secretresources/server.key" [double-quoted @116:92..116:131]
                "ports" [plain @117:9..117:14]: sequence [block]
                  - mapping [block]
                    "containerPort" [plain @118:11..118:24]: "8000" [plain @118:26..118:30]
                  - mapping [block]
                    "containerPort" [plain @119:11..119:24]: "8443" [plain @119:26..119:30]
            "volumes" [plain @120:7..120:14]: sequence [block]
              - mapping [block]
                "name" [plain @121:9..121:13]: "certkeysecretresource" [plain @121:15..121:36]
                "secret" [plain @122:9..122:15]: mapping [block]
                  "secretName" [plain @123:11..123:21]: "jwt-cert-key-secret" [plain @123:23..123:42]
                  "defaultMode" [plain @124:11..124:22]: "0400" [plain @124:24..124:28]

    document 4 (start: ---, end: implicit)
    "" [plain @125:4..125:4]
    "#);
}
