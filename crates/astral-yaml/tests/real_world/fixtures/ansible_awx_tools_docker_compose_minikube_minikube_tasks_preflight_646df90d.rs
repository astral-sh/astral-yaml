#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("ansible-awx-tools-docker-compose-minikube-minikube-tasks-preflight-646df90d"), @r#"
    document 1 (start: ---, end: implicit)
    sequence [block]
      - mapping [block]
        "name" [plain @2:3..2:7]: "Check if Kubernetes python module is installed" [plain @2:9..2:55]
        "shell" [plain @3:3..3:8]: "pip freeze | grep openshift" [plain @3:10..3:37]
        "register" [plain @4:3..4:11]: "_pip_openshift" [plain @4:13..4:27]
        "ignore_errors" [plain @5:3..5:16]: "true" [plain @5:18..5:22]
      - mapping [block]
        "name" [plain @7:3..7:7]: "Preflight check - require openshift python module" [plain @7:9..7:58]
        "assert" [plain @8:3..8:9]: mapping [block]
          "fail_msg" [plain @9:5..9:13]: "The openshift python module was not found. You can either install it via 'pip install openshift' or using your distro package manager." [double-quoted @9:15..9:151]
          "that" [plain @10:5..10:9]: sequence [block]
            - "_pip_openshift.rc == 0" [plain @11:9..11:31]
            - "'openshift==' in _pip_openshift.stdout" [double-quoted @12:9..12:49]
    "#);
}
