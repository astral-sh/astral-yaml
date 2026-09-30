#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("googlecloudplatform-microservices-demo-kustomize-components-memorystore-kustomization-34ea78ac"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @15:1..15:11]: "kustomize.config.k8s.io/v1alpha1" [plain @15:13..15:45]
      "kind" [plain @16:1..16:5]: "Component" [plain @16:7..16:16]
      "patches" [plain @17:1..17:8]: sequence [block]
        - mapping [block]
          "patch" [plain @19:3..19:8]: literal [@19:10..32:1]
            "apiVersion: apps/v1\n"
            "kind: Deployment\n"
            "metadata:\n"
            "  name: cartservice\n"
            "spec:\n"
            "  template:\n"
            "    spec:\n"
            "      containers:\n"
            "        - name: server\n"
            "          env:\n"
            "          - name: REDIS_ADDR\n"
            "            value: \"REDIS_CONNECTION_STRING\""
        - mapping [block]
          "patch" [plain @33:3..33:8]: literal [@33:10..39:1]
            "apiVersion: apps/v1\n"
            "kind: Deployment\n"
            "metadata:\n"
            "  name: redis-cart\n"
            "$patch: delete"
        - mapping [block]
          "patch" [plain @40:3..40:8]: literal [@40:10..46:1]
            "apiVersion: v1\n"
            "kind: Service\n"
            "metadata:\n"
            "  name: redis-cart\n"
            "$patch: delete"
    "#);
}
