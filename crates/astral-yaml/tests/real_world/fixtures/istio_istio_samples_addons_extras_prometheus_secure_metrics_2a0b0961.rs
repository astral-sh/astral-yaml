#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("istio-istio-samples-addons-extras-prometheus-secure-metrics-2a0b0961"), @r#"
    document 1 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @28:1..28:11]: "v1" [plain @28:13..28:15]
      "kind" [plain @29:1..29:5]: "ServiceAccount" [plain @29:7..29:21]
      "metadata" [plain @30:1..30:9]: mapping [block]
        "labels" [plain @31:3..31:9]: mapping [block]
          "app.kubernetes.io/component" [plain @32:5..32:32]: "server" [plain @32:34..32:40]
          "app.kubernetes.io/name" [plain @33:5..33:27]: "prometheus" [plain @33:29..33:39]
          "app.kubernetes.io/instance" [plain @34:5..34:31]: "prometheus" [plain @34:33..34:43]
          "app.kubernetes.io/version" [plain @35:5..35:30]: "v3.10.0" [plain @35:32..35:39]
          "helm.sh/chart" [plain @36:5..36:18]: "prometheus-28.13.0" [plain @36:20..36:38]
          "app.kubernetes.io/part-of" [plain @37:5..37:30]: "prometheus" [plain @37:32..37:42]
        "name" [plain @38:3..38:7]: "prometheus" [plain @38:9..38:19]
        "namespace" [plain @39:3..39:12]: "istio-system" [plain @39:14..39:26]
        "annotations" [plain @40:3..40:14]: mapping [flow]

    document 2 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @43:1..43:11]: "v1" [plain @43:13..43:15]
      "kind" [plain @44:1..44:5]: "ConfigMap" [plain @44:7..44:16]
      "metadata" [plain @45:1..45:9]: mapping [block]
        "labels" [plain @46:3..46:9]: mapping [block]
          "app.kubernetes.io/component" [plain @47:5..47:32]: "server" [plain @47:34..47:40]
          "app.kubernetes.io/name" [plain @48:5..48:27]: "prometheus" [plain @48:29..48:39]
          "app.kubernetes.io/instance" [plain @49:5..49:31]: "prometheus" [plain @49:33..49:43]
          "app.kubernetes.io/version" [plain @50:5..50:30]: "v3.10.0" [plain @50:32..50:39]
          "helm.sh/chart" [plain @51:5..51:18]: "prometheus-28.13.0" [plain @51:20..51:38]
          "app.kubernetes.io/part-of" [plain @52:5..52:30]: "prometheus" [plain @52:32..52:42]
        "name" [plain @53:3..53:7]: "prometheus" [plain @53:9..53:19]
        "namespace" [plain @54:3..54:12]: "istio-system" [plain @54:14..54:26]
      "data" [plain @55:1..55:5]: mapping [block]
        "allow-snippet-annotations" [plain @56:3..56:28]: "false" [double-quoted @56:30..56:37]
        "alerting_rules.yml" [plain @57:3..57:21]: literal [@57:23..59:1]
          "{}\n"
        "alerts" [plain @59:3..59:9]: literal [@59:11..61:1]
          "{}\n"
        "prometheus.yml" [plain @61:3..61:17]: literal [@61:19..410:1]
          "global:\n"
          "  evaluation_interval: 1m\n"
          "  scrape_interval: 15s\n"
          "  scrape_timeout: 10s\n"
          "scrape_configs:\n"
          "- job_name: istio-secure-merged-metrics\n"
          "  # Discovers pods annotated with prometheus.istio.io/secure-port.\n"
          "  # This annotation is automatically set by the sidecar injector when\n"
          "  # ENVOY_SECURE_MERGED_METRICS_PORT is configured on the workload.\n"
          "  kubernetes_sd_configs:\n"
          "  - role: pod\n"
          "  relabel_configs:\n"
          "  - source_labels: [__meta_kubernetes_pod_annotation_prometheus_istio_io_secure_port]\n"
          "    action: keep\n"
          "    regex: .+\n"
          "  - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_path]\n"
          "    action: replace\n"
          "    target_label: __metrics_path__\n"
          "    regex: (.+)\n"
          "  - source_labels: [__meta_kubernetes_pod_ip, __meta_kubernetes_pod_annotation_prometheus_istio_io_secure_port]\n"
          "    action: replace\n"
          "    target_label: __address__\n"
          "    regex: (.+);(.+)\n"
          "    replacement: $1:$2\n"
          "  scheme: https\n"
          "  tls_config:\n"
          "    ca_file: /etc/istio-certs/root-cert.pem\n"
          "    cert_file: /etc/istio-certs/cert-chain.pem\n"
          "    key_file: /etc/istio-certs/key.pem\n"
          "    insecure_skip_verify: true\n"
          "- job_name: istio-secure-envoy-metrics\n"
          "  # Discovers pods annotated with prometheus.istio.io/secure-envoy-port.\n"
          "  # This annotation is NOT automatically set; add it manually to workloads\n"
          "  # that configure ENVOY_SECURE_METRICS_PORT.\n"
          "  kubernetes_sd_configs:\n"
          "  - role: pod\n"
          "  relabel_configs:\n"
          "  - source_labels: [__meta_kubernetes_pod_annotation_prometheus_istio_io_secure_envoy_port]\n"
          "    action: keep\n"
          "    regex: .+\n"
          "  - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_path]\n"
          "    action: replace\n"
          "    target_label: __metrics_path__\n"
          "    regex: (.+)\n"
          "  - source_labels: [__meta_kubernetes_pod_ip, __meta_kubernetes_pod_annotation_prometheus_istio_io_secure_envoy_port]\n"
          "    action: replace\n"
          "    target_label: __address__\n"
          "    regex: (.+);(.+)\n"
          "    replacement: $1:$2\n"
          "  scheme: https\n"
          "  tls_config:\n"
          "    ca_file: /etc/istio-certs/root-cert.pem\n"
          "    cert_file: /etc/istio-certs/cert-chain.pem\n"
          "    key_file: /etc/istio-certs/key.pem\n"
          "    insecure_skip_verify: true\n"
          "- job_name: kubernetes-api-servers\n"
          "  bearer_token_file: /var/run/secrets/kubernetes.io/serviceaccount/token\n"
          "  kubernetes_sd_configs:\n"
          "  - role: endpoints\n"
          "  relabel_configs:\n"
          "  - action: keep\n"
          "    regex: default;kubernetes;https\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_namespace\n"
          "    - __meta_kubernetes_service_name\n"
          "    - __meta_kubernetes_endpoint_port_name\n"
          "  scheme: https\n"
          "  tls_config:\n"
          "    ca_file: /var/run/secrets/kubernetes.io/serviceaccount/ca.crt\n"
          "- job_name: kubernetes-nodes\n"
          "  bearer_token_file: /var/run/secrets/kubernetes.io/serviceaccount/token\n"
          "  kubernetes_sd_configs:\n"
          "  - role: node\n"
          "  relabel_configs:\n"
          "  - action: labelmap\n"
          "    regex: __meta_kubernetes_node_label_(.+)\n"
          "  scheme: https\n"
          "  tls_config:\n"
          "    ca_file: /var/run/secrets/kubernetes.io/serviceaccount/ca.crt\n"
          "    insecure_skip_verify: true\n"
          "- job_name: kubernetes-nodes-cadvisor\n"
          "  bearer_token_file: /var/run/secrets/kubernetes.io/serviceaccount/token\n"
          "  kubernetes_sd_configs:\n"
          "  - role: node\n"
          "  metrics_path: /metrics/cadvisor\n"
          "  relabel_configs:\n"
          "  - action: labelmap\n"
          "    regex: __meta_kubernetes_node_label_(.+)\n"
          "  - source_labels:\n"
          "    - __metrics_path__\n"
          "    target_label: metrics_path\n"
          "  scheme: https\n"
          "  tls_config:\n"
          "    ca_file: /var/run/secrets/kubernetes.io/serviceaccount/ca.crt\n"
          "    insecure_skip_verify: true\n"
          "- job_name: kubernetes-pods\n"
          "  honor_labels: true\n"
          "  kubernetes_sd_configs:\n"
          "  - role: pod\n"
          "  relabel_configs:\n"
          "  - action: keep\n"
          "    regex: true\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_pod_annotation_prometheus_io_scrape\n"
          "  - action: drop\n"
          "    regex: true\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_pod_annotation_prometheus_io_scrape_slow\n"
          "  - action: replace\n"
          "    regex: (https?)\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_pod_annotation_prometheus_io_scheme\n"
          "    target_label: __scheme__\n"
          "  - action: replace\n"
          "    regex: (.+)\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_pod_annotation_prometheus_io_path\n"
          "    target_label: __metrics_path__\n"
          "  - action: replace\n"
          "    regex: (\\d+);(([A-Fa-f0-9]{1,4}::?){1,7}[A-Fa-f0-9]{1,4})\n"
          "    replacement: '[$2]:$1'\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_pod_annotation_prometheus_io_port\n"
          "    - __meta_kubernetes_pod_ip\n"
          "    target_label: __address__\n"
          "  - action: replace\n"
          "    regex: (\\d+);((([0-9]+?)(\\.|$)){4})\n"
          "    replacement: $2:$1\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_pod_annotation_prometheus_io_port\n"
          "    - __meta_kubernetes_pod_ip\n"
          "    target_label: __address__\n"
          "  - action: labelmap\n"
          "    regex: __meta_kubernetes_pod_annotation_prometheus_io_param_(.+)\n"
          "    replacement: __param_$1\n"
          "  - action: labelmap\n"
          "    regex: __meta_kubernetes_pod_label_(.+)\n"
          "  - action: replace\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_namespace\n"
          "    target_label: namespace\n"
          "  - action: replace\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_pod_name\n"
          "    target_label: pod\n"
          "  - action: drop\n"
          "    regex: Pending|Succeeded|Failed|Completed\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_pod_phase\n"
          "  - action: replace\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_pod_node_name\n"
          "    target_label: node\n"
          "- job_name: kubernetes-pods-slow\n"
          "  honor_labels: true\n"
          "  kubernetes_sd_configs:\n"
          "  - role: pod\n"
          "  relabel_configs:\n"
          "  - action: keep\n"
          "    regex: true\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_pod_annotation_prometheus_io_scrape_slow\n"
          "  - action: replace\n"
          "    regex: (https?)\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_pod_annotation_prometheus_io_scheme\n"
          "    target_label: __scheme__\n"
          "  - action: replace\n"
          "    regex: (.+)\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_pod_annotation_prometheus_io_path\n"
          "    target_label: __metrics_path__\n"
          "  - action: replace\n"
          "    regex: (\\d+);(([A-Fa-f0-9]{1,4}::?){1,7}[A-Fa-f0-9]{1,4})\n"
          "    replacement: '[$2]:$1'\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_pod_annotation_prometheus_io_port\n"
          "    - __meta_kubernetes_pod_ip\n"
          "    target_label: __address__\n"
          "  - action: replace\n"
          "    regex: (\\d+);((([0-9]+?)(\\.|$)){4})\n"
          "    replacement: $2:$1\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_pod_annotation_prometheus_io_port\n"
          "    - __meta_kubernetes_pod_ip\n"
          "    target_label: __address__\n"
          "  - action: labelmap\n"
          "    regex: __meta_kubernetes_pod_annotation_prometheus_io_param_(.+)\n"
          "    replacement: __param_$1\n"
          "  - action: labelmap\n"
          "    regex: __meta_kubernetes_pod_label_(.+)\n"
          "  - action: replace\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_namespace\n"
          "    target_label: namespace\n"
          "  - action: replace\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_pod_name\n"
          "    target_label: pod\n"
          "  - action: drop\n"
          "    regex: Pending|Succeeded|Failed|Completed\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_pod_phase\n"
          "  - action: replace\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_pod_node_name\n"
          "    target_label: node\n"
          "  scrape_interval: 5m\n"
          "  scrape_timeout: 30s\n"
          "- job_name: kubernetes-service-endpoints\n"
          "  honor_labels: true\n"
          "  kubernetes_sd_configs:\n"
          "  - role: endpoints\n"
          "  relabel_configs:\n"
          "  - action: keep\n"
          "    regex: true\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_service_annotation_prometheus_io_scrape\n"
          "  - action: drop\n"
          "    regex: true\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_service_annotation_prometheus_io_scrape_slow\n"
          "  - action: replace\n"
          "    regex: (https?)\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_service_annotation_prometheus_io_scheme\n"
          "    target_label: __scheme__\n"
          "  - action: replace\n"
          "    regex: (.+)\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_service_annotation_prometheus_io_path\n"
          "    target_label: __metrics_path__\n"
          "  - action: replace\n"
          "    regex: (.+?)(?::\\d+)?;(\\d+)\n"
          "    replacement: $1:$2\n"
          "    source_labels:\n"
          "    - __address__\n"
          "    - __meta_kubernetes_service_annotation_prometheus_io_port\n"
          "    target_label: __address__\n"
          "  - action: labelmap\n"
          "    regex: __meta_kubernetes_service_annotation_prometheus_io_param_(.+)\n"
          "    replacement: __param_$1\n"
          "  - action: labelmap\n"
          "    regex: __meta_kubernetes_service_label_(.+)\n"
          "  - action: replace\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_namespace\n"
          "    target_label: namespace\n"
          "  - action: replace\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_service_name\n"
          "    target_label: service\n"
          "  - action: replace\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_pod_node_name\n"
          "    target_label: node\n"
          "- job_name: kubernetes-service-endpoints-slow\n"
          "  honor_labels: true\n"
          "  kubernetes_sd_configs:\n"
          "  - role: endpoints\n"
          "  relabel_configs:\n"
          "  - action: keep\n"
          "    regex: true\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_service_annotation_prometheus_io_scrape_slow\n"
          "  - action: replace\n"
          "    regex: (https?)\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_service_annotation_prometheus_io_scheme\n"
          "    target_label: __scheme__\n"
          "  - action: replace\n"
          "    regex: (.+)\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_service_annotation_prometheus_io_path\n"
          "    target_label: __metrics_path__\n"
          "  - action: replace\n"
          "    regex: (.+?)(?::\\d+)?;(\\d+)\n"
          "    replacement: $1:$2\n"
          "    source_labels:\n"
          "    - __address__\n"
          "    - __meta_kubernetes_service_annotation_prometheus_io_port\n"
          "    target_label: __address__\n"
          "  - action: labelmap\n"
          "    regex: __meta_kubernetes_service_annotation_prometheus_io_param_(.+)\n"
          "    replacement: __param_$1\n"
          "  - action: labelmap\n"
          "    regex: __meta_kubernetes_service_label_(.+)\n"
          "  - action: replace\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_namespace\n"
          "    target_label: namespace\n"
          "  - action: replace\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_service_name\n"
          "    target_label: service\n"
          "  - action: replace\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_pod_node_name\n"
          "    target_label: node\n"
          "  scrape_interval: 5m\n"
          "  scrape_timeout: 30s\n"
          "- job_name: kubernetes-services\n"
          "  honor_labels: true\n"
          "  kubernetes_sd_configs:\n"
          "  - role: service\n"
          "  metrics_path: /probe\n"
          "  params:\n"
          "    module:\n"
          "    - http_2xx\n"
          "  relabel_configs:\n"
          "  - action: keep\n"
          "    regex: true\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_service_annotation_prometheus_io_probe\n"
          "  - source_labels:\n"
          "    - __address__\n"
          "    target_label: __param_target\n"
          "  - replacement: blackbox\n"
          "    target_label: __address__\n"
          "  - source_labels:\n"
          "    - __param_target\n"
          "    target_label: instance\n"
          "  - action: labelmap\n"
          "    regex: __meta_kubernetes_service_label_(.+)\n"
          "  - source_labels:\n"
          "    - __meta_kubernetes_namespace\n"
          "    target_label: namespace\n"
          "  - source_labels:\n"
          "    - __meta_kubernetes_service_name\n"
          "    target_label: service\n"
          "- job_name: prometheus\n"
          "  static_configs:\n"
          "  - targets:\n"
          "    - localhost:9090\n"
          "- job_name: prometheus-pushgateway\n"
          "  honor_labels: true\n"
          "  kubernetes_sd_configs:\n"
          "  - role: service\n"
          "  relabel_configs:\n"
          "  - action: keep\n"
          "    regex: pushgateway\n"
          "    source_labels:\n"
          "    - __meta_kubernetes_service_annotation_prometheus_io_probe\n"
          "rule_files:\n"
          "- /etc/config/recording_rules.yml\n"
          "- /etc/config/alerting_rules.yml\n"
          "- /etc/config/rules\n"
          "- /etc/config/alerts\n"
        "recording_rules.yml" [plain @410:3..410:22]: literal [@410:24..412:1]
          "{}\n"
        "rules" [plain @412:3..412:8]: literal [@412:10..414:1]
          "{}\n"

    document 3 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @415:1..415:11]: "rbac.authorization.k8s.io/v1" [plain @415:13..415:41]
      "kind" [plain @416:1..416:5]: "ClusterRole" [plain @416:7..416:18]
      "metadata" [plain @417:1..417:9]: mapping [block]
        "labels" [plain @418:3..418:9]: mapping [block]
          "app.kubernetes.io/component" [plain @419:5..419:32]: "server" [plain @419:34..419:40]
          "app.kubernetes.io/name" [plain @420:5..420:27]: "prometheus" [plain @420:29..420:39]
          "app.kubernetes.io/instance" [plain @421:5..421:31]: "prometheus" [plain @421:33..421:43]
          "app.kubernetes.io/version" [plain @422:5..422:30]: "v3.10.0" [plain @422:32..422:39]
          "helm.sh/chart" [plain @423:5..423:18]: "prometheus-28.13.0" [plain @423:20..423:38]
          "app.kubernetes.io/part-of" [plain @424:5..424:30]: "prometheus" [plain @424:32..424:42]
        "name" [plain @425:3..425:7]: "prometheus" [plain @425:9..425:19]
      "rules" [plain @426:1..426:6]: sequence [block]
        - mapping [block]
          "apiGroups" [plain @427:5..427:14]: sequence [block]
            - "" [double-quoted @428:9..428:11]
          "resources" [plain @429:5..429:14]: sequence [block]
            - "nodes" [plain @430:9..430:14]
            - "nodes/metrics" [plain @431:9..431:22]
            - "services" [plain @432:9..432:17]
            - "endpoints" [plain @433:9..433:18]
            - "pods" [plain @434:9..434:13]
            - "ingresses" [plain @435:9..435:18]
            - "configmaps" [plain @436:9..436:19]
          "verbs" [plain @437:5..437:10]: sequence [block]
            - "get" [plain @438:9..438:12]
            - "list" [plain @439:9..439:13]
            - "watch" [plain @440:9..440:14]
        - mapping [block]
          "apiGroups" [plain @441:5..441:14]: sequence [block]
            - "networking.k8s.io" [double-quoted @442:9..442:28]
          "resources" [plain @443:5..443:14]: sequence [block]
            - "ingresses/status" [plain @444:9..444:25]
            - "ingresses" [plain @445:9..445:18]
          "verbs" [plain @446:5..446:10]: sequence [block]
            - "get" [plain @447:9..447:12]
            - "list" [plain @448:9..448:13]
            - "watch" [plain @449:9..449:14]
        - mapping [block]
          "apiGroups" [plain @450:5..450:14]: sequence [block]
            - "discovery.k8s.io" [double-quoted @451:9..451:27]
          "resources" [plain @452:5..452:14]: sequence [block]
            - "endpointslices" [plain @453:9..453:23]
          "verbs" [plain @454:5..454:10]: sequence [block]
            - "get" [plain @455:9..455:12]
            - "list" [plain @456:9..456:13]
            - "watch" [plain @457:9..457:14]
        - mapping [block]
          "nonResourceURLs" [plain @458:5..458:20]: sequence [block]
            - "/metrics" [double-quoted @459:9..459:19]
          "verbs" [plain @460:5..460:10]: sequence [block]
            - "get" [plain @461:9..461:12]

    document 4 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @463:1..463:11]: "rbac.authorization.k8s.io/v1" [plain @463:13..463:41]
      "kind" [plain @464:1..464:5]: "ClusterRoleBinding" [plain @464:7..464:25]
      "metadata" [plain @465:1..465:9]: mapping [block]
        "labels" [plain @466:3..466:9]: mapping [block]
          "app.kubernetes.io/component" [plain @467:5..467:32]: "server" [plain @467:34..467:40]
          "app.kubernetes.io/name" [plain @468:5..468:27]: "prometheus" [plain @468:29..468:39]
          "app.kubernetes.io/instance" [plain @469:5..469:31]: "prometheus" [plain @469:33..469:43]
          "app.kubernetes.io/version" [plain @470:5..470:30]: "v3.10.0" [plain @470:32..470:39]
          "helm.sh/chart" [plain @471:5..471:18]: "prometheus-28.13.0" [plain @471:20..471:38]
          "app.kubernetes.io/part-of" [plain @472:5..472:30]: "prometheus" [plain @472:32..472:42]
        "name" [plain @473:3..473:7]: "prometheus" [plain @473:9..473:19]
      "subjects" [plain @474:1..474:9]: sequence [block]
        - mapping [block]
          "kind" [plain @475:5..475:9]: "ServiceAccount" [plain @475:11..475:25]
          "name" [plain @476:5..476:9]: "prometheus" [plain @476:11..476:21]
          "namespace" [plain @477:5..477:14]: "istio-system" [plain @477:16..477:28]
      "roleRef" [plain @478:1..478:8]: mapping [block]
        "apiGroup" [plain @479:3..479:11]: "rbac.authorization.k8s.io" [plain @479:13..479:38]
        "kind" [plain @480:3..480:7]: "ClusterRole" [plain @480:9..480:20]
        "name" [plain @481:3..481:7]: "prometheus" [plain @481:9..481:19]

    document 5 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @483:1..483:11]: "v1" [plain @483:13..483:15]
      "kind" [plain @484:1..484:5]: "Service" [plain @484:7..484:14]
      "metadata" [plain @485:1..485:9]: mapping [block]
        "labels" [plain @486:3..486:9]: mapping [block]
          "app.kubernetes.io/component" [plain @487:5..487:32]: "server" [plain @487:34..487:40]
          "app.kubernetes.io/name" [plain @488:5..488:27]: "prometheus" [plain @488:29..488:39]
          "app.kubernetes.io/instance" [plain @489:5..489:31]: "prometheus" [plain @489:33..489:43]
          "app.kubernetes.io/version" [plain @490:5..490:30]: "v3.10.0" [plain @490:32..490:39]
          "helm.sh/chart" [plain @491:5..491:18]: "prometheus-28.13.0" [plain @491:20..491:38]
          "app.kubernetes.io/part-of" [plain @492:5..492:30]: "prometheus" [plain @492:32..492:42]
        "name" [plain @493:3..493:7]: "prometheus" [plain @493:9..493:19]
        "namespace" [plain @494:3..494:12]: "istio-system" [plain @494:14..494:26]
      "spec" [plain @495:1..495:5]: mapping [block]
        "ports" [plain @496:3..496:8]: sequence [block]
          - mapping [block]
            "name" [plain @497:7..497:11]: "http" [plain @497:13..497:17]
            "port" [plain @498:7..498:11]: "9090" [plain @498:13..498:17]
            "protocol" [plain @499:7..499:15]: "TCP" [plain @499:17..499:20]
            "targetPort" [plain @500:7..500:17]: "9090" [plain @500:19..500:23]
        "selector" [plain @501:3..501:11]: mapping [block]
          "app.kubernetes.io/component" [plain @502:5..502:32]: "server" [plain @502:34..502:40]
          "app.kubernetes.io/name" [plain @503:5..503:27]: "prometheus" [plain @503:29..503:39]
          "app.kubernetes.io/instance" [plain @504:5..504:31]: "prometheus" [plain @504:33..504:43]
        "sessionAffinity" [plain @505:3..505:18]: "None" [plain @505:20..505:24]
        "type" [plain @506:3..506:7]: "ClusterIP" [double-quoted @506:9..506:20]

    document 6 (start: ---, end: implicit)
    mapping [block]
      "apiVersion" [plain @508:1..508:11]: "apps/v1" [plain @508:13..508:20]
      "kind" [plain @509:1..509:5]: "Deployment" [plain @509:7..509:17]
      "metadata" [plain @510:1..510:9]: mapping [block]
        "labels" [plain @511:3..511:9]: mapping [block]
          "app.kubernetes.io/component" [plain @512:5..512:32]: "server" [plain @512:34..512:40]
          "app.kubernetes.io/name" [plain @513:5..513:27]: "prometheus" [plain @513:29..513:39]
          "app.kubernetes.io/instance" [plain @514:5..514:31]: "prometheus" [plain @514:33..514:43]
          "app.kubernetes.io/version" [plain @515:5..515:30]: "v3.10.0" [plain @515:32..515:39]
          "helm.sh/chart" [plain @516:5..516:18]: "prometheus-28.13.0" [plain @516:20..516:38]
          "app.kubernetes.io/part-of" [plain @517:5..517:30]: "prometheus" [plain @517:32..517:42]
        "name" [plain @518:3..518:7]: "prometheus" [plain @518:9..518:19]
        "namespace" [plain @519:3..519:12]: "istio-system" [plain @519:14..519:26]
      "spec" [plain @520:1..520:5]: mapping [block]
        "strategy" [plain @521:3..521:11]: mapping [block]
          "type" [plain @522:5..522:9]: "Recreate" [plain @522:11..522:19]
          "rollingUpdate" [plain @523:5..523:18]: "null" [plain @523:20..523:24]
        "selector" [plain @524:3..524:11]: mapping [block]
          "matchLabels" [plain @525:5..525:16]: mapping [block]
            "app.kubernetes.io/component" [plain @526:7..526:34]: "server" [plain @526:36..526:42]
            "app.kubernetes.io/name" [plain @527:7..527:29]: "prometheus" [plain @527:31..527:41]
            "app.kubernetes.io/instance" [plain @528:7..528:33]: "prometheus" [plain @528:35..528:45]
        "replicas" [plain @529:3..529:11]: "1" [plain @529:13..529:14]
        "revisionHistoryLimit" [plain @530:3..530:23]: "10" [plain @530:25..530:27]
        "template" [plain @531:3..531:11]: mapping [block]
          "metadata" [plain @532:5..532:13]: mapping [block]
            "labels" [plain @533:7..533:13]: mapping [block]
              "app.kubernetes.io/component" [plain @534:9..534:36]: "server" [plain @534:38..534:44]
              "app.kubernetes.io/name" [plain @535:9..535:31]: "prometheus" [plain @535:33..535:43]
              "app.kubernetes.io/instance" [plain @536:9..536:35]: "prometheus" [plain @536:37..536:47]
              "app.kubernetes.io/version" [plain @537:9..537:34]: "v3.10.0" [plain @537:36..537:43]
              "helm.sh/chart" [plain @538:9..538:22]: "prometheus-28.13.0" [plain @538:24..538:42]
              "app.kubernetes.io/part-of" [plain @539:9..539:34]: "prometheus" [plain @539:36..539:46]
              "sidecar.istio.io/inject" [plain @540:9..540:32]: "true" [double-quoted @540:34..540:40]
            "annotations" [plain @541:7..541:18]: mapping [block]
              "sidecar.istio.io/userVolumeMount" [plain @542:9..542:41]: "[{\"name\": \"istio-certs\", \"mountPath\": \"/etc/istio-certs\"}]" [single-quoted @542:43..542:103]
              "proxy.istio.io/config" [plain @543:9..543:30]: literal [@543:32..547:1]
                "proxyMetadata:\n"
                "  OUTPUT_CERTS: /etc/istio-certs\n"
                "  INBOUND_CAPTURE_PORTS: \"\"\n"
          "spec" [plain @547:5..547:9]: mapping [block]
            "enableServiceLinks" [plain @548:7..548:25]: "true" [plain @548:27..548:31]
            "serviceAccountName" [plain @549:7..549:25]: "prometheus" [plain @549:27..549:37]
            "containers" [plain @550:7..550:17]: sequence [block]
              - mapping [block]
                "name" [plain @551:11..551:15]: "prometheus-server-configmap-reload" [plain @551:17..551:51]
                "image" [plain @552:11..552:16]: "ghcr.io/prometheus-operator/prometheus-config-reloader:v0.89.0" [double-quoted @552:18..552:82]
                "imagePullPolicy" [plain @553:11..553:26]: "IfNotPresent" [double-quoted @553:28..553:42]
                "args" [plain @554:11..554:15]: sequence [block]
                  - "--watched-dir=/etc/config" [plain @555:15..555:40]
                  - "--listen-address=0.0.0.0:8080" [plain @556:15..556:44]
                  - "--reload-url=http://127.0.0.1:9090/-/reload" [plain @557:15..557:58]
                "ports" [plain @558:11..558:16]: sequence [block]
                  - mapping [block]
                    "containerPort" [plain @559:15..559:28]: "8080" [plain @559:30..559:34]
                    "name" [plain @560:15..560:19]: "metrics" [plain @560:21..560:28]
                "livenessProbe" [plain @561:11..561:24]: mapping [block]
                  "httpGet" [plain @562:13..562:20]: mapping [block]
                    "path" [plain @563:15..563:19]: "/healthz" [plain @563:21..563:29]
                    "port" [plain @564:15..564:19]: "metrics" [plain @564:21..564:28]
                    "scheme" [plain @565:15..565:21]: "HTTP" [plain @565:23..565:27]
                  "initialDelaySeconds" [plain @566:13..566:32]: "2" [plain @566:34..566:35]
                  "periodSeconds" [plain @567:13..567:26]: "10" [plain @567:28..567:30]
                "readinessProbe" [plain @568:11..568:25]: mapping [block]
                  "httpGet" [plain @569:13..569:20]: mapping [block]
                    "path" [plain @570:15..570:19]: "/healthz" [plain @570:21..570:29]
                    "port" [plain @571:15..571:19]: "metrics" [plain @571:21..571:28]
                    "scheme" [plain @572:15..572:21]: "HTTP" [plain @572:23..572:27]
                  "periodSeconds" [plain @573:13..573:26]: "10" [plain @573:28..573:30]
                "volumeMounts" [plain @574:11..574:23]: sequence [block]
                  - mapping [block]
                    "name" [plain @575:15..575:19]: "config-volume" [plain @575:21..575:34]
                    "mountPath" [plain @576:15..576:24]: "/etc/config" [plain @576:26..576:37]
                    "readOnly" [plain @577:15..577:23]: "true" [plain @577:25..577:29]
              - mapping [block]
                "name" [plain @578:11..578:15]: "prometheus-server" [plain @578:17..578:34]
                "image" [plain @579:11..579:16]: "docker.io/prom/prometheus:v3.10.0" [double-quoted @579:18..579:53]
                "imagePullPolicy" [plain @580:11..580:26]: "IfNotPresent" [double-quoted @580:28..580:42]
                "args" [plain @581:11..581:15]: sequence [block]
                  - "--storage.tsdb.retention.time=15d" [plain @582:15..582:48]
                  - "--config.file=/etc/config/prometheus.yml" [plain @583:15..583:55]
                  - "--storage.tsdb.path=/data" [plain @584:15..584:40]
                  - "--web.console.libraries=/etc/prometheus/console_libraries" [plain @585:15..585:72]
                  - "--web.console.templates=/etc/prometheus/consoles" [plain @586:15..586:63]
                  - "--web.enable-lifecycle" [plain @587:15..587:37]
                "ports" [plain @588:11..588:16]: sequence [block]
                  - mapping [block]
                    "containerPort" [plain @589:15..589:28]: "9090" [plain @589:30..589:34]
                "readinessProbe" [plain @590:11..590:25]: mapping [block]
                  "httpGet" [plain @591:13..591:20]: mapping [block]
                    "path" [plain @592:15..592:19]: "/-/ready" [plain @592:21..592:29]
                    "port" [plain @593:15..593:19]: "9090" [plain @593:21..593:25]
                    "scheme" [plain @594:15..594:21]: "HTTP" [plain @594:23..594:27]
                  "initialDelaySeconds" [plain @595:13..595:32]: "0" [plain @595:34..595:35]
                  "periodSeconds" [plain @596:13..596:26]: "5" [plain @596:28..596:29]
                  "timeoutSeconds" [plain @597:13..597:27]: "4" [plain @597:29..597:30]
                  "failureThreshold" [plain @598:13..598:29]: "3" [plain @598:31..598:32]
                  "successThreshold" [plain @599:13..599:29]: "1" [plain @599:31..599:32]
                "livenessProbe" [plain @600:11..600:24]: mapping [block]
                  "httpGet" [plain @601:13..601:20]: mapping [block]
                    "path" [plain @602:15..602:19]: "/-/healthy" [plain @602:21..602:31]
                    "port" [plain @603:15..603:19]: "9090" [plain @603:21..603:25]
                    "scheme" [plain @604:15..604:21]: "HTTP" [plain @604:23..604:27]
                  "initialDelaySeconds" [plain @605:13..605:32]: "30" [plain @605:34..605:36]
                  "periodSeconds" [plain @606:13..606:26]: "15" [plain @606:28..606:30]
                  "timeoutSeconds" [plain @607:13..607:27]: "10" [plain @607:29..607:31]
                  "failureThreshold" [plain @608:13..608:29]: "3" [plain @608:31..608:32]
                  "successThreshold" [plain @609:13..609:29]: "1" [plain @609:31..609:32]
                "volumeMounts" [plain @610:11..610:23]: sequence [block]
                  - mapping [block]
                    "name" [plain @611:15..611:19]: "config-volume" [plain @611:21..611:34]
                    "mountPath" [plain @612:15..612:24]: "/etc/config" [plain @612:26..612:37]
                  - mapping [block]
                    "name" [plain @613:15..613:19]: "storage-volume" [plain @613:21..613:35]
                    "mountPath" [plain @614:15..614:24]: "/data" [plain @614:26..614:31]
                    "subPath" [plain @615:15..615:22]: "" [double-quoted @615:24..615:26]
                  - mapping [block]
                    "name" [plain @616:15..616:19]: "istio-certs" [plain @616:21..616:32]
                    "mountPath" [plain @617:15..617:24]: "/etc/istio-certs" [plain @617:26..617:42]
            "dnsPolicy" [plain @618:7..618:16]: "ClusterFirst" [plain @618:18..618:30]
            "terminationGracePeriodSeconds" [plain @619:7..619:36]: "300" [plain @619:38..619:41]
            "volumes" [plain @620:7..620:14]: sequence [block]
              - mapping [block]
                "name" [plain @621:11..621:15]: "config-volume" [plain @621:17..621:30]
                "configMap" [plain @622:11..622:20]: mapping [block]
                  "name" [plain @623:13..623:17]: "prometheus" [plain @623:19..623:29]
              - mapping [block]
                "name" [plain @624:11..624:15]: "storage-volume" [plain @624:17..624:31]
                "emptyDir" [plain @625:11..625:19]: mapping [flow]
              - mapping [block]
                "name" [plain @627:11..627:15]: "istio-certs" [plain @627:17..627:28]
                "emptyDir" [plain @628:11..628:19]: mapping [flow]
    "#);
}
