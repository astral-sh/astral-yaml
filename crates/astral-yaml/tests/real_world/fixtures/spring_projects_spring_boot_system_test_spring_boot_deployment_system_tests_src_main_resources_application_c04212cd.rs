#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("spring-projects-spring-boot-system-test-spring-boot-deployment-system-tests-src-main-resources-application-c04212cd"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "management.endpoints.web.exposure.include" [plain @1:1..1:42]: "*" [single-quoted @1:44..1:47]
    "#);
}
