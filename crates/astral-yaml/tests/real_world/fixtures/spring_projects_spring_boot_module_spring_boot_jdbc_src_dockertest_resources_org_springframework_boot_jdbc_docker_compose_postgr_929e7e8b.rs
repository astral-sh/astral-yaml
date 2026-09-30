#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("spring-projects-spring-boot-module-spring-boot-jdbc-src-dockertest-resources-org-springframework-boot-jdbc-docker-compose-postgr-929e7e8b"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "services" [plain @1:1..1:9]: mapping [block]
        "database" [plain @2:3..2:11]: mapping [block]
          "image" [plain @3:5..3:10]: "{imageName}" [single-quoted @3:12..3:25]
          "ports" [plain @4:5..4:10]: sequence [block]
            - "5432" [single-quoted @5:9..5:15]
          "environment" [plain @6:5..6:16]: sequence [block]
            - "POSTGRES_USER=myuser" [single-quoted @7:9..7:31]
            - "POSTGRES_DB=mydatabase" [single-quoted @8:9..8:33]
            - "POSTGRES_PASSWORD=secret" [single-quoted @9:9..9:35]
          "labels" [plain @10:5..10:11]: mapping [block]
            "org.springframework.boot.jdbc.parameters" [plain @11:7..11:47]: "ApplicationName=spring+boot" [single-quoted @11:49..11:78]
            "org.springframework.boot.r2dbc.parameters" [plain @12:7..12:48]: "applicationName=spring boot" [single-quoted @12:50..12:79]
    "#);
}
