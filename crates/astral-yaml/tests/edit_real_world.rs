//! Edits to the pinned corpus retain presentation outside the changed nodes.
//! YAML presentation: <https://yaml.org/spec/1.2.2/#32-information-models>.

use astral_yaml::edit::{EditableDocument, Editor, Handle, NodeKind};
use astral_yaml_dev::edit_diff;

fn mapping_value(editor: &Editor<'_>, path: &[&str]) -> Handle {
    let mut node = editor.root();
    for name in path {
        node = editor
            .mapping_entries(&node)
            .unwrap()
            .into_iter()
            .find(|(key, _)| {
                matches!(editor.node(key).unwrap().kind(), NodeKind::Scalar { value, .. } if value == name)
            })
            .unwrap_or_else(|| panic!("missing fixture key {name}"))
            .1;
    }
    node
}

#[test]
fn ci_permission_edit_preserves_the_inline_comment() {
    let source = include_str!("../../../fixtures/real-world/compose-ci.yaml");
    let mut document = EditableDocument::parse(source).unwrap();
    document
        .edit(|editor| {
            let contents = mapping_value(editor, &["permissions", "contents"]);
            editor.replace(&contents, "write")
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff(source, document.source()), @"
    +-------------------------------------------------------+
    | --- before.yaml |
    | +++ after.yaml |
    | @@ -1,7 +1,7 @@ |
    |  name: ci |
    |   |
    |  permissions: |
    | -  contents: read # to fetch code (actions/checkout) |
    | +  contents: write # to fetch code (actions/checkout) |
    |   |
    |  concurrency: |
    |    group: ${{ github.workflow }}-${{ github.ref }} |
    +-------------------------------------------------------+
    ");
}

#[test]
fn openapi_server_insertion_preserves_surrounding_sections() {
    let source = include_str!("../../../fixtures/real-world/openapi-petstore.yaml");
    let mut document = EditableDocument::parse(source).unwrap();
    document
        .edit(|editor| {
            let servers = mapping_value(editor, &["servers"]);
            editor.insert_item(&servers, 1, "url: https://staging.example.com/v2")
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff(source, document.source()), @"
    +------------------------------------------------------------+
    | --- before.yaml |
    | +++ after.yaml |
    | @@ -12,7 +12,10 @@ |
    |      name: Apache 2.0 |
    |      url: https://www.apache.org/licenses/LICENSE-2.0.html |
    |  servers: |
    | -  - url: https://petstore.swagger.io/v2 |
    | +  - |
    | +    url: https://petstore.swagger.io/v2 |
    | +  - |
    | +    url: https://staging.example.com/v2 |
    |  paths: |
    |    /pets: |
    |      get: |
    +------------------------------------------------------------+
    ");
}

#[test]
fn compose_healthcheck_edit_preserves_service_comments_and_siblings() {
    let source = include_str!(
        "../../../fixtures/real-world/docker-awesome-compose-nginx-flask-mysql-compose-457480dd.yaml"
    );
    let mut document = EditableDocument::parse(source).unwrap();
    document
        .edit(|editor| {
            let healthcheck = mapping_value(editor, &["services", "db", "healthcheck"]);
            editor.remove_entry(&healthcheck, 2)?;
            editor.insert_entry(&healthcheck, 3, "timeout", "5s")
        })
        .unwrap();
    insta::assert_snapshot!(edit_diff(source, document.source()), @r#"
    +-------------------------------------------------------------------------------------------------------------------+
    | --- before.yaml |
    | +++ after.yaml |
    | @@ -9,8 +9,8 @@ |
    |      healthcheck: |
    |        test: ['CMD-SHELL', 'mysqladmin ping -h 127.0.0.1 --password="$$(cat /run/secrets/db-password)" --silent'] |
    |        interval: 3s |
    | -      retries: 5 |
    |        start_period: 30s |
    | +      timeout: 5s |
    |      secrets: |
    |        - db-password |
    |      volumes: |
    +-------------------------------------------------------------------------------------------------------------------+
    "#);
}
