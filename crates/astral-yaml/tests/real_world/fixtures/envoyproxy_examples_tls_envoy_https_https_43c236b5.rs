#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("envoyproxy-examples-tls-envoy-https-https-43c236b5"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "static_resources" [plain @1:1..1:17]: mapping [block]
        "listeners" [plain @2:3..2:12]: sequence [block]
          - mapping [block]
            "address" [plain @3:5..3:12]: mapping [block]
              "socket_address" [plain @4:7..4:21]: mapping [block]
                "address" [plain @5:9..5:16]: "0.0.0.0" [plain @5:18..5:25]
                "port_value" [plain @6:9..6:19]: "10000" [plain @6:21..6:26]
            "filter_chains" [plain @7:5..7:18]: sequence [block]
              - mapping [block]
                "filters" [plain @8:7..8:14]: sequence [block]
                  - mapping [block]
                    "name" [plain @9:9..9:13]: "envoy.filters.network.http_connection_manager" [plain @9:15..9:60]
                    "typed_config" [plain @10:9..10:21]: mapping [block]
                      "@type" [double-quoted @11:11..11:18]: "type.googleapis.com/envoy.extensions.filters.network.http_connection_manager.v3.HttpConnectionManager" [plain @11:20..11:121]
                      "codec_type" [plain @12:11..12:21]: "AUTO" [plain @12:23..12:27]
                      "stat_prefix" [plain @13:11..13:22]: "ingress_http" [plain @13:24..13:36]
                      "route_config" [plain @14:11..14:23]: mapping [block]
                        "name" [plain @15:13..15:17]: "local_route" [plain @15:19..15:30]
                        "virtual_hosts" [plain @16:13..16:26]: sequence [block]
                          - mapping [block]
                            "name" [plain @17:15..17:19]: "app" [plain @17:21..17:24]
                            "domains" [plain @18:15..18:22]: sequence [block]
                              - "*" [double-quoted @19:17..19:20]
                            "routes" [plain @20:15..20:21]: sequence [block]
                              - mapping [block]
                                "match" [plain @21:17..21:22]: mapping [block]
                                  "prefix" [plain @22:19..22:25]: "/" [double-quoted @22:27..22:30]
                                "route" [plain @23:17..23:22]: mapping [block]
                                  "cluster" [plain @24:19..24:26]: "service-https" [plain @24:28..24:41]
                      "http_filters" [plain @25:11..25:23]: sequence [block]
                        - mapping [block]
                          "name" [plain @26:13..26:17]: "envoy.filters.http.router" [plain @26:19..26:44]
                          "typed_config" [plain @27:13..27:25]: mapping [block]
                            "@type" [double-quoted @28:15..28:22]: "type.googleapis.com/envoy.extensions.filters.http.router.v3.Router" [plain @28:24..28:90]
                "transport_socket" [plain @29:7..29:23]: mapping [block]
                  "name" [plain @30:9..30:13]: "envoy.transport_sockets.tls" [plain @30:15..30:42]
                  "typed_config" [plain @31:9..31:21]: mapping [block]
                    "@type" [double-quoted @32:11..32:18]: "type.googleapis.com/envoy.extensions.transport_sockets.tls.v3.DownstreamTlsContext" [plain @32:20..32:102]
                    "common_tls_context" [plain @33:11..33:29]: mapping [block]
                      "tls_certificates" [plain @34:13..34:29]: sequence [block]
                        - mapping [block]
                          "certificate_chain" [plain @44:15..44:32]: mapping [block]
                            "inline_string" [plain @45:17..45:30]: literal [@45:32..63:1]
                              "-----BEGIN CERTIFICATE-----\n"
                              "MIICqDCCAZACCQCquzpHNpqBcDANBgkqhkiG9w0BAQsFADAWMRQwEgYDVQQDDAtm\n"
                              "cm9udC1lbnZveTAeFw0yMDA3MDgwMTMxNDZaFw0zMDA3MDYwMTMxNDZaMBYxFDAS\n"
                              "BgNVBAMMC2Zyb250LWVudm95MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKC\n"
                              "AQEAthnYkqVQBX+Wg7aQWyCCb87hBce1hAFhbRM8Y9dQTqxoMXZiA2n8G089hUou\n"
                              "oQpEdJgitXVS6YMFPFUUWfwcqxYAynLK4X5im26Yfa1eO8La8sZUS+4Bjao1gF5/\n"
                              "VJxSEo2yZ7fFBo8M4E44ZehIIocipCRS+YZehFs6dmHoq/MGvh2eAHIa+O9xssPt\n"
                              "ofFcQMR8rwBHVbKy484O10tNCouX4yUkyQXqCRy6HRu7kSjOjNKSGtjfG+h5M8bh\n"
                              "10W7ZrsJ1hWhzBulSaMZaUY3vh5ngpws1JATQVSK1Jm/dmMRciwlTK7KfzgxHlSX\n"
                              "58ENpS7yPTISkEICcLbXkkKGEQIDAQABMA0GCSqGSIb3DQEBCwUAA4IBAQCmj6Hg\n"
                              "vwOxWz0xu+6fSfRL6PGJUGq6wghCfUvjfwZ7zppDUqU47fk+yqPIOzuGZMdAqi7N\n"
                              "v1DXkeO4A3hnMD22Rlqt25vfogAaZVToBeQxCPd/ALBLFrvLUFYuSlS3zXSBpQqQ\n"
                              "Ny2IKFYsMllz5RSROONHBjaJOn5OwqenJ91MPmTAG7ujXKN6INSBM0PjX9Jy4Xb9\n"
                              "zT+I85jRDQHnTFce1WICBDCYidTIvJtdSSokGSuy4/xyxAAc/BpZAfOjBQ4G1QRe\n"
                              "9XwOi790LyNUYFJVyeOvNJwveloWuPLHb9idmY5YABwikUY6QNcXwyHTbRCkPB2I\n"
                              "m+/R4XnmL4cKQ+5Z\n"
                              "-----END CERTIFICATE-----\n"
                          "private_key" [plain @63:15..63:26]: mapping [block]
                            "inline_string" [plain @64:17..64:30]: literal [@64:32..94:1]
                              "-----BEGIN PRIVATE KEY-----\n"
                              "MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC2GdiSpVAFf5aD\n"
                              "tpBbIIJvzuEFx7WEAWFtEzxj11BOrGgxdmIDafwbTz2FSi6hCkR0mCK1dVLpgwU8\n"
                              "VRRZ/ByrFgDKcsrhfmKbbph9rV47wtryxlRL7gGNqjWAXn9UnFISjbJnt8UGjwzg\n"
                              "Tjhl6EgihyKkJFL5hl6EWzp2Yeir8wa+HZ4Achr473Gyw+2h8VxAxHyvAEdVsrLj\n"
                              "zg7XS00Ki5fjJSTJBeoJHLodG7uRKM6M0pIa2N8b6HkzxuHXRbtmuwnWFaHMG6VJ\n"
                              "oxlpRje+HmeCnCzUkBNBVIrUmb92YxFyLCVMrsp/ODEeVJfnwQ2lLvI9MhKQQgJw\n"
                              "tteSQoYRAgMBAAECggEAeDGdEkYNCGQLe8pvg8Z0ccoSGpeTxpqGrNEKhjfi6NrB\n"
                              "NwyVav10iq4FxEmPd3nobzDPkAftfvWc6hKaCT7vyTkPspCMOsQJ39/ixOk+jqFx\n"
                              "lNa1YxyoZ9IV2DIHR1iaj2Z5gB367PZUoGTgstrbafbaNY9IOSyojCIO935ubbcx\n"
                              "DWwL24XAf51ez6sXnI8V5tXmrFlNXhbhJdH8iIxNyM45HrnlUlOk0lCK4gmLJjy9\n"
                              "10IS2H2Wh3M5zsTpihH1JvM56oAH1ahrhMXs/rVFXXkg50yD1KV+HQiEbglYKUxO\n"
                              "eMYtfaY9i2CuLwhDnWp3oxP3HfgQQhD09OEN3e0IlQKBgQDZ/3poG9TiMZSjfKqL\n"
                              "xnCABMXGVQsfFWNC8THoW6RRx5Rqi8q08yJrmhCu32YKvccsOljDQJQQJdQO1g09\n"
                              "e/adJmCnTrqxNtjPkX9txV23Lp6Ak7emjiQ5ICu7iWxrcO3zf7hmKtj7z+av8sjO\n"
                              "mDI7NkX5vnlE74nztBEjp3eC0wKBgQDV2GeJV028RW3b/QyP3Gwmax2+cKLR9PKR\n"
                              "nJnmO5bxAT0nQ3xuJEAqMIss/Rfb/macWc2N/6CWJCRT6a2vgy6xBW+bqG6RdQMB\n"
                              "xEZXFZl+sSKhXPkc5Wjb4lQ14YWyRPrTjMlwez3k4UolIJhJmwl+D7OkMRrOUERO\n"
                              "EtUvc7odCwKBgBi+nhdZKWXveM7B5N3uzXBKmmRz3MpPdC/yDtcwJ8u8msUpTv4R\n"
                              "JxQNrd0bsIqBli0YBmFLYEMg+BwjAee7vXeDFq+HCTv6XMva2RsNryCO4yD3I359\n"
                              "XfE6DJzB8ZOUgv4Dvluie3TB2Y6ZQV/p+LGt7G13yG4hvofyJYvlg3RPAoGAcjDg\n"
                              "+OH5zLN2eqah8qBN0CYa9/rFt0AJ19+7/smLTJ7QvQq4g0gwS1couplcCEnNGWiK\n"
                              "72y1n/ckvvplmPeAE19HveMvR9UoCeV5ej86fACy8V/oVpnaaLBvL2aCMjPLjPP9\n"
                              "DWeCIZp8MV86cvOrGfngf6kJG2qZTueXl4NAuwkCgYEArKkhlZVXjwBoVvtHYmN2\n"
                              "o+F6cGMlRJTLhNc391WApsgDZfTZSdeJsBsvvzS/Nc0burrufJg0wYioTlpReSy4\n"
                              "ohhtprnQQAddfjHP7rh2LGt+irFzhdXXQ1ybGaGM9D764KUNCXLuwdly0vzXU4HU\n"
                              "q5sGxGrC1RECGB5Zwx2S2ZY=\n"
                              "-----END PRIVATE KEY-----\n"
        "clusters" [plain @94:3..94:11]: sequence [block]
          - mapping [block]
            "name" [plain @95:5..95:9]: "service-https" [plain @95:11..95:24]
            "type" [plain @96:5..96:9]: "STRICT_DNS" [plain @96:11..96:21]
            "lb_policy" [plain @97:5..97:14]: "ROUND_ROBIN" [plain @97:16..97:27]
            "load_assignment" [plain @98:5..98:20]: mapping [block]
              "cluster_name" [plain @99:7..99:19]: "service-https" [plain @99:21..99:34]
              "endpoints" [plain @100:7..100:16]: sequence [block]
                - mapping [block]
                  "lb_endpoints" [plain @101:9..101:21]: sequence [block]
                    - mapping [block]
                      "endpoint" [plain @102:11..102:19]: mapping [block]
                        "address" [plain @103:13..103:20]: mapping [block]
                          "socket_address" [plain @104:15..104:29]: mapping [block]
                            "address" [plain @105:17..105:24]: "service-https" [plain @105:26..105:39]
                            "port_value" [plain @106:17..106:27]: "443" [plain @106:29..106:32]
            "transport_socket" [plain @107:5..107:21]: mapping [block]
              "name" [plain @108:7..108:11]: "envoy.transport_sockets.tls" [plain @108:13..108:40]
              "typed_config" [plain @109:7..109:19]: mapping [block]
                "@type" [double-quoted @110:9..110:16]: "type.googleapis.com/envoy.extensions.transport_sockets.tls.v3.UpstreamTlsContext" [plain @110:18..110:98]
                "common_tls_context" [plain @111:9..111:27]: mapping [block]
                  "tls_params" [plain @112:11..112:21]: mapping [block]
                    "tls_minimum_protocol_version" [plain @113:13..113:41]: "TLSv1_3" [plain @113:43..113:50]
                    "tls_maximum_protocol_version" [plain @114:13..114:41]: "TLSv1_3" [plain @114:43..114:50]
    "#);
}
