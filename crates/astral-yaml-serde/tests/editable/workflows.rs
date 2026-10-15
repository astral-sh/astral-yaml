use std::collections::BTreeMap;

use astral_yaml_dev::edit_diff;
use astral_yaml_serde::Editable;
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum Permission {
    Read,
    Write,
}

#[derive(Deserialize)]
struct Workflow {
    permissions: BTreeMap<String, Permission>,
}

#[test]
fn repeated_permission_updates_preserve_the_workflow() {
    let source = include_str!("../../../../fixtures/real-world/compose-ci.yaml");
    let mut document = Editable::<Workflow>::parse(source).unwrap();
    let path = ["permissions", "contents"];
    for (text, expected) in [
        ("write", Permission::Write),
        ("read", Permission::Read),
        ("write", Permission::Write),
    ] {
        for repeat in [false, true] {
            let before = document.source().to_owned();
            document.set_at(path.as_slice(), text).unwrap();
            assert_eq!(document.value().permissions["contents"], expected);
            if repeat {
                assert_eq!(
                    document.source(),
                    before,
                    "repeating a setter retains source"
                );
            }
        }
        let before = document.source().to_owned();
        assert!(document.set_at(path, "admin").is_err());
        assert_eq!(document.source(), before);
        assert_eq!(document.value().permissions["contents"], expected);
    }
    insta::assert_snapshot!(edit_diff(source, document.source()), @"
    @@ -1,7 +1,7 @@
     name: ci
     ⏎
     permissions:
    -  contents: read # to fetch code (actions/checkout)
    +  contents: write # to fetch code (actions/checkout)
     ⏎
     concurrency:
       group: ${{ github.workflow }}-${{ github.ref }}
    ");
}

#[derive(Deserialize)]
struct Compose {
    services: BTreeMap<String, Service>,
}

#[derive(Deserialize)]
struct Service {
    healthcheck: Option<Healthcheck>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Healthcheck {
    interval: String,
    retries: u32,
    timeout: Option<String>,
}

#[test]
fn repeated_healthcheck_updates_preserve_unmodeled_service_fields() {
    let source = include_str!(
        "../../../../fixtures/real-world/docker-awesome-compose-nginx-flask-mysql-compose-457480dd.yaml"
    );
    let mut document = Editable::<Compose>::parse(source).unwrap();
    for retries in [3, 7, 5] {
        document
            .edit_at(["services", "db", "healthcheck"], |health| {
                let mut health = health.into_mapping()?;
                health.field("retries")?.set("temporary")?;
                health.field("retries")?.set(retries)?;
                health.field("interval")?.set("5s")?;
                health.set("timeout", "5s")?;
                Ok(())
            })
            .unwrap();
        let expected = Healthcheck {
            interval: "5s".to_owned(),
            retries,
            timeout: Some("5s".to_owned()),
        };
        assert_eq!(
            document.value().services["db"].healthcheck.as_ref(),
            Some(&expected)
        );
        let before = document.source().to_owned();
        assert!(
            document
                .set_at(["services", "db", "healthcheck", "retries"], -1)
                .is_err()
        );
        assert_eq!(document.source(), before);
        assert_eq!(
            document.value().services["db"].healthcheck.as_ref(),
            Some(&expected)
        );
    }
    insta::assert_snapshot!(edit_diff(source, document.source()), @r#"
    @@ -8,9 +8,10 @@
         restart: always
         healthcheck:
           test: ['CMD-SHELL', 'mysqladmin ping -h 127.0.0.1 --password="$$(cat /run/secrets/db-password)" --silent']
    -      interval: 3s
    +      interval: 5s
           retries: 5
           start_period: 30s
    +      "timeout": "5s"
         secrets:
           - db-password
         volumes:
    "#);
}

#[test]
fn a_commit_error_identifies_the_alias_use_instead_of_the_edit_path() {
    #[derive(Deserialize)]
    struct Config {
        default: String,
        ports: Vec<u16>,
    }

    let source = "default: &port 80\nports: [*port]\n";
    let mut config = Editable::<Config>::parse(source).unwrap();
    let error = config.set_at(["default"], "http").unwrap_err();

    insta::assert_snapshot!(error, @r#"$["ports"][0]: expected an integer at bytes 15..19"#);
    assert_eq!(config.source(), source);
    assert_eq!(config.value().default, "80");
    assert_eq!(config.value().ports, [80]);
}
