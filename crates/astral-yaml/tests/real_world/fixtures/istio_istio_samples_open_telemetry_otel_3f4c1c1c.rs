#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("istio-istio-samples-open-telemetry-otel-3f4c1c1c"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "v1" [plain @1:13..1:15]
      "kind" [plain @2:1..2:5]: "ConfigMap" [plain @2:7..2:16]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "opentelemetry-collector-conf" [plain @4:9..4:37]
        "labels" [plain @5:3..5:9]: mapping [block]
          "app" [plain @6:5..6:8]: "opentelemetry-collector" [plain @6:10..6:33]
      "data" [plain @7:1..7:5]: mapping [block]
        "opentelemetry-collector-config" [plain @8:3..8:33]: literal [@8:35..62:1]
          "receivers:\n"
          "  opencensus:\n"
          "    endpoint: 0.0.0.0:55678\n"
          "  otlp/auth:\n"
          "    protocols:\n"
          "      grpc:\n"
          "        endpoint: 0.0.0.0:5317\n"
          "        auth:\n"
          "          authenticator: bearertokenauth\n"
          "  otlp:\n"
          "    protocols:\n"
          "      grpc:\n"
          "        endpoint: 0.0.0.0:4317\n"
          "      http:\n"
          "        endpoint: 0.0.0.0:4318\n"
          "processors:\n"
          "  batch:\n"
          "exporters:\n"
          "  zipkin:\n"
          "    # Export to zipkin for easy querying\n"
          "    endpoint: http://zipkin.istio-system.svc:9411/api/v2/spans\n"
          "  debug:\n"
          "    verbosity: detailed\n"
          "extensions:\n"
          "  bearertokenauth:\n"
          "    token: \"somerandomtoken\"\n"
          "  health_check:\n"
          "  pprof:\n"
          "  zpages:\n"
          "service:\n"
          "  extensions:\n"
          "  - health_check\n"
          "  - pprof\n"
          "  - zpages\n"
          "  - bearertokenauth\n"
          "  pipelines:\n"
          "    logs:\n"
          "      receivers: [otlp]\n"
          "      processors: [batch]\n"
          "      exporters: [debug]\n"
          "    traces:\n"
          "      receivers:\n"
          "      - otlp\n"
          "      - opencensus\n"
          "      exporters:\n"
          "      - zipkin\n"
          "      - debug\n"
          "    traces/auth:\n"
          "      receivers:\n"
          "      - otlp/auth\n"
          "      exporters:\n"
          "      - zipkin\n"
          "      - debug\n"

    document 2 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @63:1..63:11]: "v1" [plain @63:13..63:15]
      "kind" [plain @64:1..64:5]: "Service" [plain @64:7..64:14]
      "metadata" [plain @65:1..65:9]: mapping [block]
        "name" [plain @66:3..66:7]: "opentelemetry-collector" [plain @66:9..66:32]
        "labels" [plain @67:3..67:9]: mapping [block]
          "app" [plain @68:5..68:8]: "opentelemetry-collector" [plain @68:10..68:33]
      "spec" [plain @69:1..69:5]: mapping [block]
        "ports" [plain @70:3..70:8]: sequence [block]
          - mapping [block]
            "name" [plain @71:7..71:11]: "grpc-opencensus" [plain @71:13..71:28]
            "port" [plain @72:7..72:11]: "55678" [plain @72:13..72:18]
            "protocol" [plain @73:7..73:15]: "TCP" [plain @73:17..73:20]
            "targetPort" [plain @74:7..74:17]: "55678" [plain @74:19..74:24]
          - mapping [block]
            "name" [plain @75:7..75:11]: "grpc-otlp" [plain @75:13..75:22]
            "port" [plain @76:7..76:11]: "4317" [plain @76:13..76:17]
            "protocol" [plain @77:7..77:15]: "TCP" [plain @77:17..77:20]
            "targetPort" [plain @78:7..78:17]: "4317" [plain @78:19..78:23]
          - mapping [block]
            "name" [plain @79:7..79:11]: "http-otlp" [plain @79:13..79:22]
            "port" [plain @80:7..80:11]: "4318" [plain @80:13..80:17]
            "protocol" [plain @81:7..81:15]: "TCP" [plain @81:17..81:20]
            "targetPort" [plain @82:7..82:17]: "4318" [plain @82:19..82:23]
          - mapping [block]
            "name" [plain @83:7..83:11]: "grpc-otlp-auth" [plain @83:13..83:27]
            "port" [plain @84:7..84:11]: "5317" [plain @84:13..84:17]
            "protocol" [plain @85:7..85:15]: "TCP" [plain @85:17..85:20]
            "targetPort" [plain @86:7..86:17]: "5317" [plain @86:19..86:23]
        "selector" [plain @87:3..87:11]: mapping [block]
          "app" [plain @88:5..88:8]: "opentelemetry-collector" [plain @88:10..88:33]

    document 3 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @90:1..90:11]: "apps/v1" [plain @90:13..90:20]
      "kind" [plain @91:1..91:5]: "Deployment" [plain @91:7..91:17]
      "metadata" [plain @92:1..92:9]: mapping [block]
        "name" [plain @93:3..93:7]: "opentelemetry-collector" [plain @93:9..93:32]
      "spec" [plain @94:1..94:5]: mapping [block]
        "selector" [plain @95:3..95:11]: mapping [block]
          "matchLabels" [plain @96:5..96:16]: mapping [block]
            "app" [plain @97:7..97:10]: "opentelemetry-collector" [plain @97:12..97:35]
        "strategy" [plain @98:3..98:11]: mapping [block]
          "rollingUpdate" [plain @99:5..99:18]: mapping [block]
            "maxSurge" [plain @100:7..100:15]: "1" [plain @100:17..100:18]
            "maxUnavailable" [plain @101:7..101:21]: "1" [plain @101:23..101:24]
          "type" [plain @102:5..102:9]: "RollingUpdate" [plain @102:11..102:24]
        "template" [plain @103:3..103:11]: mapping [block]
          "metadata" [plain @104:5..104:13]: mapping [block]
            "labels" [plain @105:7..105:13]: mapping [block]
              "app" [plain @106:9..106:12]: "opentelemetry-collector" [plain @106:14..106:37]
              "sidecar.istio.io/inject" [plain @107:9..107:32]: "false" [double-quoted @107:34..107:41]
          "spec" [plain @108:5..108:9]: mapping [block]
            "containers" [plain @109:7..109:17]: sequence [block]
              - mapping [block]
                "args" [plain @110:11..110:15]: sequence [block]
                  - "--config=/conf/opentelemetry-collector-config.yaml" [double-quoted @111:15..111:67]
                "env" [plain @112:11..112:14]: sequence [block]
                  - mapping [block]
                    "name" [plain @113:15..113:19]: "POD_NAME" [plain @113:21..113:29]
                    "valueFrom" [plain @114:15..114:24]: mapping [block]
                      "fieldRef" [plain @115:17..115:25]: mapping [block]
                        "apiVersion" [plain @116:19..116:29]: "v1" [plain @116:31..116:33]
                        "fieldPath" [plain @117:19..117:28]: "metadata.name" [plain @117:30..117:43]
                  - mapping [block]
                    "name" [plain @118:15..118:19]: "POD_NAMESPACE" [plain @118:21..118:34]
                    "valueFrom" [plain @119:15..119:24]: mapping [block]
                      "fieldRef" [plain @120:17..120:25]: mapping [block]
                        "apiVersion" [plain @121:19..121:29]: "v1" [plain @121:31..121:33]
                        "fieldPath" [plain @122:19..122:28]: "metadata.namespace" [plain @122:30..122:48]
                "image" [plain @123:11..123:16]: "docker.io/otel/opentelemetry-collector-contrib:0.123.0" [plain @123:18..123:72]
                "imagePullPolicy" [plain @124:11..124:26]: "IfNotPresent" [plain @124:28..124:40]
                "name" [plain @125:11..125:15]: "opentelemetry-collector" [plain @125:17..125:40]
                "ports" [plain @126:11..126:16]: sequence [block]
                  - mapping [block]
                    "containerPort" [plain @127:15..127:28]: "4317" [plain @127:30..127:34]
                    "protocol" [plain @128:15..128:23]: "TCP" [plain @128:25..128:28]
                  - mapping [block]
                    "containerPort" [plain @129:15..129:28]: "4318" [plain @129:30..129:34]
                    "protocol" [plain @130:15..130:23]: "TCP" [plain @130:25..130:28]
                  - mapping [block]
                    "name" [plain @131:15..131:19]: "grpc-opencensus" [plain @131:21..131:36]
                    "containerPort" [plain @132:15..132:28]: "55678" [plain @132:30..132:35]
                    "protocol" [plain @133:15..133:23]: "TCP" [plain @133:25..133:28]
                "resources" [plain @134:11..134:20]: mapping [block]
                  "limits" [plain @135:13..135:19]: mapping [block]
                    "cpu" [plain @136:15..136:18]: "2" [double-quoted @136:20..136:23]
                    "memory" [plain @137:15..137:21]: "4Gi" [plain @137:23..137:26]
                  "requests" [plain @138:13..138:21]: mapping [block]
                    "cpu" [plain @139:15..139:18]: "200m" [plain @139:20..139:24]
                    "memory" [plain @140:15..140:21]: "400Mi" [plain @140:23..140:28]
                "terminationMessagePath" [plain @141:11..141:33]: "/dev/termination-log" [plain @141:35..141:55]
                "terminationMessagePolicy" [plain @142:11..142:35]: "File" [plain @142:37..142:41]
                "volumeMounts" [plain @143:11..143:23]: sequence [block]
                  - mapping [block]
                    "name" [plain @144:15..144:19]: "opentelemetry-collector-config-vol" [plain @144:21..144:55]
                    "mountPath" [plain @145:15..145:24]: "/conf" [plain @145:26..145:31]
            "dnsPolicy" [plain @146:7..146:16]: "ClusterFirst" [plain @146:18..146:30]
            "restartPolicy" [plain @147:7..147:20]: "Always" [plain @147:22..147:28]
            "schedulerName" [plain @148:7..148:20]: "default-scheduler" [plain @148:22..148:39]
            "terminationGracePeriodSeconds" [plain @149:7..149:36]: "30" [plain @149:38..149:40]
            "volumes" [plain @150:7..150:14]: sequence [block]
              - mapping [block]
                "configMap" [plain @151:11..151:20]: mapping [block]
                  "defaultMode" [plain @152:13..152:24]: "420" [plain @152:26..152:29]
                  "items" [plain @153:13..153:18]: sequence [block]
                    - mapping [block]
                      "key" [plain @154:17..154:20]: "opentelemetry-collector-config" [plain @154:22..154:52]
                      "path" [plain @155:17..155:21]: "opentelemetry-collector-config.yaml" [plain @155:23..155:58]
                  "name" [plain @156:13..156:17]: "opentelemetry-collector-conf" [plain @156:19..156:47]
                "name" [plain @157:11..157:15]: "opentelemetry-collector-config-vol" [plain @157:17..157:51]
    "#);
}
