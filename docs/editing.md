# Editing YAML

Enable the `editable` feature:

```toml
[dependencies]
astral-yaml-serde = { version = "0.1.0", features = ["editable"] }
serde = { version = "1", features = ["derive"] }
```

## Update a configuration

`Editable<T>` keeps the YAML source alongside its deserialized value. Edits retain
comments and fields absent from `T`:

```rust
use astral_yaml_serde::Editable;
use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    timeout: u64,
    servers: Vec<String>,
    #[serde(default)]
    retries: u8,
}

fn main() -> Result<(), astral_yaml_serde::Error> {
    let source = r"timeout: 30 # seconds
servers: [primary]
extra: 'retained'
";
    let mut config = Editable::<Config>::parse(source)?;
    config.edit(|editor| {
        let mut settings = editor.root_mut().into_mapping()?;
        settings.field("timeout")?.set(60)?;
        settings.entry("retries")?.or_insert(3)?;

        let mut servers = settings.field("servers")?.into_sequence()?;
        servers.push("backup")?;
        servers.push("archive")?;
        Ok(())
    })?;

    assert_eq!(config.value().timeout, 60);
    assert_eq!(config.value().servers, ["primary", "backup", "archive"]);
    assert_eq!(config.value().retries, 3);
    print!("{}", config.source());
    Ok(())
}
```

Output:

```yaml
timeout: 60 # seconds
servers: [primary, "backup", "archive"]
extra: 'retained'
"retries": 3
```

Collection views let you edit siblings without selecting their parent again.
Navigate to a nested collection with a chain such as
`editor.root_mut().field("services")?.field("db")?.into_mapping()?`.
Each view lends a cursor for a child while retaining its own position.

`field()` requires a string key to exist, while `get()` returns `None` when it is
absent. `entry(name)?.or_insert(value)?` supplies an optional default. Existing
entries stay unchanged and do not serialize the default. Use `set(name, value)`
on a mapping view to replace an existing value or append a new entry.

## Roll back a failed edit

A transaction commits only if its output parses and deserializes as `T`. Any
error leaves the document unchanged:

```rust
use astral_yaml_serde::Editable;

fn main() -> Result<(), astral_yaml_serde::Error> {
    let mut ports = Editable::<Vec<u16>>::parse("[80, 443] # ports\n")?;
    let before = ports.to_yaml();
    let result = ports.edit(|editor| {
        let mut items = editor.root_mut().into_sequence()?;
        items.push(8080)?;
        items.item(0)?.set("http")?;
        Ok(())
    });

    assert!(result.is_err());
    assert_eq!(ports.source(), before);
    assert_eq!(ports.value(), &[80, 443]);
    Ok(())
}
```

Mutation failures also prevent commit when the callback handles their errors.
Failed selections leave the transaction usable, so you can try another field.

## Use an application error

`try_edit()` accepts errors that implement `From<astral_yaml_serde::Error>`.
Application errors retain their payload, and the callback's successful return
value is returned only after the edit commits:

```rust
use astral_yaml_serde::{Editable, Error};

#[derive(Debug)]
enum UpdateError {
    Yaml(Error),
    Rejected(String),
}

impl From<Error> for UpdateError {
    fn from(error: Error) -> Self {
        Self::Yaml(error)
    }
}

fn main() -> Result<(), Error> {
    let mut ports = Editable::<Vec<u16>>::parse("[80, 443]\n")?;
    let before = ports.to_yaml();
    let result: Result<(), UpdateError> = ports.try_edit(|editor| {
        editor.root_mut().into_sequence()?.push(8080)?;
        Err(UpdateError::Rejected("port is already reserved".to_owned()))
    });

    match result {
        Err(UpdateError::Rejected(reason)) => assert_eq!(reason, "port is already reserved"),
        Err(UpdateError::Yaml(error)) => return Err(error),
        Ok(()) => panic!("the application rejected this edit"),
    }
    assert_eq!(ports.source(), before);
    assert_eq!(ports.value(), &[80, 443]);
    Ok(())
}
```

## Edit a selected node with YAML

Use `set_yaml()` when a fragment should keep the selected node's anchor and
compatible style. `replace_yaml()` uses the fragment's style and anchor. Both
preserve outer comments and still check the final typed value:

```rust
use astral_yaml_serde::Editable;
use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    name: String,
    ports: Vec<u16>,
}

fn main() -> Result<(), astral_yaml_serde::Error> {
    let mut config = Editable::<Config>::parse("name: 'primary' # service\nports: [80]\n")?;
    config.edit(|editor| {
        let mut settings = editor.root_mut().into_mapping()?;
        settings.field("name")?.set_yaml("\"backup\"")?;
        settings.field("ports")?.replace_yaml("- 8080\n- 8443\n")?;
        Ok(())
    })?;

    assert_eq!(config.value().name, "backup");
    assert_eq!(config.value().ports, [8080, 8443]);
    assert!(config.source().contains("name: 'backup' # service"));
    print!("{}", config.source());
    Ok(())
}
```

Loading normalizes CR and CRLF line endings to LF. Edits preserve source outside
rewritten regions, but may change whitespace or scalar spelling inside them.
`parse_with_options()` sets syntax limits for loading and subsequent edits.
Serde's separate alias-expansion limits still apply.

Use `edit_syntax()` for arbitrary mapping keys and explicit alias editing. For editing
without a typed view, enable `astral-yaml`'s `editable` feature and use
`astral_yaml::edit::EditableDocument`. The
[configuration example](../crates/astral-yaml-serde/examples/edit-config.rs) also
shows comment and style changes.
