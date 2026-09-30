#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("envoyproxy-examples-websocket-envoy-wss-1971da66"), @r#"
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
                      "stat_prefix" [plain @12:11..12:22]: "ingress_wss_to_wss" [plain @12:24..12:42]
                      "upgrade_configs" [plain @13:11..13:26]: sequence [block]
                        - mapping [block]
                          "upgrade_type" [plain @14:13..14:25]: "websocket" [plain @14:27..14:36]
                      "route_config" [plain @15:11..15:23]: mapping [block]
                        "name" [plain @16:13..16:17]: "local_route" [plain @16:19..16:30]
                        "virtual_hosts" [plain @17:13..17:26]: sequence [block]
                          - mapping [block]
                            "name" [plain @18:15..18:19]: "app" [plain @18:21..18:24]
                            "domains" [plain @19:15..19:22]: sequence [block]
                              - "*" [double-quoted @20:17..20:20]
                            "routes" [plain @21:15..21:21]: sequence [block]
                              - mapping [block]
                                "match" [plain @22:17..22:22]: mapping [block]
                                  "prefix" [plain @23:19..23:25]: "/" [double-quoted @23:27..23:30]
                                "route" [plain @24:17..24:22]: mapping [block]
                                  "cluster" [plain @25:19..25:26]: "service_wss" [plain @25:28..25:39]
                      "http_filters" [plain @26:11..26:23]: sequence [block]
                        - mapping [block]
                          "name" [plain @27:13..27:17]: "envoy.filters.http.router" [plain @27:19..27:44]
                          "typed_config" [plain @28:13..28:25]: mapping [block]
                            "@type" [double-quoted @29:15..29:22]: "type.googleapis.com/envoy.extensions.filters.http.router.v3.Router" [plain @29:24..29:90]
                "transport_socket" [plain @30:7..30:23]: mapping [block]
                  "name" [plain @31:9..31:13]: "envoy.transport_sockets.tls" [plain @31:15..31:42]
                  "typed_config" [plain @32:9..32:21]: mapping [block]
                    "@type" [double-quoted @33:11..33:18]: "type.googleapis.com/envoy.extensions.transport_sockets.tls.v3.DownstreamTlsContext" [plain @33:20..33:102]
                    "common_tls_context" [plain @34:11..34:29]: mapping [block]
                      "tls_certificates" [plain @35:13..35:29]: sequence [block]
                        - mapping [block]
                          "certificate_chain" [plain @45:15..45:32]: mapping [block]
                            "inline_string" [plain @46:17..46:30]: literal [@46:32..64:1]
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
                          "private_key" [plain @64:15..64:26]: mapping [block]
                            "inline_string" [plain @65:17..65:30]: literal [@65:32..95:1]
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
        "clusters" [plain @95:3..95:11]: sequence [block]
          - mapping [block]
            "name" [plain @96:5..96:9]: "service_wss" [plain @96:11..96:22]
            "type" [plain @97:5..97:9]: "STRICT_DNS" [plain @97:11..97:21]
            "lb_policy" [plain @98:5..98:14]: "ROUND_ROBIN" [plain @98:16..98:27]
            "load_assignment" [plain @99:5..99:20]: mapping [block]
              "cluster_name" [plain @100:7..100:19]: "service_wss" [plain @100:21..100:32]
              "endpoints" [plain @101:7..101:16]: sequence [block]
                - mapping [block]
                  "lb_endpoints" [plain @102:9..102:21]: sequence [block]
                    - mapping [block]
                      "endpoint" [plain @103:11..103:19]: mapping [block]
                        "address" [plain @104:13..104:20]: mapping [block]
                          "socket_address" [plain @105:15..105:29]: mapping [block]
                            "address" [plain @106:17..106:24]: "service-wss" [plain @106:26..106:37]
                            "port_value" [plain @107:17..107:27]: "443" [plain @107:29..107:32]
            "transport_socket" [plain @108:5..108:21]: mapping [block]
              "name" [plain @109:7..109:11]: "envoy.transport_sockets.tls" [plain @109:13..109:40]
              "typed_config" [plain @110:7..110:19]: mapping [block]
                "@type" [double-quoted @111:9..111:16]: "type.googleapis.com/envoy.extensions.transport_sockets.tls.v3.UpstreamTlsContext" [plain @111:18..111:98]
    "#);
}
