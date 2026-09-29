#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("envoyproxy-examples-front-proxy-envoy-3de81c13"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "static_resources" [plain @1:1..1:17]: mapping [block]
        "listeners" [plain @2:3..2:12]: sequence [block]
          - mapping [block]
            "address" [plain @3:5..3:12]: mapping [block]
              "socket_address" [plain @4:7..4:21]: mapping [block]
                "address" [plain @5:9..5:16]: "0.0.0.0" [plain @5:18..5:25]
                "port_value" [plain @6:9..6:19]: "8080" [plain @6:21..6:25]
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
                            "name" [plain @17:15..17:19]: "backend" [plain @17:21..17:28]
                            "domains" [plain @18:15..18:22]: sequence [block]
                              - "*" [double-quoted @19:17..19:20]
                            "routes" [plain @20:15..20:21]: sequence [block]
                              - mapping [block]
                                "match" [plain @21:17..21:22]: mapping [block]
                                  "prefix" [plain @22:19..22:25]: "/service/1" [double-quoted @22:27..22:39]
                                "route" [plain @23:17..23:22]: mapping [block]
                                  "cluster" [plain @24:19..24:26]: "service1-envoy" [plain @24:28..24:42]
                              - mapping [block]
                                "match" [plain @25:17..25:22]: mapping [block]
                                  "prefix" [plain @26:19..26:25]: "/service/2" [double-quoted @26:27..26:39]
                                "route" [plain @27:17..27:22]: mapping [block]
                                  "cluster" [plain @28:19..28:26]: "service2-envoy" [plain @28:28..28:42]
                      "http_filters" [plain @29:11..29:23]: sequence [block]
                        - mapping [block]
                          "name" [plain @30:13..30:17]: "envoy.filters.http.router" [plain @30:19..30:44]
                          "typed_config" [plain @31:13..31:25]: mapping [block]
                            "@type" [double-quoted @32:15..32:22]: "type.googleapis.com/envoy.extensions.filters.http.router.v3.Router" [plain @32:24..32:90]
          - mapping [block]
            "address" [plain @34:5..34:12]: mapping [block]
              "socket_address" [plain @35:7..35:21]: mapping [block]
                "address" [plain @36:9..36:16]: "0.0.0.0" [plain @36:18..36:25]
                "port_value" [plain @37:9..37:19]: "8443" [plain @37:21..37:25]
            "filter_chains" [plain @38:5..38:18]: sequence [block]
              - mapping [block]
                "filters" [plain @39:7..39:14]: sequence [block]
                  - mapping [block]
                    "name" [plain @40:9..40:13]: "envoy.filters.network.http_connection_manager" [plain @40:15..40:60]
                    "typed_config" [plain @41:9..41:21]: mapping [block]
                      "@type" [double-quoted @42:11..42:18]: "type.googleapis.com/envoy.extensions.filters.network.http_connection_manager.v3.HttpConnectionManager" [plain @42:20..42:121]
                      "codec_type" [plain @43:11..43:21]: "AUTO" [plain @43:23..43:27]
                      "stat_prefix" [plain @44:11..44:22]: "ingress_http" [plain @44:24..44:36]
                      "route_config" [plain @45:11..45:23]: mapping [block]
                        "name" [plain @46:13..46:17]: "local_route" [plain @46:19..46:30]
                        "virtual_hosts" [plain @47:13..47:26]: sequence [block]
                          - mapping [block]
                            "name" [plain @48:15..48:19]: "backend" [plain @48:21..48:28]
                            "domains" [plain @49:15..49:22]: sequence [block]
                              - "*" [double-quoted @50:17..50:20]
                            "routes" [plain @51:15..51:21]: sequence [block]
                              - mapping [block]
                                "match" [plain @52:17..52:22]: mapping [block]
                                  "prefix" [plain @53:19..53:25]: "/service/1" [double-quoted @53:27..53:39]
                                "route" [plain @54:17..54:22]: mapping [block]
                                  "cluster" [plain @55:19..55:26]: "service1-envoy" [plain @55:28..55:42]
                              - mapping [block]
                                "match" [plain @56:17..56:22]: mapping [block]
                                  "prefix" [plain @57:19..57:25]: "/service/2" [double-quoted @57:27..57:39]
                                "route" [plain @58:17..58:22]: mapping [block]
                                  "cluster" [plain @59:19..59:26]: "service2-envoy" [plain @59:28..59:42]
                      "http_filters" [plain @60:11..60:23]: sequence [block]
                        - mapping [block]
                          "name" [plain @61:13..61:17]: "envoy.filters.http.router" [plain @61:19..61:44]
                          "typed_config" [plain @62:13..62:25]: mapping [block]
                            "@type" [double-quoted @63:15..63:22]: "type.googleapis.com/envoy.extensions.filters.http.router.v3.Router" [plain @63:24..63:90]
                "transport_socket" [plain @65:7..65:23]: mapping [block]
                  "name" [plain @66:9..66:13]: "envoy.transport_sockets.tls" [plain @66:15..66:42]
                  "typed_config" [plain @67:9..67:21]: mapping [block]
                    "@type" [double-quoted @68:11..68:18]: "type.googleapis.com/envoy.extensions.transport_sockets.tls.v3.DownstreamTlsContext" [plain @68:20..68:102]
                    "common_tls_context" [plain @69:11..69:29]: mapping [block]
                      "tls_certificates" [plain @70:13..70:29]: sequence [block]
                        - mapping [block]
                          "certificate_chain" [plain @80:15..80:32]: mapping [block]
                            "inline_string" [plain @81:17..81:30]: literal [@81:32..99:1]
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
                          "private_key" [plain @99:15..99:26]: mapping [block]
                            "inline_string" [plain @100:17..100:30]: literal [@100:32..130:1]
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
        "clusters" [plain @130:3..130:11]: sequence [block]
          - mapping [block]
            "name" [plain @131:5..131:9]: "service1-envoy" [plain @131:11..131:25]
            "type" [plain @132:5..132:9]: "STRICT_DNS" [plain @132:11..132:21]
            "lb_policy" [plain @133:5..133:14]: "ROUND_ROBIN" [plain @133:16..133:27]
            "load_assignment" [plain @134:5..134:20]: mapping [block]
              "cluster_name" [plain @135:7..135:19]: "service1-envoy" [plain @135:21..135:35]
              "endpoints" [plain @136:7..136:16]: sequence [block]
                - mapping [block]
                  "lb_endpoints" [plain @137:9..137:21]: sequence [block]
                    - mapping [block]
                      "endpoint" [plain @138:11..138:19]: mapping [block]
                        "address" [plain @139:13..139:20]: mapping [block]
                          "socket_address" [plain @140:15..140:29]: mapping [block]
                            "address" [plain @141:17..141:24]: "service-envoy-1" [plain @141:26..141:41]
                            "port_value" [plain @142:17..142:27]: "8000" [plain @142:29..142:33]
          - mapping [block]
            "name" [plain @143:5..143:9]: "service2-envoy" [plain @143:11..143:25]
            "type" [plain @144:5..144:9]: "STRICT_DNS" [plain @144:11..144:21]
            "lb_policy" [plain @145:5..145:14]: "ROUND_ROBIN" [plain @145:16..145:27]
            "load_assignment" [plain @146:5..146:20]: mapping [block]
              "cluster_name" [plain @147:7..147:19]: "service2-envoy" [plain @147:21..147:35]
              "endpoints" [plain @148:7..148:16]: sequence [block]
                - mapping [block]
                  "lb_endpoints" [plain @149:9..149:21]: sequence [block]
                    - mapping [block]
                      "endpoint" [plain @150:11..150:19]: mapping [block]
                        "address" [plain @151:13..151:20]: mapping [block]
                          "socket_address" [plain @152:15..152:29]: mapping [block]
                            "address" [plain @153:17..153:24]: "service-envoy-2" [plain @153:26..153:41]
                            "port_value" [plain @154:17..154:27]: "8000" [plain @154:29..154:33]
      "admin" [plain @155:1..155:6]: mapping [block]
        "address" [plain @156:3..156:10]: mapping [block]
          "socket_address" [plain @157:5..157:19]: mapping [block]
            "address" [plain @158:7..158:14]: "0.0.0.0" [plain @158:16..158:23]
            "port_value" [plain @159:7..159:17]: "8001" [plain @159:19..159:23]
      "layered_runtime" [plain @160:1..160:16]: mapping [block]
        "layers" [plain @161:3..161:9]: sequence [block]
          - mapping [block]
            "name" [plain @162:5..162:9]: "static_layer_0" [plain @162:11..162:25]
            "static_layer" [plain @163:5..163:17]: mapping [block]
              "envoy" [plain @164:7..164:12]: mapping [block]
                "resource_limits" [plain @165:9..165:24]: mapping [block]
                  "listener" [plain @166:11..166:19]: mapping [block]
                    "example_listener_name" [plain @167:13..167:34]: mapping [block]
                      "connection_limit" [plain @168:15..168:31]: "10000" [plain @168:33..168:38]
    "#);
}
