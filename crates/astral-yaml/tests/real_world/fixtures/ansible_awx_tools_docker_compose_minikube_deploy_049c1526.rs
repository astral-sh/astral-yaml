#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("ansible-awx-tools-docker-compose-minikube-deploy-049c1526"), @r#"
    document 1 (start: ---, end: implicit)
    sequence [block]
      - mapping [block]
        "name" [plain @2:3..2:7]: "Deploy Minikube and connect with AWX" [plain @2:9..2:45]
        "hosts" [plain @3:3..3:8]: "localhost" [plain @3:10..3:19]
        "gather_facts" [plain @4:3..4:15]: "true" [plain @4:17..4:21]
        "roles" [plain @5:3..5:8]: sequence [block]
          - mapping [flow]
            "role" [plain @6:8..6:12]: "minikube" [plain @6:14..6:22]
    "#);
}
