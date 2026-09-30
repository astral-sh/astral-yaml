#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("istio-istio-samples-addons-jaeger-31b4431b"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "apps/v1" [plain @1:13..1:20]
      "kind" [plain @2:1..2:5]: "Deployment" [plain @2:7..2:17]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "name" [plain @4:3..4:7]: "jaeger" [plain @4:9..4:15]
        "namespace" [plain @5:3..5:12]: "istio-system" [plain @5:14..5:26]
        "labels" [plain @6:3..6:9]: mapping [block]
          "app" [plain @7:5..7:8]: "jaeger" [plain @7:10..7:16]
      "spec" [plain @8:1..8:5]: mapping [block]
        "selector" [plain @9:3..9:11]: mapping [block]
          "matchLabels" [plain @10:5..10:16]: mapping [block]
            "app" [plain @11:7..11:10]: "jaeger" [plain @11:12..11:18]
        "template" [plain @12:3..12:11]: mapping [block]
          "metadata" [plain @13:5..13:13]: mapping [block]
            "labels" [plain @14:7..14:13]: mapping [block]
              "app" [plain @15:9..15:12]: "jaeger" [plain @15:14..15:20]
              "sidecar.istio.io/inject" [plain @16:9..16:32]: "false" [double-quoted @16:34..16:41]
            "annotations" [plain @17:7..17:18]: mapping [block]
              "prometheus.io/scrape" [plain @18:9..18:29]: "true" [double-quoted @18:31..18:37]
              "prometheus.io/port" [plain @19:9..19:27]: "8888" [double-quoted @19:29..19:35]
          "spec" [plain @20:5..20:9]: mapping [block]
            "containers" [plain @21:7..21:17]: sequence [block]
              - mapping [block]
                "name" [plain @22:11..22:15]: "jaeger" [plain @22:17..22:23]
                "image" [plain @23:11..23:16]: "docker.io/jaegertracing/jaeger:2.17.0" [double-quoted @23:18..23:57]
                "args" [plain @24:11..24:15]: sequence [block]
                  - "--config" [double-quoted @25:15..25:25]
                  - "/jaeger-configuration/config.yaml" [double-quoted @26:15..26:50]
                "livenessProbe" [plain @27:11..27:24]: mapping [block]
                  "httpGet" [plain @28:13..28:20]: mapping [block]
                    "path" [plain @29:15..29:19]: "/status" [plain @29:21..29:28]
                    "port" [plain @30:15..30:19]: "13133" [plain @30:21..30:26]
                "readinessProbe" [plain @31:11..31:25]: mapping [block]
                  "httpGet" [plain @32:13..32:20]: mapping [block]
                    "path" [plain @33:15..33:19]: "/status" [plain @33:21..33:28]
                    "port" [plain @34:15..34:19]: "13133" [plain @34:21..34:26]
                "volumeMounts" [plain @35:11..35:23]: sequence [block]
                  - mapping [block]
                    "name" [plain @36:15..36:19]: "data" [plain @36:21..36:25]
                    "mountPath" [plain @37:15..37:24]: "/badger" [plain @37:26..37:33]
                  - mapping [block]
                    "name" [plain @38:15..38:19]: "jaeger-configuration" [plain @38:21..38:41]
                    "mountPath" [plain @39:15..39:24]: "/jaeger-configuration" [double-quoted @39:26..39:49]
                "resources" [plain @40:11..40:20]: mapping [block]
                  "requests" [plain @41:13..41:21]: mapping [block]
                    "cpu" [plain @42:15..42:18]: "10m" [plain @42:20..42:23]
            "volumes" [plain @43:7..43:14]: sequence [block]
              - mapping [block]
                "name" [plain @44:11..44:15]: "data" [plain @44:17..44:21]
                "emptyDir" [plain @45:11..45:19]: mapping [flow]
              - mapping [block]
                "name" [plain @46:11..46:15]: "jaeger-configuration" [plain @46:17..46:37]
                "configMap" [plain @47:11..47:20]: mapping [block]
                  "name" [plain @48:13..48:17]: "jaeger" [plain @48:19..48:25]

    document 2 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @50:1..50:11]: "v1" [plain @50:13..50:15]
      "kind" [plain @51:1..51:5]: "Service" [plain @51:7..51:14]
      "metadata" [plain @52:1..52:9]: mapping [block]
        "name" [plain @53:3..53:7]: "tracing" [plain @53:9..53:16]
        "namespace" [plain @54:3..54:12]: "istio-system" [plain @54:14..54:26]
        "labels" [plain @55:3..55:9]: mapping [block]
          "app" [plain @56:5..56:8]: "jaeger" [plain @56:10..56:16]
      "spec" [plain @57:1..57:5]: mapping [block]
        "type" [plain @58:3..58:7]: "ClusterIP" [plain @58:9..58:18]
        "ports" [plain @59:3..59:8]: sequence [block]
          - mapping [block]
            "name" [plain @60:7..60:11]: "http-query" [plain @60:13..60:23]
            "port" [plain @61:7..61:11]: "80" [plain @61:13..61:15]
            "protocol" [plain @62:7..62:15]: "TCP" [plain @62:17..62:20]
            "targetPort" [plain @63:7..63:17]: "16686" [plain @63:19..63:24]
          - mapping [block]
            "name" [plain @65:7..65:11]: "grpc-query" [plain @65:13..65:23]
            "port" [plain @66:7..66:11]: "16685" [plain @66:13..66:18]
            "protocol" [plain @67:7..67:15]: "TCP" [plain @67:17..67:20]
            "targetPort" [plain @68:7..68:17]: "16685" [plain @68:19..68:24]
        "selector" [plain @69:3..69:11]: mapping [block]
          "app" [plain @70:5..70:8]: "jaeger" [plain @70:10..70:16]

    document 3 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @73:1..73:11]: "v1" [plain @73:13..73:15]
      "kind" [plain @74:1..74:5]: "ConfigMap" [plain @74:7..74:16]
      "metadata" [plain @75:1..75:9]: mapping [block]
        "name" [plain @76:3..76:7]: "jaeger" [plain @76:9..76:15]
        "namespace" [plain @77:3..77:12]: "istio-system" [plain @77:14..77:26]
        "labels" [plain @78:3..78:9]: mapping [block]
          "app" [plain @79:5..79:8]: "jaeger" [plain @79:10..79:16]
      "data" [plain @80:1..80:5]: mapping [block]
        "config.yaml" [plain @84:3..84:14]: literal [@84:16..179:1]
          "service:\n"
          "  extensions: [jaeger_storage, jaeger_query, remote_sampling, healthcheckv2, expvar, zpages]\n"
          "  pipelines:\n"
          "    traces:\n"
          "      receivers: [otlp, jaeger, zipkin]\n"
          "      processors: [batch]\n"
          "      exporters: [jaeger_storage_exporter]\n"
          "  telemetry:\n"
          "    resource:\n"
          "      service.name: jaeger\n"
          "    metrics:\n"
          "      level: detailed\n"
          "      readers:\n"
          "        - pull:\n"
          "            exporter:\n"
          "              prometheus:\n"
          "                host: \"0.0.0.0\"\n"
          "                port: 8888\n"
          "    logs:\n"
          "      level: info\n"
          "    # TODO Initialize telemetry tracer once OTEL released new feature.\n"
          "    # https://github.com/open-telemetry/opentelemetry-collector/issues/10663\n"
          "\n"
          "extensions:\n"
          "  jaeger_query:\n"
          "    base_path: /jaeger\n"
          "    storage:\n"
          "      traces: badger_storage\n"
          "\n"
          "  jaeger_storage:\n"
          "    backends:\n"
          "      badger_storage:\n"
          "        badger:\n"
          "          ephemeral: false\n"
          "          directories:\n"
          "            keys: \"/badger/key\"\n"
          "            values: \"/badger/data/\"\n"
          "\n"
          "  remote_sampling:\n"
          "    # We can either use file or adaptive sampling strategy in remote_sampling\n"
          "    file:\n"
          "      path:\n"
          "      default_sampling_probability: 1\n"
          "      reload_interval: 1s\n"
          "    # adaptive:\n"
          "    #   sampling_store: some_store\n"
          "    #   initial_sampling_probability: 0.1\n"
          "    http:\n"
          "      endpoint: \"0.0.0.0:5778\"\n"
          "    grpc:\n"
          "      endpoint: \"0.0.0.0:5779\"\n"
          "\n"
          "  healthcheckv2:\n"
          "    use_v2: true\n"
          "    http:\n"
          "      endpoint: \"0.0.0.0:13133\"\n"
          "    grpc:\n"
          "\n"
          "  expvar:\n"
          "    endpoint: \"0.0.0.0:27777\"\n"
          "\n"
          "  zpages:\n"
          "    # for some reason the official extension listens on ephemeral port 55679\n"
          "    # so we override it with a normal port\n"
          "    endpoint: \"0.0.0.0:27778\"\n"
          "\n"
          "receivers:\n"
          "  otlp:\n"
          "    protocols:\n"
          "      grpc:\n"
          "        endpoint: \"0.0.0.0:4317\"\n"
          "      http:\n"
          "        endpoint: \"0.0.0.0:4318\"\n"
          "\n"
          "  jaeger:\n"
          "    protocols:\n"
          "      grpc:\n"
          "        endpoint: \"0.0.0.0:14250\"\n"
          "      thrift_http:\n"
          "        endpoint: \"0.0.0.0:14268\"\n"
          "      thrift_binary:\n"
          "        endpoint: \"0.0.0.0:6832\"\n"
          "      thrift_compact:\n"
          "        endpoint: \"0.0.0.0:6831\"\n"
          "\n"
          "  zipkin:\n"
          "    endpoint: \"0.0.0.0:9411\"\n"
          "\n"
          "processors:\n"
          "  batch:\n"
          "\n"
          "exporters:\n"
          "  jaeger_storage_exporter:\n"
          "    trace_storage: badger_storage\n"

    document 4 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @181:1..181:11]: "v1" [plain @181:13..181:15]
      "kind" [plain @182:1..182:5]: "Service" [plain @182:7..182:14]
      "metadata" [plain @183:1..183:9]: mapping [block]
        "labels" [plain @184:3..184:9]: mapping [block]
          "name" [plain @185:5..185:9]: "zipkin" [plain @185:11..185:17]
        "name" [plain @186:3..186:7]: "zipkin" [plain @186:9..186:15]
        "namespace" [plain @187:3..187:12]: "istio-system" [plain @187:14..187:26]
      "spec" [plain @188:1..188:5]: mapping [block]
        "ports" [plain @189:3..189:8]: sequence [block]
          - mapping [block]
            "port" [plain @190:7..190:11]: "9411" [plain @190:13..190:17]
            "targetPort" [plain @191:7..191:17]: "9411" [plain @191:19..191:23]
            "name" [plain @192:7..192:11]: "http-query" [plain @192:13..192:23]
        "selector" [plain @193:3..193:11]: mapping [block]
          "app" [plain @194:5..194:8]: "jaeger" [plain @194:10..194:16]

    document 5 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @196:1..196:11]: "v1" [plain @196:13..196:15]
      "kind" [plain @197:1..197:5]: "Service" [plain @197:7..197:14]
      "metadata" [plain @198:1..198:9]: mapping [block]
        "name" [plain @199:3..199:7]: "jaeger-collector" [plain @199:9..199:25]
        "namespace" [plain @200:3..200:12]: "istio-system" [plain @200:14..200:26]
        "labels" [plain @201:3..201:9]: mapping [block]
          "app" [plain @202:5..202:8]: "jaeger" [plain @202:10..202:16]
      "spec" [plain @203:1..203:5]: mapping [block]
        "type" [plain @204:3..204:7]: "ClusterIP" [plain @204:9..204:18]
        "ports" [plain @205:3..205:8]: sequence [block]
          - mapping [block]
            "name" [plain @206:5..206:9]: "jaeger-collector-http" [plain @206:11..206:32]
            "port" [plain @207:5..207:9]: "14268" [plain @207:11..207:16]
            "targetPort" [plain @208:5..208:15]: "14268" [plain @208:17..208:22]
            "protocol" [plain @209:5..209:13]: "TCP" [plain @209:15..209:18]
          - mapping [block]
            "name" [plain @210:5..210:9]: "jaeger-collector-grpc" [plain @210:11..210:32]
            "port" [plain @211:5..211:9]: "14250" [plain @211:11..211:16]
            "targetPort" [plain @212:5..212:15]: "14250" [plain @212:17..212:22]
            "protocol" [plain @213:5..213:13]: "TCP" [plain @213:15..213:18]
          - mapping [block]
            "port" [plain @214:5..214:9]: "9411" [plain @214:11..214:15]
            "targetPort" [plain @215:5..215:15]: "9411" [plain @215:17..215:21]
            "name" [plain @216:5..216:9]: "http-zipkin" [plain @216:11..216:22]
          - mapping [block]
            "port" [plain @217:5..217:9]: "4317" [plain @217:11..217:15]
            "name" [plain @218:5..218:9]: "grpc-otel" [plain @218:11..218:20]
          - mapping [block]
            "port" [plain @219:5..219:9]: "4318" [plain @219:11..219:15]
            "name" [plain @220:5..220:9]: "http-otel" [plain @220:11..220:20]
        "selector" [plain @221:3..221:11]: mapping [block]
          "app" [plain @222:5..222:8]: "jaeger" [plain @222:10..222:16]
    "#);
}
