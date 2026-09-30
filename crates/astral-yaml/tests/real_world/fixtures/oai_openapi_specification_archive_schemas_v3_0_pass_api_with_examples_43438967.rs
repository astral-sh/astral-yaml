#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("oai-openapi-specification-archive-schemas-v3-0-pass-api-with-examples-43438967"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "openapi" [plain @1:1..1:8]: "3.0.0" [double-quoted @1:10..1:17]
      "info" [plain @2:1..2:5]: mapping [block]
        "title" [plain @3:3..3:8]: "Simple API overview" [plain @3:10..3:29]
        "version" [plain @4:3..4:10]: "2.0.0" [plain @4:12..4:17]
      "paths" [plain @5:1..5:6]: mapping [block]
        "/" [plain @6:3..6:4]: mapping [block]
          "get" [plain @7:5..7:8]: mapping [block]
            "operationId" [plain @8:7..8:18]: "listVersionsv2" [plain @8:20..8:34]
            "summary" [plain @9:7..9:14]: "List API versions" [plain @9:16..9:33]
            "responses" [plain @10:7..10:16]: mapping [block]
              "200" [single-quoted @11:9..11:14]: mapping [block]
                "description" [plain @12:11..12:22]: "200 response" [literal @12:24..14:1]
                "content" [plain @14:11..14:18]: mapping [block]
                  "application/json" [plain @15:13..15:29]: mapping [block]
                    "examples" [plain @16:15..16:23]: mapping [block]
                      "foo" [plain @17:17..17:20]: mapping [block]
                        "value" [plain @18:19..18:24]: mapping [flow]
                          "versions" [double-quoted @20:23..20:33]: sequence [flow]
                            - mapping [flow]
                              "status" [double-quoted @22:29..22:37]: "CURRENT" [double-quoted @22:39..22:48]
                              "updated" [double-quoted @23:29..23:38]: "2011-01-21T11:33:21Z" [double-quoted @23:40..23:62]
                              "id" [double-quoted @24:29..24:33]: "v2.0" [double-quoted @24:35..24:41]
                              "links" [double-quoted @25:29..25:36]: sequence [flow]
                                - mapping [flow]
                                  "href" [double-quoted @27:37..27:43]: "http://127.0.0.1:8774/v2/" [double-quoted @27:45..27:72]
                                  "rel" [double-quoted @28:37..28:42]: "self" [double-quoted @28:44..28:50]
                            - mapping [flow]
                              "status" [double-quoted @33:29..33:37]: "EXPERIMENTAL" [double-quoted @33:39..33:53]
                              "updated" [double-quoted @34:29..34:38]: "2013-07-23T11:33:21Z" [double-quoted @34:40..34:62]
                              "id" [double-quoted @35:29..35:33]: "v3.0" [double-quoted @35:35..35:41]
                              "links" [double-quoted @36:29..36:36]: sequence [flow]
                                - mapping [flow]
                                  "href" [double-quoted @38:37..38:43]: "http://127.0.0.1:8774/v3/" [double-quoted @38:45..38:72]
                                  "rel" [double-quoted @39:37..39:42]: "self" [double-quoted @39:44..39:50]
              "300" [single-quoted @45:9..45:14]: mapping [block]
                "description" [plain @46:11..46:22]: "300 response" [literal @46:24..48:1]
                "content" [plain @48:11..48:18]: mapping [block]
                  "application/json" [plain @49:13..49:29]: mapping [block]
                    "examples" [plain @50:15..50:23]: mapping [block]
                      "foo" [plain @51:17..51:20]: mapping [block]
                        "value" [plain @52:19..52:24]: literal [@52:26..79:1]
                          "{\n"
                          " \"versions\": [\n"
                          "       {\n"
                          "         \"status\": \"CURRENT\",\n"
                          "         \"updated\": \"2011-01-21T11:33:21Z\",\n"
                          "         \"id\": \"v2.0\",\n"
                          "         \"links\": [\n"
                          "             {\n"
                          "                 \"href\": \"http://127.0.0.1:8774/v2/\",\n"
                          "                 \"rel\": \"self\"\n"
                          "             }\n"
                          "         ]\n"
                          "     },\n"
                          "     {\n"
                          "         \"status\": \"EXPERIMENTAL\",\n"
                          "         \"updated\": \"2013-07-23T11:33:21Z\",\n"
                          "         \"id\": \"v3.0\",\n"
                          "         \"links\": [\n"
                          "             {\n"
                          "                 \"href\": \"http://127.0.0.1:8774/v3/\",\n"
                          "                 \"rel\": \"self\"\n"
                          "             }\n"
                          "         ]\n"
                          "     }\n"
                          " ]\n"
                          "}\n"
        "/v2" [plain @79:3..79:6]: mapping [block]
          "get" [plain @80:5..80:8]: mapping [block]
            "operationId" [plain @81:7..81:18]: "getVersionDetailsv2" [plain @81:20..81:39]
            "summary" [plain @82:7..82:14]: "Show API version details" [plain @82:16..82:40]
            "responses" [plain @83:7..83:16]: mapping [block]
              "200" [single-quoted @84:9..84:14]: mapping [block]
                "description" [plain @85:11..85:22]: "200 response" [literal @85:24..87:1]
                "content" [plain @87:11..87:18]: mapping [block]
                  "application/json" [plain @88:13..88:29]: mapping [block]
                    "examples" [plain @89:15..89:23]: mapping [block]
                      "foo" [plain @90:17..90:20]: mapping [block]
                        "value" [plain @91:19..91:24]: mapping [flow]
                          "version" [double-quoted @93:23..93:32]: mapping [flow]
                            "status" [double-quoted @94:25..94:33]: "CURRENT" [double-quoted @94:35..94:44]
                            "updated" [double-quoted @95:25..95:34]: "2011-01-21T11:33:21Z" [double-quoted @95:36..95:58]
                            "media-types" [double-quoted @96:25..96:38]: sequence [flow]
                              - mapping [flow]
                                "base" [double-quoted @98:31..98:37]: "application/xml" [double-quoted @98:39..98:56]
                                "type" [double-quoted @99:31..99:37]: "application/vnd.openstack.compute+xml;version=2" [double-quoted @99:39..99:88]
                              - mapping [flow]
                                "base" [double-quoted @102:31..102:37]: "application/json" [double-quoted @102:39..102:57]
                                "type" [double-quoted @103:31..103:37]: "application/vnd.openstack.compute+json;version=2" [double-quoted @103:39..103:89]
                            "id" [double-quoted @106:25..106:29]: "v2.0" [double-quoted @106:31..106:37]
                            "links" [double-quoted @107:25..107:32]: sequence [flow]
                              - mapping [flow]
                                "href" [double-quoted @109:31..109:37]: "http://127.0.0.1:8774/v2/" [double-quoted @109:39..109:66]
                                "rel" [double-quoted @110:31..110:36]: "self" [double-quoted @110:38..110:44]
                              - mapping [flow]
                                "href" [double-quoted @113:31..113:37]: "http://docs.openstack.org/api/openstack-compute/2/os-compute-devguide-2.pdf" [double-quoted @113:39..113:116]
                                "type" [double-quoted @114:31..114:37]: "application/pdf" [double-quoted @114:39..114:56]
                                "rel" [double-quoted @115:31..115:36]: "describedby" [double-quoted @115:38..115:51]
                              - mapping [flow]
                                "href" [double-quoted @118:31..118:37]: "http://docs.openstack.org/api/openstack-compute/2/wadl/os-compute-2.wadl" [double-quoted @118:39..118:113]
                                "type" [double-quoted @119:31..119:37]: "application/vnd.sun.wadl+xml" [double-quoted @119:39..119:69]
                                "rel" [double-quoted @120:31..120:36]: "describedby" [double-quoted @120:38..120:51]
                              - mapping [flow]
                                "href" [double-quoted @123:29..123:35]: "http://docs.openstack.org/api/openstack-compute/2/wadl/os-compute-2.wadl" [double-quoted @123:37..123:111]
                                "type" [double-quoted @124:29..124:35]: "application/vnd.sun.wadl+xml" [double-quoted @124:37..124:67]
                                "rel" [double-quoted @125:29..125:34]: "describedby" [double-quoted @125:36..125:49]
              "203" [single-quoted @130:9..130:14]: mapping [block]
                "description" [plain @131:11..131:22]: "203 response" [literal @131:24..133:1]
                "content" [plain @133:11..133:18]: mapping [block]
                  "application/json" [plain @134:13..134:29]: mapping [block]
                    "examples" [plain @135:15..135:23]: mapping [block]
                      "foo" [plain @136:17..136:20]: mapping [block]
                        "value" [plain @137:19..137:24]: mapping [flow]
                          "version" [double-quoted @139:23..139:32]: mapping [flow]
                            "status" [double-quoted @140:25..140:33]: "CURRENT" [double-quoted @140:35..140:44]
                            "updated" [double-quoted @141:25..141:34]: "2011-01-21T11:33:21Z" [double-quoted @141:36..141:58]
                            "media-types" [double-quoted @142:25..142:38]: sequence [flow]
                              - mapping [flow]
                                "base" [double-quoted @144:31..144:37]: "application/xml" [double-quoted @144:39..144:56]
                                "type" [double-quoted @145:31..145:37]: "application/vnd.openstack.compute+xml;version=2" [double-quoted @145:39..145:88]
                              - mapping [flow]
                                "base" [double-quoted @148:31..148:37]: "application/json" [double-quoted @148:39..148:57]
                                "type" [double-quoted @149:31..149:37]: "application/vnd.openstack.compute+json;version=2" [double-quoted @149:39..149:89]
                            "id" [double-quoted @152:25..152:29]: "v2.0" [double-quoted @152:31..152:37]
                            "links" [double-quoted @153:25..153:32]: sequence [flow]
                              - mapping [flow]
                                "href" [double-quoted @155:31..155:37]: "http://23.253.228.211:8774/v2/" [double-quoted @155:39..155:71]
                                "rel" [double-quoted @156:31..156:36]: "self" [double-quoted @156:38..156:44]
                              - mapping [flow]
                                "href" [double-quoted @159:31..159:37]: "http://docs.openstack.org/api/openstack-compute/2/os-compute-devguide-2.pdf" [double-quoted @159:39..159:116]
                                "type" [double-quoted @160:31..160:37]: "application/pdf" [double-quoted @160:39..160:56]
                                "rel" [double-quoted @161:31..161:36]: "describedby" [double-quoted @161:38..161:51]
                              - mapping [flow]
                                "href" [double-quoted @164:31..164:37]: "http://docs.openstack.org/api/openstack-compute/2/wadl/os-compute-2.wadl" [double-quoted @164:39..164:113]
                                "type" [double-quoted @165:31..165:37]: "application/vnd.sun.wadl+xml" [double-quoted @165:39..165:69]
                                "rel" [double-quoted @166:31..166:36]: "describedby" [double-quoted @166:38..166:51]
    "#);
}
