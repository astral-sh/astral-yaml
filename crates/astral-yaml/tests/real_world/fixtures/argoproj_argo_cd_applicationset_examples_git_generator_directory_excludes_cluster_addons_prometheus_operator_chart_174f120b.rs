#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("argoproj-argo-cd-applicationset-examples-git-generator-directory-excludes-cluster-addons-prometheus-operator-chart-174f120b"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "v2" [plain @1:13..1:15]
      "name" [plain @2:1..2:5]: "helm-prometheus-operator" [plain @2:7..2:31]
      "type" [plain @4:1..4:5]: "application" [plain @4:7..4:18]
      "version" [plain @9:1..9:8]: "0.1.0" [plain @9:10..9:15]
      "appVersion" [plain @14:1..14:11]: "1.0" [double-quoted @14:13..14:18]
    "#);
}
