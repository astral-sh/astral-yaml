#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("docker-awesome-compose-portainer-compose-de1e5a44"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "services" [plain @1:1..1:9]: mapping [block]
        "portainer" [plain @2:3..2:12]: mapping [block]
          "image" [plain @3:5..3:10]: "portainer/portainer-ce:alpine" [plain @3:12..3:41]
          "container_name" [plain @4:5..4:19]: "portainer" [plain @4:21..4:30]
          "command" [plain @5:5..5:12]: "-H unix:///var/run/docker.sock" [plain @5:14..5:44]
          "ports" [plain @6:5..6:10]: sequence [block]
            - "9000:9000" [double-quoted @7:9..7:20]
          "volumes" [plain @8:5..8:12]: sequence [block]
            - "/var/run/docker.sock:/var/run/docker.sock" [double-quoted @9:9..9:52]
            - "portainer_data:/data" [double-quoted @10:9..10:31]
          "restart" [plain @11:5..11:12]: "always" [plain @11:14..11:20]
      "volumes" [plain @13:1..13:8]: mapping [block]
        "portainer_data" [plain @14:3..14:17]: "" [plain @14:18..14:18]
    "#);
}
