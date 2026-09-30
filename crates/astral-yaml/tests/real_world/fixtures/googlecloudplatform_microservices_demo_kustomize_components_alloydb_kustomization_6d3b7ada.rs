#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("googlecloudplatform-microservices-demo-kustomize-components-alloydb-kustomization-6d3b7ada"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @15:1..15:11]: "kustomize.config.k8s.io/v1alpha1" [plain @15:13..15:45]
      "kind" [plain @16:1..16:5]: "Component" [plain @16:7..16:16]
      "patches" [plain @17:1..17:8]: sequence [block]
        - mapping [block]
          "patch" [plain @21:3..21:8]: literal [@21:10..44:1]
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
            "            $patch: delete\n"
            "          - name: ALLOYDB_PRIMARY_IP\n"
            "            value: ALLOYDB_PRIMARY_IP_VAL\n"
            "          - name: ALLOYDB_DATABASE_NAME\n"
            "            value: ALLOYDB_CARTS_DATABASE_NAME_VAL\n"
            "          - name: ALLOYDB_TABLE_NAME\n"
            "            value: ALLOYDB_CARTS_TABLE_NAME_VAL\n"
            "          - name: ALLOYDB_SECRET_NAME\n"
            "            value: ALLOYDB_SECRET_NAME_VAL\n"
            "          - name: PROJECT_ID\n"
            "            value: PROJECT_ID_VAL"
        - mapping [block]
          "patch" [plain @45:3..45:8]: literal [@45:10..52:1]
            "apiVersion: v1\n"
            "kind: ServiceAccount\n"
            "metadata:\n"
            "  name: cartservice\n"
            "  annotations:\n"
            "    iam.gke.io/gcp-service-account: ALLOYDB_USER_GSA_ID"
        - mapping [block]
          "patch" [plain @53:3..53:8]: literal [@53:10..78:1]
            "apiVersion: apps/v1\n"
            "kind: Deployment\n"
            "metadata:\n"
            "  name: productcatalogservice\n"
            "spec:\n"
            "  template:\n"
            "    spec:\n"
            "      containers:\n"
            "        - name: server\n"
            "          env:\n"
            "          - name: ALLOYDB_CLUSTER_NAME\n"
            "            value: ALLOYDB_CLUSTER_NAME_VAL\n"
            "          - name: ALLOYDB_INSTANCE_NAME\n"
            "            value: ALLOYDB_INSTANCE_NAME_VAL\n"
            "          - name: ALLOYDB_DATABASE_NAME\n"
            "            value: ALLOYDB_PRODUCTS_DATABASE_NAME_VAL\n"
            "          - name: ALLOYDB_TABLE_NAME\n"
            "            value: ALLOYDB_PRODUCTS_TABLE_NAME_VAL\n"
            "          - name: ALLOYDB_SECRET_NAME\n"
            "            value: ALLOYDB_SECRET_NAME_VAL\n"
            "          - name: PROJECT_ID\n"
            "            value: PROJECT_ID_VAL\n"
            "          - name: REGION\n"
            "            value: REGION_VAL"
        - mapping [block]
          "patch" [plain @79:3..79:8]: literal [@79:10..86:1]
            "apiVersion: v1\n"
            "kind: ServiceAccount\n"
            "metadata:\n"
            "  name: productcatalogservice\n"
            "  annotations:\n"
            "    iam.gke.io/gcp-service-account: ALLOYDB_USER_GSA_ID"
        - mapping [block]
          "patch" [plain @87:3..87:8]: literal [@87:10..93:1]
            "apiVersion: apps/v1\n"
            "kind: Deployment\n"
            "metadata:\n"
            "  name: redis-cart\n"
            "$patch: delete"
        - mapping [block]
          "patch" [plain @94:3..94:8]: literal [@94:10..100:1]
            "apiVersion: v1\n"
            "kind: Service\n"
            "metadata:\n"
            "  name: redis-cart\n"
            "$patch: delete"
    "#);
}
