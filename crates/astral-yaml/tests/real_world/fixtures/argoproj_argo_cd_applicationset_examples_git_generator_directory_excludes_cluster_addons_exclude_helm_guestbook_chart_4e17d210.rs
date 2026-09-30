#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("argoproj-argo-cd-applicationset-examples-git-generator-directory-excludes-cluster-addons-exclude-helm-guestbook-chart-4e17d210"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "apiVersion" [plain @1:1..1:11]: "v2" [plain @1:13..1:15]
      "name" [plain @2:1..2:5]: "helm-guestbook" [plain @2:7..2:21]
      "description" [plain @3:1..3:12]: "A Helm chart for Kubernetes" [plain @3:14..3:41]
      "type" [plain @13:1..13:5]: "application" [plain @13:7..13:18]
      "version" [plain @18:1..18:8]: "0.1.0" [plain @18:10..18:15]
      "appVersion" [plain @23:1..23:11]: "1.0" [double-quoted @23:13..23:18]
    "#);
}
