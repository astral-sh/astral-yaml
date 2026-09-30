#[test]
fn snapshot() {
    insta::assert_snapshot!(crate::check_fixture("docker-awesome-compose-minecraft-compose-41173892"), @r#"
    document 1 (start: implicit, end: implicit)
    mapping [block]
      "services" [plain @1:1..1:9]: mapping [block]
        "minecraft" [plain @2:2..2:11]: mapping [block]
          "image" [plain @3:4..3:9]: "itzg/minecraft-server" [plain @3:11..3:32]
          "ports" [plain @4:4..4:9]: sequence [block]
            - "25565:25565" [double-quoted @5:8..5:21]
          "environment" [plain @6:4..6:15]: mapping [block]
            "EULA" [plain @7:6..7:10]: "TRUE" [double-quoted @7:12..7:18]
          "deploy" [plain @8:4..8:10]: mapping [block]
            "resources" [plain @9:6..9:15]: mapping [block]
              "limits" [plain @10:8..10:14]: mapping [block]
                "memory" [plain @11:10..11:16]: "1.5G" [plain @11:18..11:22]
          "volumes" [plain @12:4..12:11]: sequence [block]
            - "~/minecraft_data:/data" [double-quoted @13:8..13:32]
    "#);
}
