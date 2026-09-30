#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("istio-istio-samples-security-spire-istio-spire-config-3d25d0e7"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "install.istio.io/v1alpha1" [plain @1:13..1:38]
      "kind" [plain @2:1..2:5]: "IstioOperator" [plain @2:7..2:20]
      "metadata" [plain @3:1..3:9]: mapping [block]
        "namespace" [plain @4:3..4:12]: "istio-system" [plain @4:14..4:26]
      "spec" [plain @5:1..5:5]: mapping [block]
        "profile" [plain @6:3..6:10]: "default" [plain @6:12..6:19]
        "meshConfig" [plain @7:3..7:13]: mapping [block]
          "trustDomain" [plain @8:5..8:16]: "example.org" [plain @8:18..8:29]
        "values" [plain @9:3..9:9]: mapping [block]
          "sidecarInjectorWebhook" [plain @11:5..11:27]: mapping [block]
            "templates" [plain @12:7..12:16]: mapping [block]
              "spire" [plain @13:9..13:14]: literal [@13:16..26:1]
                "spec:\n"
                "  containers:\n"
                "  - name: istio-proxy\n"
                "    volumeMounts:\n"
                "    - name: workload-socket\n"
                "      mountPath: /run/secrets/workload-spiffe-uds\n"
                "      readOnly: true\n"
                "  volumes:\n"
                "    - name: workload-socket\n"
                "      csi:\n"
                "        driver: \"csi.spiffe.io\"\n"
                "        readOnly: true\n"
        "components" [plain @26:3..26:13]: mapping [block]
          "ingressGateways" [plain @27:5..27:20]: sequence [block]
            - mapping [block]
              "name" [plain @28:9..28:13]: "istio-ingressgateway" [plain @28:15..28:35]
              "enabled" [plain @29:9..29:16]: "true" [plain @29:18..29:22]
              "label" [plain @30:9..30:14]: mapping [block]
                "istio" [plain @31:11..31:16]: "ingressgateway" [plain @31:18..31:32]
              "k8s" [plain @32:9..32:12]: mapping [block]
                "overlays" [plain @33:11..33:19]: sequence [block]
                  - mapping [block]
                    "apiVersion" [plain @34:15..34:25]: "apps/v1" [plain @34:27..34:34]
                    "kind" [plain @35:15..35:19]: "Deployment" [plain @35:21..35:31]
                    "name" [plain @36:15..36:19]: "istio-ingressgateway" [plain @36:21..36:41]
                    "patches" [plain @37:15..37:22]: sequence [block]
                      - mapping [block]
                        "path" [plain @38:19..38:23]: "spec.template.spec.volumes.[name:workload-socket]" [plain @38:25..38:74]
                        "value" [plain @39:19..39:24]: mapping [block]
                          "name" [plain @40:21..40:25]: "workload-socket" [plain @40:27..40:42]
                          "csi" [plain @41:21..41:24]: mapping [block]
                            "driver" [plain @42:23..42:29]: "csi.spiffe.io" [double-quoted @42:31..42:46]
                            "readOnly" [plain @43:23..43:31]: "true" [plain @43:33..43:37]
                      - mapping [block]
                        "path" [plain @44:19..44:23]: "spec.template.spec.containers.[name:istio-proxy].volumeMounts.[name:workload-socket]" [plain @44:25..44:109]
                        "value" [plain @45:19..45:24]: mapping [block]
                          "name" [plain @46:21..46:25]: "workload-socket" [plain @46:27..46:42]
                          "mountPath" [plain @47:21..47:30]: "/run/secrets/workload-spiffe-uds" [double-quoted @47:32..47:66]
                          "readOnly" [plain @48:21..48:29]: "true" [plain @48:31..48:35]
                      - mapping [block]
                        "path" [plain @49:19..49:23]: "spec.template.spec.initContainers" [plain @49:25..49:58]
                        "value" [plain @50:19..50:24]: sequence [block]
                          - mapping [block]
                            "name" [plain @51:23..51:27]: "wait-for-spire-socket" [plain @51:29..51:50]
                            "image" [plain @52:23..52:28]: "docker.io/library/busybox:1.28" [plain @52:30..52:60]
                            "volumeMounts" [plain @53:23..53:35]: sequence [block]
                              - mapping [block]
                                "name" [plain @54:27..54:31]: "workload-socket" [plain @54:33..54:48]
                                "mountPath" [plain @55:27..55:36]: "/run/secrets/workload-spiffe-uds" [plain @55:38..55:70]
                                "readOnly" [plain @56:27..56:35]: "true" [plain @56:37..56:41]
                            "env" [plain @57:23..57:26]: sequence [block]
                              - mapping [block]
                                "name" [plain @58:27..58:31]: "CHECK_FILE" [plain @58:33..58:43]
                                "value" [plain @59:27..59:32]: "/run/secrets/workload-spiffe-uds/socket" [plain @59:34..59:73]
                            "command" [plain @60:23..60:30]: sequence [block]
                              - "sh" [plain @61:27..61:29]
                              - "-c" [double-quoted @62:27..62:31]
                              - literal [@63:27..70:1]
                                "echo `date -Iseconds` Waiting for: ${CHECK_FILE}\n"
                                "while [[ ! -e ${CHECK_FILE} ]] ; do\n"
                                "  echo `date -Iseconds` File does not exist: ${CHECK_FILE}\n"
                                "  sleep 15\n"
                                "done\n"
                                "ls -l ${CHECK_FILE}"
    "#);
}
