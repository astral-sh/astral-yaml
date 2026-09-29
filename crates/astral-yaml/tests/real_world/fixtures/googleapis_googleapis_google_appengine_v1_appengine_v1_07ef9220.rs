#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("googleapis-googleapis-google-appengine-v1-appengine-v1-07ef9220"), @r###"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "type" [plain @1:1..1:5]: "google.api.Service" [plain @1:7..1:25]
      "config_version" [plain @2:1..2:15]: "3" [plain @2:17..2:18]
      "name" [plain @3:1..3:5]: "appengine.googleapis.com" [plain @3:7..3:31]
      "title" [plain @4:1..4:6]: "App Engine Admin API" [plain @4:8..4:28]
      "apis" [plain @6:1..6:5]: sequence [block]
        - mapping [block]
          "name" [plain @7:3..7:7]: "google.appengine.v1.Applications" [plain @7:9..7:41]
        - mapping [block]
          "name" [plain @8:3..8:7]: "google.appengine.v1.AuthorizedCertificates" [plain @8:9..8:51]
        - mapping [block]
          "name" [plain @9:3..9:7]: "google.appengine.v1.AuthorizedDomains" [plain @9:9..9:46]
        - mapping [block]
          "name" [plain @10:3..10:7]: "google.appengine.v1.DomainMappings" [plain @10:9..10:43]
        - mapping [block]
          "name" [plain @11:3..11:7]: "google.appengine.v1.Firewall" [plain @11:9..11:37]
        - mapping [block]
          "name" [plain @12:3..12:7]: "google.appengine.v1.Instances" [plain @12:9..12:38]
        - mapping [block]
          "name" [plain @13:3..13:7]: "google.appengine.v1.Services" [plain @13:9..13:37]
        - mapping [block]
          "name" [plain @14:3..14:7]: "google.appengine.v1.Versions" [plain @14:9..14:37]
      "types" [plain @16:1..16:6]: sequence [block]
        - mapping [block]
          "name" [plain @17:3..17:7]: "google.appengine.v1.LocationMetadata" [plain @17:9..17:45]
        - mapping [block]
          "name" [plain @18:3..18:7]: "google.appengine.v1.OperationMetadataV1" [plain @18:9..18:48]
      "documentation" [plain @20:1..20:14]: mapping [block]
        "summary" [plain @21:3..21:10]: "Provisions and manages developers' App Engine applications." [plain @21:12..21:71]
        "overview" [plain @22:3..22:11]: literal [@22:13..36:1]
          "# Google App Engine Admin API\n"
          "\n"
          "## Overview\n"
          "\n"
          "The Google App Engine Admin API is a RESTful API for managing App Engine\n"
          "applications. The Admin API provides programmatic access to several of the\n"
          "App Engine administrative operations that are found in the [Google Cloud\n"
          "Platform Console](https://console.cloud.google.com/appengine).\n"
          "\n"
          "## Documentation\n"
          "\n"
          "[Google App Engine Admin API\n"
          "Documentation](https://cloud.google.com/appengine/docs/admin-api/)"
        "rules" [plain @36:3..36:8]: sequence [block]
          - mapping [block]
            "selector" [plain @37:5..37:13]: "google.cloud.location.Locations.GetLocation" [plain @37:15..37:58]
            "description" [plain @38:5..38:16]: "Gets information about a location." [plain @38:18..38:52]
          - mapping [block]
            "selector" [plain @40:5..40:13]: "google.cloud.location.Locations.ListLocations" [plain @40:15..40:60]
            "description" [plain @41:5..41:16]: "Lists information about the supported locations for this service." [plain @41:18..41:83]
      "backend" [plain @43:1..43:8]: mapping [block]
        "rules" [plain @44:3..44:8]: sequence [block]
          - mapping [block]
            "selector" [plain @45:5..45:13]: "google.appengine.v1.Applications.*" [single-quoted @45:15..45:51]
            "deadline" [plain @46:5..46:13]: "30.0" [plain @46:15..46:19]
          - mapping [block]
            "selector" [plain @47:5..47:13]: "google.appengine.v1.AuthorizedCertificates.*" [single-quoted @47:15..47:61]
            "deadline" [plain @48:5..48:13]: "30.0" [plain @48:15..48:19]
          - mapping [block]
            "selector" [plain @49:5..49:13]: "google.appengine.v1.AuthorizedCertificates.GetAuthorizedCertificate" [plain @49:15..49:82]
            "deadline" [plain @50:5..50:13]: "60.0" [plain @50:15..50:19]
          - mapping [block]
            "selector" [plain @51:5..51:13]: "google.appengine.v1.AuthorizedDomains.ListAuthorizedDomains" [plain @51:15..51:74]
            "deadline" [plain @52:5..52:13]: "30.0" [plain @52:15..52:19]
          - mapping [block]
            "selector" [plain @53:5..53:13]: "google.appengine.v1.DomainMappings.*" [single-quoted @53:15..53:53]
            "deadline" [plain @54:5..54:13]: "30.0" [plain @54:15..54:19]
          - mapping [block]
            "selector" [plain @55:5..55:13]: "google.appengine.v1.DomainMappings.GetDomainMapping" [plain @55:15..55:66]
            "deadline" [plain @56:5..56:13]: "60.0" [plain @56:15..56:19]
          - mapping [block]
            "selector" [plain @57:5..57:13]: "google.appengine.v1.DomainMappings.ListDomainMappings" [plain @57:15..57:68]
            "deadline" [plain @58:5..58:13]: "60.0" [plain @58:15..58:19]
          - mapping [block]
            "selector" [plain @59:5..59:13]: "google.appengine.v1.Firewall.*" [single-quoted @59:15..59:47]
            "deadline" [plain @60:5..60:13]: "30.0" [plain @60:15..60:19]
          - mapping [block]
            "selector" [plain @61:5..61:13]: "google.appengine.v1.Instances.*" [single-quoted @61:15..61:48]
            "deadline" [plain @62:5..62:13]: "30.0" [plain @62:15..62:19]
          - mapping [block]
            "selector" [plain @63:5..63:13]: "google.appengine.v1.Instances.ListInstances" [plain @63:15..63:58]
            "deadline" [plain @64:5..64:13]: "60.0" [plain @64:15..64:19]
          - mapping [block]
            "selector" [plain @65:5..65:13]: "google.appengine.v1.Services.*" [single-quoted @65:15..65:47]
            "deadline" [plain @66:5..66:13]: "30.0" [plain @66:15..66:19]
          - mapping [block]
            "selector" [plain @67:5..67:13]: "google.appengine.v1.Versions.*" [single-quoted @67:15..67:47]
            "deadline" [plain @68:5..68:13]: "30.0" [plain @68:15..68:19]
          - mapping [block]
            "selector" [plain @69:5..69:13]: "google.cloud.location.Locations.GetLocation" [plain @69:15..69:58]
            "deadline" [plain @70:5..70:13]: "30.0" [plain @70:15..70:19]
          - mapping [block]
            "selector" [plain @71:5..71:13]: "google.cloud.location.Locations.ListLocations" [plain @71:15..71:60]
            "deadline" [plain @72:5..72:13]: "30.0" [plain @72:15..72:19]
          - mapping [block]
            "selector" [plain @73:5..73:13]: "google.longrunning.Operations.*" [single-quoted @73:15..73:48]
            "deadline" [plain @74:5..74:13]: "30.0" [plain @74:15..74:19]
      "http" [plain @76:1..76:5]: mapping [block]
        "rules" [plain @77:3..77:8]: sequence [block]
          - mapping [block]
            "selector" [plain @78:5..78:13]: "google.cloud.location.Locations.GetLocation" [plain @78:15..78:58]
            "get" [plain @79:5..79:8]: "/v1/{name=apps/*/locations/*}" [single-quoted @79:10..79:41]
          - mapping [block]
            "selector" [plain @80:5..80:13]: "google.cloud.location.Locations.ListLocations" [plain @80:15..80:60]
            "get" [plain @81:5..81:8]: "/v1/{name=apps/*}/locations" [single-quoted @81:10..81:39]
          - mapping [block]
            "selector" [plain @82:5..82:13]: "google.longrunning.Operations.GetOperation" [plain @82:15..82:57]
            "get" [plain @83:5..83:8]: "/v1/{name=apps/*/operations/*}" [single-quoted @83:10..83:42]
          - mapping [block]
            "selector" [plain @84:5..84:13]: "google.longrunning.Operations.ListOperations" [plain @84:15..84:59]
            "get" [plain @85:5..85:8]: "/v1/{name=apps/*}/operations" [single-quoted @85:10..85:40]
      "authentication" [plain @87:1..87:15]: mapping [block]
        "rules" [plain @88:3..88:8]: sequence [block]
          - mapping [block]
            "selector" [plain @89:5..89:13]: "google.appengine.v1.Applications.*" [single-quoted @89:15..89:51]
            "oauth" [plain @90:5..90:10]: mapping [block]
              "canonical_scopes" [plain @91:7..91:23]: "https://www.googleapis.com/auth/cloud-platform" [literal @91:25..93:1]
          - mapping [block]
            "selector" [plain @93:5..93:13]: "google.appengine.v1.Applications.GetApplication" [plain @93:15..93:62]
            "oauth" [plain @94:5..94:10]: mapping [block]
              "canonical_scopes" [plain @95:7..95:23]: literal [@95:25..99:1]
                "https://www.googleapis.com/auth/appengine.admin,\n"
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/cloud-platform.read-only"
          - mapping [block]
            "selector" [plain @99:5..99:13]: "google.appengine.v1.AuthorizedCertificates.*" [single-quoted @99:15..99:61]
            "oauth" [plain @100:5..100:10]: mapping [block]
              "canonical_scopes" [plain @101:7..101:23]: "https://www.googleapis.com/auth/cloud-platform" [literal @101:25..103:1]
          - mapping [block]
            "selector" [plain @103:5..103:13]: "google.appengine.v1.AuthorizedCertificates.GetAuthorizedCertificate" [plain @103:15..103:82]
            "oauth" [plain @104:5..104:10]: mapping [block]
              "canonical_scopes" [plain @105:7..105:23]: literal [@105:25..109:1]
                "https://www.googleapis.com/auth/appengine.admin,\n"
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/cloud-platform.read-only"
          - mapping [block]
            "selector" [plain @109:5..109:13]: "google.appengine.v1.AuthorizedCertificates.ListAuthorizedCertificates" [plain @109:15..109:84]
            "oauth" [plain @110:5..110:10]: mapping [block]
              "canonical_scopes" [plain @111:7..111:23]: literal [@111:25..115:1]
                "https://www.googleapis.com/auth/appengine.admin,\n"
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/cloud-platform.read-only"
          - mapping [block]
            "selector" [plain @115:5..115:13]: "google.appengine.v1.AuthorizedDomains.ListAuthorizedDomains" [plain @115:15..115:74]
            "oauth" [plain @116:5..116:10]: mapping [block]
              "canonical_scopes" [plain @117:7..117:23]: literal [@117:25..121:1]
                "https://www.googleapis.com/auth/appengine.admin,\n"
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/cloud-platform.read-only"
          - mapping [block]
            "selector" [plain @121:5..121:13]: "google.appengine.v1.DomainMappings.*" [single-quoted @121:15..121:53]
            "oauth" [plain @122:5..122:10]: mapping [block]
              "canonical_scopes" [plain @123:7..123:23]: "https://www.googleapis.com/auth/cloud-platform" [literal @123:25..125:1]
          - mapping [block]
            "selector" [plain @125:5..125:13]: "google.appengine.v1.DomainMappings.GetDomainMapping" [plain @125:15..125:66]
            "oauth" [plain @126:5..126:10]: mapping [block]
              "canonical_scopes" [plain @127:7..127:23]: literal [@127:25..131:1]
                "https://www.googleapis.com/auth/appengine.admin,\n"
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/cloud-platform.read-only"
          - mapping [block]
            "selector" [plain @131:5..131:13]: "google.appengine.v1.DomainMappings.ListDomainMappings" [plain @131:15..131:68]
            "oauth" [plain @132:5..132:10]: mapping [block]
              "canonical_scopes" [plain @133:7..133:23]: literal [@133:25..137:1]
                "https://www.googleapis.com/auth/appengine.admin,\n"
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/cloud-platform.read-only"
          - mapping [block]
            "selector" [plain @137:5..137:13]: "google.appengine.v1.Firewall.*" [single-quoted @137:15..137:47]
            "oauth" [plain @138:5..138:10]: mapping [block]
              "canonical_scopes" [plain @139:7..139:23]: "https://www.googleapis.com/auth/cloud-platform" [literal @139:25..141:1]
          - mapping [block]
            "selector" [plain @141:5..141:13]: "google.appengine.v1.Firewall.GetIngressRule" [plain @141:15..141:58]
            "oauth" [plain @142:5..142:10]: mapping [block]
              "canonical_scopes" [plain @143:7..143:23]: literal [@143:25..147:1]
                "https://www.googleapis.com/auth/appengine.admin,\n"
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/cloud-platform.read-only"
          - mapping [block]
            "selector" [plain @147:5..147:13]: "google.appengine.v1.Firewall.ListIngressRules" [plain @147:15..147:60]
            "oauth" [plain @148:5..148:10]: mapping [block]
              "canonical_scopes" [plain @149:7..149:23]: literal [@149:25..153:1]
                "https://www.googleapis.com/auth/appengine.admin,\n"
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/cloud-platform.read-only"
          - mapping [block]
            "selector" [plain @153:5..153:13]: "google.appengine.v1.Instances.DebugInstance" [plain @153:15..153:58]
            "oauth" [plain @154:5..154:10]: mapping [block]
              "canonical_scopes" [plain @155:7..155:23]: "https://www.googleapis.com/auth/cloud-platform" [literal @155:25..157:1]
          - mapping [block]
            "selector" [plain @157:5..157:13]: "google.appengine.v1.Instances.DeleteInstance" [plain @157:15..157:59]
            "oauth" [plain @158:5..158:10]: mapping [block]
              "canonical_scopes" [plain @159:7..159:23]: "https://www.googleapis.com/auth/cloud-platform" [literal @159:25..161:1]
          - mapping [block]
            "selector" [plain @161:5..161:13]: "google.appengine.v1.Instances.GetInstance" [plain @161:15..161:56]
            "oauth" [plain @162:5..162:10]: mapping [block]
              "canonical_scopes" [plain @163:7..163:23]: literal [@163:25..167:1]
                "https://www.googleapis.com/auth/appengine.admin,\n"
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/cloud-platform.read-only"
          - mapping [block]
            "selector" [plain @167:5..167:13]: "google.appengine.v1.Instances.ListInstances" [plain @167:15..167:58]
            "oauth" [plain @168:5..168:10]: mapping [block]
              "canonical_scopes" [plain @169:7..169:23]: literal [@169:25..173:1]
                "https://www.googleapis.com/auth/appengine.admin,\n"
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/cloud-platform.read-only"
          - mapping [block]
            "selector" [plain @173:5..173:13]: "google.appengine.v1.Services.DeleteService" [plain @173:15..173:57]
            "oauth" [plain @174:5..174:10]: mapping [block]
              "canonical_scopes" [plain @175:7..175:23]: "https://www.googleapis.com/auth/cloud-platform" [literal @175:25..177:1]
          - mapping [block]
            "selector" [plain @177:5..177:13]: "google.appengine.v1.Services.GetService" [plain @177:15..177:54]
            "oauth" [plain @178:5..178:10]: mapping [block]
              "canonical_scopes" [plain @179:7..179:23]: literal [@179:25..183:1]
                "https://www.googleapis.com/auth/appengine.admin,\n"
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/cloud-platform.read-only"
          - mapping [block]
            "selector" [plain @183:5..183:13]: "google.appengine.v1.Services.ListServices" [plain @183:15..183:56]
            "oauth" [plain @184:5..184:10]: mapping [block]
              "canonical_scopes" [plain @185:7..185:23]: literal [@185:25..189:1]
                "https://www.googleapis.com/auth/appengine.admin,\n"
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/cloud-platform.read-only"
          - mapping [block]
            "selector" [plain @189:5..189:13]: "google.appengine.v1.Services.UpdateService" [plain @189:15..189:57]
            "oauth" [plain @190:5..190:10]: mapping [block]
              "canonical_scopes" [plain @191:7..191:23]: "https://www.googleapis.com/auth/cloud-platform" [literal @191:25..193:1]
          - mapping [block]
            "selector" [plain @193:5..193:13]: "google.appengine.v1.Versions.*" [single-quoted @193:15..193:47]
            "oauth" [plain @194:5..194:10]: mapping [block]
              "canonical_scopes" [plain @195:7..195:23]: "https://www.googleapis.com/auth/cloud-platform" [literal @195:25..197:1]
          - mapping [block]
            "selector" [plain @197:5..197:13]: "google.appengine.v1.Versions.GetVersion" [plain @197:15..197:54]
            "oauth" [plain @198:5..198:10]: mapping [block]
              "canonical_scopes" [plain @199:7..199:23]: literal [@199:25..203:1]
                "https://www.googleapis.com/auth/appengine.admin,\n"
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/cloud-platform.read-only"
          - mapping [block]
            "selector" [plain @203:5..203:13]: "google.appengine.v1.Versions.ListVersions" [plain @203:15..203:56]
            "oauth" [plain @204:5..204:10]: mapping [block]
              "canonical_scopes" [plain @205:7..205:23]: literal [@205:25..209:1]
                "https://www.googleapis.com/auth/appengine.admin,\n"
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/cloud-platform.read-only"
          - mapping [block]
            "selector" [plain @209:5..209:13]: "google.cloud.location.Locations.GetLocation" [plain @209:15..209:58]
            "oauth" [plain @210:5..210:10]: mapping [block]
              "canonical_scopes" [plain @211:7..211:23]: literal [@211:25..215:1]
                "https://www.googleapis.com/auth/appengine.admin,\n"
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/cloud-platform.read-only"
          - mapping [block]
            "selector" [plain @215:5..215:13]: "google.cloud.location.Locations.ListLocations" [plain @215:15..215:60]
            "oauth" [plain @216:5..216:10]: mapping [block]
              "canonical_scopes" [plain @217:7..217:23]: literal [@217:25..221:1]
                "https://www.googleapis.com/auth/appengine.admin,\n"
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/cloud-platform.read-only"
          - mapping [block]
            "selector" [plain @221:5..221:13]: "google.longrunning.Operations.GetOperation" [plain @221:15..221:57]
            "oauth" [plain @222:5..222:10]: mapping [block]
              "canonical_scopes" [plain @223:7..223:23]: literal [@223:25..227:1]
                "https://www.googleapis.com/auth/appengine.admin,\n"
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/cloud-platform.read-only"
          - mapping [block]
            "selector" [plain @227:5..227:13]: "google.longrunning.Operations.ListOperations" [plain @227:15..227:59]
            "oauth" [plain @228:5..228:10]: mapping [block]
              "canonical_scopes" [plain @229:7..229:23]: literal [@229:25..233:1]
                "https://www.googleapis.com/auth/appengine.admin,\n"
                "https://www.googleapis.com/auth/cloud-platform,\n"
                "https://www.googleapis.com/auth/cloud-platform.read-only"
          - mapping [block]
            "selector" [plain @233:5..233:13]: "google.longrunning.Operations.WaitOperation" [plain @233:15..233:58]
            "oauth" [plain @234:5..234:10]: mapping [block]
              "canonical_scopes" [plain @235:7..235:23]: "https://www.googleapis.com/auth/cloud-platform" [literal @235:25..237:1]
    "###);
}
