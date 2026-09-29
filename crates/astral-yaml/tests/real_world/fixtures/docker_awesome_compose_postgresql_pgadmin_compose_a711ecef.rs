#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("docker-awesome-compose-postgresql-pgadmin-compose-a711ecef"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "services" [plain @1:1..1:9]: mapping [block]
        "postgres" [plain @2:3..2:11]: mapping [block]
          "container_name" [plain @3:5..3:19]: "postgres" [plain @3:21..3:29]
          "image" [plain @4:5..4:10]: "postgres:latest" [plain @4:12..4:27]
          "environment" [plain @5:5..5:16]: sequence [block]
            - "POSTGRES_USER=${POSTGRES_USER}" [plain @6:9..6:39]
            - "POSTGRES_PASSWORD=${POSTGRES_PW}" [plain @7:9..7:41]
            - "POSTGRES_DB=${POSTGRES_DB}" [plain @8:9..8:35]
          "ports" [plain @9:5..9:10]: sequence [block]
            - "5432:5432" [double-quoted @10:9..10:20]
          "restart" [plain @11:5..11:12]: "always" [plain @11:14..11:20]
        "pgadmin" [plain @13:3..13:10]: mapping [block]
          "container_name" [plain @14:5..14:19]: "pgadmin" [plain @14:21..14:28]
          "image" [plain @15:5..15:10]: "dpage/pgadmin4:latest" [plain @15:12..15:33]
          "environment" [plain @16:5..16:16]: sequence [block]
            - "PGADMIN_DEFAULT_EMAIL=${PGADMIN_MAIL}" [plain @17:9..17:46]
            - "PGADMIN_DEFAULT_PASSWORD=${PGADMIN_PW}" [plain @18:9..18:47]
          "ports" [plain @19:5..19:10]: sequence [block]
            - "5050:80" [double-quoted @20:9..20:18]
          "restart" [plain @21:5..21:12]: "always" [plain @21:14..21:20]
    "#);
}
