#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("spring-projects-spring-boot-smoke-test-spring-boot-smoke-test-profile-src-main-resources-application-9e2ba205"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "test.name" [plain @1:1..1:10]: "Phil" [plain @1:12..1:16]

    document 2 (start: ---, end: implicit)
    mapping [block]
      "spring.config.activate.on-profile" [plain @5:1..5:34]: "goodbye | dev" [plain @5:36..5:49]
      "test.name" [plain @6:1..6:10]: "Everyone" [plain @6:12..6:20]
    "#);
}
