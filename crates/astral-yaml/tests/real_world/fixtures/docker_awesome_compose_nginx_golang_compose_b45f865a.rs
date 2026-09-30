#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("docker-awesome-compose-nginx-golang-compose-b45f865a"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "services" [plain @1:1..1:9]: mapping [block]
        "proxy" [plain @2:3..2:8]: mapping [block]
          "image" [plain @3:5..3:10]: "nginx" [plain @3:12..3:17]
          "volumes" [plain @4:5..4:12]: sequence [block]
            - mapping [block]
              "type" [plain @5:9..5:13]: "bind" [plain @5:15..5:19]
              "source" [plain @6:9..6:15]: "./proxy/nginx.conf" [plain @6:17..6:35]
              "target" [plain @7:9..7:15]: "/etc/nginx/conf.d/default.conf" [plain @7:17..7:47]
              "read_only" [plain @8:9..8:18]: "true" [plain @8:20..8:24]
          "ports" [plain @9:5..9:10]: sequence [block]
            - "80:80" [plain @10:9..10:14]
          "depends_on" [plain @11:5..11:15]: sequence [block]
            - "backend" [plain @12:9..12:16]
        "backend" [plain @14:3..14:10]: mapping [block]
          "build" [plain @15:5..15:10]: mapping [block]
            "context" [plain @16:7..16:14]: "backend" [plain @16:16..16:23]
            "target" [plain @17:7..17:13]: "builder" [plain @17:15..17:22]
    "#);
}
