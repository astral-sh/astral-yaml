#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("argoproj-argo-cd-applicationset-examples-git-generator-directory-excludes-cluster-addons-argo-workflows-kustomization-23308e1d"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "resources" [plain @3:1..3:10]: sequence [block]
        - "https://github.com/argoproj/argo-workflows/releases/download/v3.4.0/namespace-install.yaml" [plain @4:3..4:93]
      "apiVersion" [plain @5:1..5:11]: "kustomize.config.k8s.io/v1beta1" [plain @5:13..5:44]
      "kind" [plain @6:1..6:5]: "Kustomization" [plain @6:7..6:20]
    "#);
}
