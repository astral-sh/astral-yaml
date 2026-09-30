#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("docker-awesome-compose-plex-compose-82a8ac11"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "services" [plain @1:1..1:9]: mapping [block]
        "plex" [plain @2:3..2:7]: mapping [block]
          "image" [plain @3:5..3:10]: "linuxserver/plex" [plain @3:12..3:28]
          "container_name" [plain @4:5..4:19]: "plex" [plain @4:21..4:25]
          "network_mode" [plain @5:5..5:17]: "host" [plain @5:19..5:23]
          "environment" [plain @6:5..6:16]: sequence [block]
            - "VERSION=docker" [plain @7:9..7:23]
          "restart" [plain @8:5..8:12]: "always" [plain @8:14..8:20]
          "volumes" [plain @9:5..9:12]: sequence [block]
            - "${PLEX_MEDIA_PATH}:/media/" [plain @10:9..10:35]
    "#);
}
