#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("kubernetes-examples-ai-vllm-deployment-hpa-prometheus-adapter-2479583d"), @r##"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @9:1..9:11]: "v1" [plain @9:13..9:15]
      "kind" [plain @10:1..10:5]: "ServiceAccount" [plain @10:7..10:21]
      "metadata" [plain @11:1..11:9]: mapping [block]
        "name" [plain @12:3..12:7]: "prometheus-adapter" [plain @12:9..12:27]
        "namespace" [plain @13:3..13:12]: "monitoring" [plain @13:14..13:24]

    document 2 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @15:1..15:11]: "rbac.authorization.k8s.io/v1" [plain @15:13..15:41]
      "kind" [plain @16:1..16:5]: "ClusterRole" [plain @16:7..16:18]
      "metadata" [plain @17:1..17:9]: mapping [block]
        "name" [plain @18:3..18:7]: "prometheus-adapter" [plain @18:9..18:27]
      "rules" [plain @19:1..19:6]: sequence [block]
        - mapping [block]
          "apiGroups" [plain @20:3..20:12]: sequence [block]
            - "" [double-quoted @21:5..21:7]
          "resources" [plain @22:3..22:12]: sequence [block]
            - "nodes" [plain @23:5..23:10]
            - "namespaces" [plain @24:5..24:15]
            - "pods" [plain @25:5..25:9]
            - "services" [plain @26:5..26:13]
          "verbs" [plain @27:3..27:8]: sequence [block]
            - "get" [plain @28:5..28:8]
            - "list" [plain @29:5..29:9]
            - "watch" [plain @30:5..30:10]
        - mapping [block]
          "apiGroups" [plain @31:3..31:12]: sequence [block]
            - "extensions" [plain @32:5..32:15]
          "resources" [plain @33:3..33:12]: sequence [block]
            - "deployments" [plain @34:5..34:16]
            - "replicasets" [plain @35:5..35:16]
            - "ingresses" [plain @36:5..36:14]
          "verbs" [plain @37:3..37:8]: sequence [block]
            - "get" [plain @38:5..38:8]
            - "list" [plain @39:5..39:9]
            - "watch" [plain @40:5..40:10]
        - mapping [block]
          "apiGroups" [plain @41:3..41:12]: sequence [block]
            - "apps" [plain @42:5..42:9]
          "resources" [plain @43:3..43:12]: sequence [block]
            - "deployments" [plain @44:5..44:16]
            - "replicasets" [plain @45:5..45:16]
            - "statefulsets" [plain @46:5..46:17]
          "verbs" [plain @47:3..47:8]: sequence [block]
            - "get" [plain @48:5..48:8]
            - "list" [plain @49:5..49:9]
            - "watch" [plain @50:5..50:10]
        - mapping [block]
          "apiGroups" [plain @51:3..51:12]: sequence [block]
            - "custom.metrics.k8s.io" [plain @52:5..52:26]
          "resources" [plain @53:3..53:12]: sequence [block]
            - "*" [single-quoted @54:5..54:8]
          "verbs" [plain @55:3..55:8]: sequence [block]
            - "*" [single-quoted @56:5..56:8]

    document 3 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @58:1..58:11]: "rbac.authorization.k8s.io/v1" [plain @58:13..58:41]
      "kind" [plain @59:1..59:5]: "ClusterRoleBinding" [plain @59:7..59:25]
      "metadata" [plain @60:1..60:9]: mapping [block]
        "name" [plain @61:3..61:7]: "prometheus-adapter" [plain @61:9..61:27]
      "roleRef" [plain @62:1..62:8]: mapping [block]
        "apiGroup" [plain @63:3..63:11]: "rbac.authorization.k8s.io" [plain @63:13..63:38]
        "kind" [plain @64:3..64:7]: "ClusterRole" [plain @64:9..64:20]
        "name" [plain @65:3..65:7]: "prometheus-adapter" [plain @65:9..65:27]
      "subjects" [plain @66:1..66:9]: sequence [block]
        - mapping [block]
          "kind" [plain @67:3..67:7]: "ServiceAccount" [plain @67:9..67:23]
          "name" [plain @68:3..68:7]: "prometheus-adapter" [plain @68:9..68:27]
          "namespace" [plain @69:3..69:12]: "monitoring" [plain @69:14..69:24]

    document 4 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @71:1..71:11]: "rbac.authorization.k8s.io/v1" [plain @71:13..71:41]
      "kind" [plain @72:1..72:5]: "RoleBinding" [plain @72:7..72:18]
      "metadata" [plain @73:1..73:9]: mapping [block]
        "name" [plain @74:3..74:7]: "prometheus-adapter-auth-reader" [plain @74:9..74:39]
        "namespace" [plain @75:3..75:12]: "kube-system" [plain @75:14..75:25]
      "roleRef" [plain @76:1..76:8]: mapping [block]
        "apiGroup" [plain @77:3..77:11]: "rbac.authorization.k8s.io" [plain @77:13..77:38]
        "kind" [plain @78:3..78:7]: "Role" [plain @78:9..78:13]
        "name" [plain @79:3..79:7]: "extension-apiserver-authentication-reader" [plain @79:9..79:50]
      "subjects" [plain @80:1..80:9]: sequence [block]
        - mapping [block]
          "kind" [plain @81:3..81:7]: "ServiceAccount" [plain @81:9..81:23]
          "name" [plain @82:3..82:7]: "prometheus-adapter" [plain @82:9..82:27]
          "namespace" [plain @83:3..83:12]: "monitoring" [plain @83:14..83:24]

    document 5 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @85:1..85:11]: "rbac.authorization.k8s.io/v1" [plain @85:13..85:41]
      "kind" [plain @86:1..86:5]: "ClusterRoleBinding" [plain @86:7..86:25]
      "metadata" [plain @87:1..87:9]: mapping [block]
        "name" [plain @88:3..88:7]: "prometheus-adapter-system-auth-delegator" [plain @88:9..88:49]
      "roleRef" [plain @89:1..89:8]: mapping [block]
        "apiGroup" [plain @90:3..90:11]: "rbac.authorization.k8s.io" [plain @90:13..90:38]
        "kind" [plain @91:3..91:7]: "ClusterRole" [plain @91:9..91:20]
        "name" [plain @92:3..92:7]: "system:auth-delegator" [plain @92:9..92:30]
      "subjects" [plain @93:1..93:9]: sequence [block]
        - mapping [block]
          "kind" [plain @94:3..94:7]: "ServiceAccount" [plain @94:9..94:23]
          "name" [plain @95:3..95:7]: "prometheus-adapter" [plain @95:9..95:27]
          "namespace" [plain @96:3..96:12]: "monitoring" [plain @96:14..96:24]

    document 6 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @98:1..98:11]: "v1" [plain @98:13..98:15]
      "kind" [plain @99:1..99:5]: "ConfigMap" [plain @99:7..99:16]
      "metadata" [plain @100:1..100:9]: mapping [block]
        "name" [plain @101:3..101:7]: "prometheus-adapter" [plain @101:9..101:27]
        "namespace" [plain @102:3..102:12]: "monitoring" [plain @102:14..102:24]
      "data" [plain @103:1..103:5]: mapping [block]
        "config.yaml" [plain @104:3..104:14]: literal [@104:16..131:1]
          "# This 'rules' section is the core of the Prometheus Adapter's configuration.\n"
          "# It defines how metrics from Prometheus are transformed and exposed to the\n"
          "# Kubernetes Custom Metrics API.\n"
          "rules:\n"
          "# This rule renames the 'vllm:num_requests_running' metric to\n"
          "# 'vllm_num_requests_running' to make it a valid custom metric name.\n"
          "- seriesQuery: 'vllm:num_requests_running'\n"
          "  resources:\n"
          "    overrides:\n"
          "      namespace: {resource: \"namespace\"}\n"
          "      pod: {resource: \"pod\"}\n"
          "  name:\n"
          "    matches: \"vllm:num_requests_running\"\n"
          "    as: \"vllm_num_requests_running\"\n"
          "  metricsQuery: 'sum(vllm:num_requests_running{<<.LabelMatchers>>}) by (<<.GroupBy>>)'\n"
          "  # It takes 'dcgm_fi_dev_gpu_util_relabelled' (which now has the correct\n"
          "  # 'pod' and 'namespace' labels) and exposes it as 'gpu_utilization_percent'.\n"
          "  - seriesQuery: 'dcgm_fi_dev_gpu_util_relabelled'\n"
          "    resources:\n"
          "      overrides:\n"
          "        namespace: {resource: \"namespace\"}\n"
          "        pod: {resource: \"pod\"}\n"
          "    name:\n"
          "      matches: \"dcgm_fi_dev_gpu_util_relabelled\"\n"
          "      as: \"gpu_utilization_percent\"\n"
          "    metricsQuery: 'sum(dcgm_fi_dev_gpu_util_relabelled{<<.LabelMatchers>>}) by (<<.GroupBy>>)'\n"

    document 7 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @132:1..132:11]: "apps/v1" [plain @132:13..132:20]
      "kind" [plain @133:1..133:5]: "Deployment" [plain @133:7..133:17]
      "metadata" [plain @134:1..134:9]: mapping [block]
        "name" [plain @135:3..135:7]: "prometheus-adapter" [plain @135:9..135:27]
        "namespace" [plain @136:3..136:12]: "monitoring" [plain @136:14..136:24]
      "spec" [plain @137:1..137:5]: mapping [block]
        "replicas" [plain @138:3..138:11]: "1" [plain @138:13..138:14]
        "selector" [plain @139:3..139:11]: mapping [block]
          "matchLabels" [plain @140:5..140:16]: mapping [block]
            "app" [plain @141:7..141:10]: "prometheus-adapter" [plain @141:12..141:30]
        "template" [plain @142:3..142:11]: mapping [block]
          "metadata" [plain @143:5..143:13]: mapping [block]
            "labels" [plain @144:7..144:13]: mapping [block]
              "app" [plain @145:9..145:12]: "prometheus-adapter" [plain @145:14..145:32]
          "spec" [plain @146:5..146:9]: mapping [block]
            "serviceAccountName" [plain @147:7..147:25]: "prometheus-adapter" [plain @147:27..147:45]
            "containers" [plain @148:7..148:17]: sequence [block]
              - mapping [block]
                "name" [plain @149:9..149:13]: "prometheus-adapter" [plain @149:15..149:33]
                "image" [plain @150:9..150:14]: "registry.k8s.io/prometheus-adapter/prometheus-adapter:v0.11.2" [plain @150:16..150:77]
                "args" [plain @151:9..151:13]: sequence [block]
                  - "--prometheus-url=http://prometheus-kube-prometheus-prometheus.monitoring.svc:9090/" [plain @152:11..152:93]
                  - "--metrics-relist-interval=1m" [plain @153:11..153:39]
                  - "--config=/etc/adapter/config.yaml" [plain @154:11..154:44]
                  - "--secure-port=6443" [plain @155:11..155:29]
                  - "--cert-dir=/tmp/cert" [plain @156:11..156:31]
                  - "--v=4" [plain @157:11..157:16]
                "ports" [plain @158:9..158:14]: sequence [block]
                  - mapping [block]
                    "containerPort" [plain @159:11..159:24]: "6443" [plain @159:26..159:30]
                    "name" [plain @160:11..160:15]: "https" [plain @160:17..160:22]
                "volumeMounts" [plain @161:9..161:21]: sequence [block]
                  - mapping [block]
                    "name" [plain @162:11..162:15]: "config" [plain @162:17..162:23]
                    "mountPath" [plain @163:11..163:20]: "/etc/adapter" [plain @163:22..163:34]
                    "readOnly" [plain @164:11..164:19]: "true" [plain @164:21..164:25]
                  - mapping [block]
                    "name" [plain @165:11..165:15]: "certs" [plain @165:17..165:22]
                    "mountPath" [plain @166:11..166:20]: "/tmp/cert" [plain @166:22..166:31]
            "volumes" [plain @167:7..167:14]: sequence [block]
              - mapping [block]
                "name" [plain @168:9..168:13]: "config" [plain @168:15..168:21]
                "configMap" [plain @169:9..169:18]: mapping [block]
                  "name" [plain @170:11..170:15]: "prometheus-adapter" [plain @170:17..170:35]
              - mapping [block]
                "name" [plain @171:9..171:13]: "certs" [plain @171:15..171:20]
                "emptyDir" [plain @172:9..172:17]: mapping [flow]

    document 8 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @174:1..174:11]: "v1" [plain @174:13..174:15]
      "kind" [plain @175:1..175:5]: "Service" [plain @175:7..175:14]
      "metadata" [plain @176:1..176:9]: mapping [block]
        "name" [plain @177:3..177:7]: "prometheus-adapter" [plain @177:9..177:27]
        "namespace" [plain @178:3..178:12]: "monitoring" [plain @178:14..178:24]
      "spec" [plain @179:1..179:5]: mapping [block]
        "ports" [plain @180:3..180:8]: sequence [block]
          - mapping [block]
            "port" [plain @181:5..181:9]: "443" [plain @181:11..181:14]
            "targetPort" [plain @182:5..182:15]: "https" [plain @182:17..182:22]
        "selector" [plain @183:3..183:11]: mapping [block]
          "app" [plain @184:5..184:8]: "prometheus-adapter" [plain @184:10..184:28]

    document 9 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @186:1..186:11]: "apiregistration.k8s.io/v1" [plain @186:13..186:38]
      "kind" [plain @187:1..187:5]: "APIService" [plain @187:7..187:17]
      "metadata" [plain @188:1..188:9]: mapping [block]
        "name" [plain @189:3..189:7]: "v1beta1.custom.metrics.k8s.io" [plain @189:9..189:38]
      "spec" [plain @190:1..190:5]: mapping [block]
        "service" [plain @191:3..191:10]: mapping [block]
          "name" [plain @192:5..192:9]: "prometheus-adapter" [plain @192:11..192:29]
          "namespace" [plain @193:5..193:14]: "monitoring" [plain @193:16..193:26]
        "group" [plain @194:3..194:8]: "custom.metrics.k8s.io" [plain @194:10..194:31]
        "version" [plain @195:3..195:10]: "v1beta1" [plain @195:12..195:19]
        "insecureSkipTLSVerify" [plain @196:3..196:24]: "true" [plain @196:26..196:30]
        "groupPriorityMinimum" [plain @197:3..197:23]: "100" [plain @197:25..197:28]
        "versionPriority" [plain @198:3..198:18]: "100" [plain @198:20..198:23]
    "##);
}
