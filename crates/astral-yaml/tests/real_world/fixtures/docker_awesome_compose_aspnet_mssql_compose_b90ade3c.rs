#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("docker-awesome-compose-aspnet-mssql-compose-b90ade3c"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "services" [plain @1:1..1:9]: mapping [block]
        "web" [plain @2:3..2:6]: mapping [block]
          "build" [plain @3:5..3:10]: "app/aspnetapp" [plain @3:12..3:25]
          "ports" [plain @4:5..4:10]: sequence [block]
            - "80:80" [plain @5:9..5:14]
        "db" [plain @6:3..6:5]: mapping [block]
          "environment" [plain @7:5..7:16]: mapping [block]
            "ACCEPT_EULA" [plain @8:7..8:18]: "Y" [double-quoted @8:20..8:23]
            "SA_PASSWORD" [plain @9:7..9:18]: "example_123" [plain @9:20..9:31]
          "image" [plain @11:5..11:10]: "mcr.microsoft.com/azure-sql-edge:1.0.4" [plain @11:12..11:50]
          "restart" [plain @14:5..14:12]: "always" [plain @14:14..14:20]
          "healthcheck" [plain @15:5..15:16]: mapping [block]
            "test" [plain @16:9..16:13]: sequence [flow]
              - "CMD-SHELL" [double-quoted @16:16..16:27]
              - "/opt/mssql-tools/bin/sqlcmd -S localhost -U sa -P example_123 -Q 'SELECT 1' || exit 1" [double-quoted @16:29..16:116]
            "interval" [plain @17:9..17:17]: "10s" [plain @17:19..17:22]
            "retries" [plain @18:9..18:16]: "10" [plain @18:18..18:20]
            "start_period" [plain @19:9..19:21]: "10s" [plain @19:23..19:26]
            "timeout" [plain @20:9..20:16]: "3s" [plain @20:18..20:20]
    "#);
}
