#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("open-telemetry-opentelemetry-collector-contrib-examples-kubernetes-otel-collector-47e8be13"), @r#"
    document 1 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @2:1..2:11]: "v1" [plain @2:13..2:15]
      "kind" [plain @3:1..3:5]: "ConfigMap" [plain @3:7..3:16]
      "metadata" [plain @4:1..4:9]: mapping [block]
        "name" [plain @5:3..5:7]: "otel-collector-config" [plain @5:9..5:30]
      "data" [plain @6:1..6:5]: mapping [block]
        "config.yaml" [plain @7:3..7:14]: literal [@7:16..125:1]
          "receivers:\n"
          "  file_log:\n"
          "    include:\n"
          "      - /var/log/pods/*/*/*.log\n"
          "    exclude:\n"
          "      # Exclude logs from all containers named otel-collector\n"
          "      - /var/log/pods/*/otel-collector/*.log\n"
          "    start_at: end\n"
          "    include_file_path: true\n"
          "    include_file_name: false\n"
          "    operators:\n"
          "      # Find out which format is used by kubernetes\n"
          "      - type: router\n"
          "        id: get-format\n"
          "        routes:\n"
          "          - output: parser-docker\n"
          "            expr: 'body matches \"^\\\\{\"'\n"
          "          - output: parser-crio\n"
          "            expr: 'body matches \"^[^ Z]+ \"'\n"
          "          - output: parser-containerd\n"
          "            expr: 'body matches \"^[^ Z]+Z\"'\n"
          "      # Parse CRI-O format\n"
          "      - type: regex_parser\n"
          "        id: parser-crio\n"
          "        regex: '^(?P<time>[^ Z]+) (?P<stream>stdout|stderr) (?P<logtag>[^ ]*) ?(?P<log>.*)$'\n"
          "        output: extract_metadata_from_filepath\n"
          "        timestamp:\n"
          "          parse_from: attributes.time\n"
          "          layout_type: gotime\n"
          "          layout: '2006-01-02T15:04:05.999999999Z07:00'\n"
          "      # Parse CRI-Containerd format\n"
          "      - type: regex_parser\n"
          "        id: parser-containerd\n"
          "        regex: '^(?P<time>[^ ^Z]+Z) (?P<stream>stdout|stderr) (?P<logtag>[^ ]*) ?(?P<log>.*)$'\n"
          "        output: extract_metadata_from_filepath\n"
          "        timestamp:\n"
          "          parse_from: attributes.time\n"
          "          layout: '%Y-%m-%dT%H:%M:%S.%LZ'\n"
          "      # Parse Docker format\n"
          "      - type: json_parser\n"
          "        id: parser-docker\n"
          "        output: extract_metadata_from_filepath\n"
          "        timestamp:\n"
          "          parse_from: attributes.time\n"
          "          layout: '%Y-%m-%dT%H:%M:%S.%LZ'\n"
          "      # Extract metadata from file path\n"
          "      - type: regex_parser\n"
          "        id: extract_metadata_from_filepath\n"
          "        regex: '^.*\\/(?P<namespace>[^_]+)_(?P<pod_name>[^_]+)_(?P<uid>[a-f0-9\\-]{36})\\/(?P<container_name>[^\\._]+)\\/(?P<restart_count>\\d+)\\.log$'\n"
          "        parse_from: attributes[\"log.file.path\"]\n"
          "        cache:\n"
          "          size: 128  # default maximum amount of Pods per Node is 110\n"
          "      # Update body field after finishing all parsing\n"
          "      - type: move\n"
          "        from: attributes.log\n"
          "        to: body\n"
          "      # Rename attributes\n"
          "      - type: move\n"
          "        from: attributes.stream\n"
          "        to: attributes[\"log.iostream\"]\n"
          "      - type: move\n"
          "        from: attributes.container_name\n"
          "        to: resource[\"k8s.container.name\"]\n"
          "      - type: move\n"
          "        from: attributes.namespace\n"
          "        to: resource[\"k8s.namespace.name\"]\n"
          "      - type: move\n"
          "        from: attributes.pod_name\n"
          "        to: resource[\"k8s.pod.name\"]\n"
          "      - type: move\n"
          "        from: attributes.restart_count\n"
          "        to: resource[\"k8s.container.restart_count\"]\n"
          "      - type: move\n"
          "        from: attributes.uid\n"
          "        to: resource[\"k8s.pod.uid\"]\n"
          "\n"
          "processors:\n"
          "  # k8sattributes processor to get the metadata from K8s\n"
          "  k8sattributes:\n"
          "    auth_type: \"serviceAccount\"\n"
          "    passthrough: false\n"
          "    extract:\n"
          "      metadata:\n"
          "        - k8s.pod.name\n"
          "        - k8s.pod.uid\n"
          "        - k8s.deployment.name\n"
          "        - k8s.namespace.name\n"
          "        - k8s.node.name\n"
          "        - k8s.pod.start_time\n"
          "        - k8s.cluster.uid\n"
          "      # Pod labels which can be fetched via K8sattributeprocessor\n"
          "      labels:\n"
          "        - tag_name: key1\n"
          "          key: label1\n"
          "          from: pod\n"
          "        - tag_name: key2\n"
          "          key: label2\n"
          "          from: pod\n"
          "    # Pod association using resource attributes and connection\n"
          "    pod_association:\n"
          "      - sources:\n"
          "          - from: resource_attribute\n"
          "            name: k8s.pod.uid\n"
          "          - from: resource_attribute\n"
          "            name: k8s.pod.ip\n"
          "          - from: connection\n"
          "\n"
          "exporters:\n"
          "  debug:\n"
          "    verbosity: detailed\n"
          "service:\n"
          "  pipelines:\n"
          "    logs:\n"
          "      receivers: [file_log]\n"
          "      processors: [k8sattributes]\n"
          "      exporters: [debug]"

    document 2 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @126:1..126:11]: "apps/v1" [plain @126:13..126:20]
      "kind" [plain @127:1..127:5]: "DaemonSet" [plain @127:7..127:16]
      "metadata" [plain @128:1..128:9]: mapping [block]
        "name" [plain @129:3..129:7]: "otel-collector" [plain @129:9..129:23]
        "labels" [plain @130:3..130:9]: mapping [block]
          "app" [plain @131:5..131:8]: "opentelemetry" [plain @131:10..131:23]
          "component" [plain @132:5..132:14]: "otel-collector" [plain @132:16..132:30]
      "spec" [plain @133:1..133:5]: mapping [block]
        "selector" [plain @134:3..134:11]: mapping [block]
          "matchLabels" [plain @135:5..135:16]: mapping [block]
            "app" [plain @136:7..136:10]: "opentelemetry" [plain @136:12..136:25]
            "component" [plain @137:7..137:16]: "otel-collector" [plain @137:18..137:32]
        "template" [plain @138:3..138:11]: mapping [block]
          "metadata" [plain @139:5..139:13]: mapping [block]
            "labels" [plain @140:7..140:13]: mapping [block]
              "app" [plain @141:9..141:12]: "opentelemetry" [plain @141:14..141:27]
              "component" [plain @142:9..142:18]: "otel-collector" [plain @142:20..142:34]
          "spec" [plain @143:5..143:9]: mapping [block]
            "containers" [plain @144:7..144:17]: sequence [block]
              - mapping [block]
                "name" [plain @145:9..145:13]: "otel-collector" [plain @145:15..145:29]
                "image" [plain @146:9..146:14]: "otel/opentelemetry-collector-contrib:0.51.0" [plain @146:16..146:59]
                "resources" [plain @147:9..147:18]: mapping [block]
                  "limits" [plain @148:11..148:17]: mapping [block]
                    "cpu" [plain @149:13..149:16]: "100m" [plain @149:18..149:22]
                    "memory" [plain @150:13..150:19]: "200Mi" [plain @150:21..150:26]
                  "requests" [plain @151:11..151:19]: mapping [block]
                    "cpu" [plain @152:13..152:16]: "100m" [plain @152:18..152:22]
                    "memory" [plain @153:13..153:19]: "200Mi" [plain @153:21..153:26]
                "volumeMounts" [plain @154:9..154:21]: sequence [block]
                  - mapping [block]
                    "mountPath" [plain @155:11..155:20]: "/var/log" [plain @155:22..155:30]
                    "name" [plain @156:11..156:15]: "varlog" [plain @156:17..156:23]
                    "readOnly" [plain @157:11..157:19]: "true" [plain @157:21..157:25]
                  - mapping [block]
                    "mountPath" [plain @158:11..158:20]: "/var/lib/docker/containers" [plain @158:22..158:48]
                    "name" [plain @159:11..159:15]: "varlibdockercontainers" [plain @159:17..159:39]
                    "readOnly" [plain @160:11..160:19]: "true" [plain @160:21..160:25]
                  - mapping [block]
                    "mountPath" [plain @161:11..161:20]: "/etc/otelcol-contrib/config.yaml" [plain @161:22..161:54]
                    "name" [plain @162:11..162:15]: "data" [plain @162:17..162:21]
                    "subPath" [plain @163:11..163:18]: "config.yaml" [plain @163:20..163:31]
                    "readOnly" [plain @164:11..164:19]: "true" [plain @164:21..164:25]
            "terminationGracePeriodSeconds" [plain @165:7..165:36]: "30" [plain @165:38..165:40]
            "volumes" [plain @166:7..166:14]: sequence [block]
              - mapping [block]
                "name" [plain @167:9..167:13]: "varlog" [plain @167:15..167:21]
                "hostPath" [plain @168:9..168:17]: mapping [block]
                  "path" [plain @169:11..169:15]: "/var/log" [plain @169:17..169:25]
              - mapping [block]
                "name" [plain @170:9..170:13]: "varlibdockercontainers" [plain @170:15..170:37]
                "hostPath" [plain @171:9..171:17]: mapping [block]
                  "path" [plain @172:11..172:15]: "/var/lib/docker/containers" [plain @172:17..172:43]
              - mapping [block]
                "name" [plain @173:9..173:13]: "data" [plain @173:15..173:19]
                "configMap" [plain @174:9..174:18]: mapping [block]
                  "name" [plain @175:11..175:15]: "otel-collector-config" [plain @175:17..175:38]

    document 3 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @178:1..178:11]: "v1" [plain @178:13..178:15]
      "kind" [plain @179:1..179:5]: "Service" [plain @179:7..179:14]
      "metadata" [plain @180:1..180:9]: mapping [block]
        "name" [plain @181:3..181:7]: "otel-collector" [plain @181:9..181:23]
        "labels" [plain @182:3..182:9]: mapping [block]
          "app" [plain @183:5..183:8]: "opentelemetry" [plain @183:10..183:23]
          "component" [plain @184:5..184:14]: "otel-collector" [plain @184:16..184:30]
      "spec" [plain @185:1..185:5]: mapping [block]
        "ports" [plain @186:3..186:8]: sequence [block]
          - mapping [block]
            "name" [plain @187:5..187:9]: "metrics" [plain @187:11..187:18]
            "port" [plain @188:5..188:9]: "8888" [plain @188:11..188:15]
        "selector" [plain @189:3..189:11]: mapping [block]
          "component" [plain @190:5..190:14]: "otel-collector" [plain @190:16..190:30]
    "#);
}
