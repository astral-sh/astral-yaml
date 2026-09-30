#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("envoyproxy-examples-zstd-envoy-3525bc34"), @r#"
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
                      "stat_prefix" [plain @12:11..12:22]: "ingress_http" [plain @12:24..12:36]
                      "route_config" [plain @13:11..13:23]: mapping [block]
                        "name" [plain @14:13..14:17]: "local_route" [plain @14:19..14:30]
                        "virtual_hosts" [plain @15:13..15:26]: sequence [block]
                          - mapping [block]
                            "name" [plain @16:15..16:19]: "backend" [plain @16:21..16:28]
                            "domains" [plain @17:15..17:22]: sequence [block]
                              - "*" [double-quoted @18:17..18:20]
                            "routes" [plain @19:15..19:21]: sequence [block]
                              - mapping [block]
                                "match" [plain @20:17..20:22]: mapping [block]
                                  "prefix" [plain @21:19..21:25]: "/" [double-quoted @21:27..21:30]
                                "route" [plain @22:17..22:22]: mapping [block]
                                  "cluster" [plain @23:19..23:26]: "service" [plain @23:28..23:35]
                      "http_filters" [plain @24:11..24:23]: sequence [block]
                        - mapping [block]
                          "name" [plain @25:13..25:17]: "envoy.filters.http.compressor" [plain @25:19..25:48]
                          "typed_config" [plain @26:13..26:25]: mapping [block]
                            "@type" [double-quoted @27:15..27:22]: "type.googleapis.com/envoy.extensions.filters.http.compressor.v3.Compressor" [plain @27:24..27:98]
                            "response_direction_config" [plain @28:15..28:40]: mapping [block]
                              "common_config" [plain @29:17..29:30]: mapping [block]
                                "min_content_length" [plain @30:19..30:37]: "100" [plain @30:39..30:42]
                                "content_type" [plain @31:19..31:31]: sequence [block]
                                  - "application/json" [plain @32:21..32:37]
                              "disable_on_etag_header" [plain @33:17..33:39]: "true" [plain @33:41..33:45]
                            "compressor_library" [plain @34:15..34:33]: mapping [block]
                              "name" [plain @35:17..35:21]: "text_optimized" [plain @35:23..35:37]
                              "typed_config" [plain @36:17..36:29]: mapping [block]
                                "@type" [double-quoted @37:19..37:26]: "type.googleapis.com/envoy.extensions.compression.zstd.compressor.v3.Zstd" [plain @37:28..37:100]
                                "compression_level" [plain @38:19..38:36]: "10" [plain @38:38..38:40]
                        - mapping [block]
                          "name" [plain @39:13..39:17]: "envoy.filters.http.router" [plain @39:19..39:44]
                          "typed_config" [plain @40:13..40:25]: mapping [block]
                            "@type" [double-quoted @41:15..41:22]: "type.googleapis.com/envoy.extensions.filters.http.router.v3.Router" [plain @41:24..41:90]
                "transport_socket" [plain @42:7..42:23]: mapping [block]
                  "name" [plain @43:9..43:13]: "envoy.transport_sockets.tls" [plain @43:15..43:42]
                  "typed_config" [plain @44:9..44:21]: mapping [block]
                    "@type" [double-quoted @45:11..45:18]: "type.googleapis.com/envoy.extensions.transport_sockets.tls.v3.DownstreamTlsContext" [plain @45:20..45:102]
                    "common_tls_context" [plain @46:11..46:29]: mapping [block]
                      "tls_certificates" [plain @47:13..47:29]: sequence [block]
                        - mapping [block]
                          "certificate_chain" [plain @56:15..56:32]: mapping [block]
                            "inline_string" [plain @57:17..57:30]: literal [@57:32..75:1]
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
                          "private_key" [plain @75:15..75:26]: mapping [block]
                            "inline_string" [plain @76:17..76:30]: literal [@76:32..105:1]
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
          - mapping [block]
            "address" [plain @105:5..105:12]: mapping [block]
              "socket_address" [plain @106:7..106:21]: mapping [block]
                "address" [plain @107:9..107:16]: "0.0.0.0" [plain @107:18..107:25]
                "port_value" [plain @108:9..108:19]: "9902" [plain @108:21..108:25]
            "filter_chains" [plain @109:5..109:18]: sequence [block]
              - mapping [block]
                "filters" [plain @110:7..110:14]: sequence [block]
                  - mapping [block]
                    "name" [plain @111:9..111:13]: "envoy.filters.network.http_connection_manager" [plain @111:15..111:60]
                    "typed_config" [plain @112:9..112:21]: mapping [block]
                      "@type" [double-quoted @113:11..113:18]: "type.googleapis.com/envoy.extensions.filters.network.http_connection_manager.v3.HttpConnectionManager" [plain @113:20..113:121]
                      "stat_prefix" [plain @114:11..114:22]: "ingress_http" [plain @114:24..114:36]
                      "route_config" [plain @115:11..115:23]: mapping [block]
                        "name" [plain @116:13..116:17]: "local_route" [plain @116:19..116:30]
                        "virtual_hosts" [plain @117:13..117:26]: sequence [block]
                          - mapping [block]
                            "name" [plain @118:15..118:19]: "backend" [plain @118:21..118:28]
                            "domains" [plain @119:15..119:22]: sequence [block]
                              - "*" [double-quoted @120:17..120:20]
                            "routes" [plain @121:15..121:21]: sequence [block]
                              - mapping [block]
                                "match" [plain @122:17..122:22]: mapping [block]
                                  "prefix" [plain @123:19..123:25]: "/stats/prometheus" [double-quoted @123:27..123:46]
                                "route" [plain @124:17..124:22]: mapping [block]
                                  "cluster" [plain @125:19..125:26]: "envoy-stats" [plain @125:28..125:39]
                      "http_filters" [plain @126:11..126:23]: sequence [block]
                        - mapping [block]
                          "name" [plain @127:13..127:17]: "envoy.filters.http.compressor" [plain @127:19..127:48]
                          "typed_config" [plain @128:13..128:25]: mapping [block]
                            "@type" [double-quoted @129:15..129:22]: "type.googleapis.com/envoy.extensions.filters.http.compressor.v3.Compressor" [plain @129:24..129:98]
                            "response_direction_config" [plain @130:15..130:40]: mapping [block]
                              "common_config" [plain @131:17..131:30]: mapping [block]
                                "min_content_length" [plain @132:19..132:37]: "100" [plain @132:39..132:42]
                                "content_type" [plain @133:19..133:31]: sequence [block]
                                  - "text/plain" [plain @134:21..134:31]
                              "disable_on_etag_header" [plain @135:17..135:39]: "true" [plain @135:41..135:45]
                            "compressor_library" [plain @136:15..136:33]: mapping [block]
                              "name" [plain @137:17..137:21]: "text_optimized" [plain @137:23..137:37]
                              "typed_config" [plain @138:17..138:29]: mapping [block]
                                "@type" [double-quoted @139:19..139:26]: "type.googleapis.com/envoy.extensions.compression.zstd.compressor.v3.Zstd" [plain @139:28..139:100]
                                "compression_level" [plain @140:19..140:36]: "10" [plain @140:38..140:40]
                        - mapping [block]
                          "name" [plain @141:13..141:17]: "envoy.filters.http.router" [plain @141:19..141:44]
                          "typed_config" [plain @142:13..142:25]: mapping [block]
                            "@type" [double-quoted @143:15..143:22]: "type.googleapis.com/envoy.extensions.filters.http.router.v3.Router" [plain @143:24..143:90]
                "transport_socket" [plain @144:7..144:23]: mapping [block]
                  "name" [plain @145:9..145:13]: "envoy.transport_sockets.tls" [plain @145:15..145:42]
                  "typed_config" [plain @146:9..146:21]: mapping [block]
                    "@type" [double-quoted @147:11..147:18]: "type.googleapis.com/envoy.extensions.transport_sockets.tls.v3.DownstreamTlsContext" [plain @147:20..147:102]
                    "common_tls_context" [plain @148:11..148:29]: mapping [block]
                      "tls_certificates" [plain @149:13..149:29]: sequence [block]
                        - mapping [block]
                          "certificate_chain" [plain @158:15..158:32]: mapping [block]
                            "inline_string" [plain @159:17..159:30]: literal [@159:32..177:1]
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
                          "private_key" [plain @177:15..177:26]: mapping [block]
                            "inline_string" [plain @178:17..178:30]: literal [@178:32..208:1]
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
        "clusters" [plain @208:3..208:11]: sequence [block]
          - mapping [block]
            "name" [plain @209:5..209:9]: "envoy-stats" [plain @209:11..209:22]
            "connect_timeout" [plain @210:5..210:20]: "0.25s" [plain @210:22..210:27]
            "type" [plain @211:5..211:9]: "STATIC" [plain @211:11..211:17]
            "load_assignment" [plain @212:5..212:20]: mapping [block]
              "cluster_name" [plain @213:7..213:19]: "envoy-stats" [plain @213:21..213:32]
              "endpoints" [plain @214:7..214:16]: sequence [block]
                - mapping [block]
                  "lb_endpoints" [plain @215:9..215:21]: sequence [block]
                    - mapping [block]
                      "endpoint" [plain @216:11..216:19]: mapping [block]
                        "address" [plain @217:13..217:20]: mapping [block]
                          "socket_address" [plain @218:15..218:29]: mapping [block]
                            "address" [plain @219:17..219:24]: "127.0.0.1" [plain @219:26..219:35]
                            "port_value" [plain @220:17..220:27]: "9901" [plain @220:29..220:33]
          - mapping [block]
            "name" [plain @221:5..221:9]: "service" [plain @221:11..221:18]
            "connect_timeout" [plain @222:5..222:20]: "0.25s" [plain @222:22..222:27]
            "type" [plain @223:5..223:9]: "STRICT_DNS" [plain @223:11..223:21]
            "lb_policy" [plain @224:5..224:14]: "ROUND_ROBIN" [plain @224:16..224:27]
            "load_assignment" [plain @225:5..225:20]: mapping [block]
              "cluster_name" [plain @226:7..226:19]: "service" [plain @226:21..226:28]
              "endpoints" [plain @227:7..227:16]: sequence [block]
                - mapping [block]
                  "lb_endpoints" [plain @228:9..228:21]: sequence [block]
                    - mapping [block]
                      "endpoint" [plain @229:11..229:19]: mapping [block]
                        "address" [plain @230:13..230:20]: mapping [block]
                          "socket_address" [plain @231:15..231:29]: mapping [block]
                            "address" [plain @232:17..232:24]: "service" [plain @232:26..232:33]
                            "port_value" [plain @233:17..233:27]: "8080" [plain @233:29..233:33]
      "admin" [plain @234:1..234:6]: mapping [block]
        "address" [plain @235:3..235:10]: mapping [block]
          "socket_address" [plain @236:5..236:19]: mapping [block]
            "address" [plain @237:7..237:14]: "0.0.0.0" [plain @237:16..237:23]
            "port_value" [plain @238:7..238:17]: "9901" [plain @238:19..238:23]
    "#);
}
