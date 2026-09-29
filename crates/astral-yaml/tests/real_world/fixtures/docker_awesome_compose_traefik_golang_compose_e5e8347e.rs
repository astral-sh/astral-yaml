#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("docker-awesome-compose-traefik-golang-compose-e5e8347e"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "services" [plain @1:1..1:9]: mapping [block]
        "frontend" [plain @2:3..2:11]: mapping [block]
          "image" [plain @3:5..3:10]: "traefik:2.6" [plain @3:12..3:23]
          "command" [plain @4:5..4:12]: "--providers.docker --entrypoints.web.address=:80 --providers.docker.exposedbydefault=false" [plain @4:14..4:104]
          "ports" [plain @5:5..5:10]: sequence [block]
            - "80:80" [double-quoted @7:9..7:16]
          "volumes" [plain @8:5..8:12]: sequence [block]
            - "/var/run/docker.sock:/var/run/docker.sock" [plain @10:9..10:50]
          "depends_on" [plain @11:5..11:15]: sequence [block]
            - "backend" [plain @12:9..12:16]
        "backend" [plain @13:3..13:10]: mapping [block]
          "build" [plain @14:5..14:10]: "backend" [plain @14:12..14:19]
          "labels" [plain @15:5..15:11]: sequence [block]
            - "traefik.enable=true" [double-quoted @16:9..16:30]
            - "traefik.http.routers.go.rule=Path(`/`)" [double-quoted @17:9..17:49]
            - "traefik.http.services.go.loadbalancer.server.port=80" [double-quoted @18:9..18:63]
    "#);
}
