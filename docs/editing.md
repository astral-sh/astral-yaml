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
    config.edit(|root| {
        let mut settings = root.into_mapping()?;
        settings.field("timeout")?.update(|seconds: u64| seconds + 30)?;
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

The callback starts at the root. To start at an existing field, write
`config.edit_at(["timeout"], |mut timeout| timeout.set(60))?`.
Paths use literal keys, so `["service.db"]` selects one key and
`["service", "db"]` selects a nested field. Collection views let you edit
siblings without selecting their parent again.

`update()` reads the current value, transforms it, and writes the result at the
selected position. Update the field itself to preserve surrounding YAML, including
fields your Rust type doesn't model. For a conditional edit, read the pending
value directly:

```rust
use astral_yaml_serde::{Editable, Error};

fn main() -> Result<(), Error> {
    let mut servers = Editable::<Vec<String>>::parse("['primary', 'retired'] # failover\n")?;
    servers.edit_at([1], |mut server| {
        if server.read::<&str>()? == "retired" {
            server.set("backup")?;
        }
        Ok(())
    })?;
    assert_eq!(servers.source(), "['primary', 'backup'] # failover\n");
    Ok(())
}
```

Reads see earlier edits in the same transaction. Reading an alias follows its
target, while setting or updating that position replaces the alias itself.

`field()` requires a string key to exist, while `get()` returns `None` when it is
absent. `entry(name)?.or_insert(value)?` supplies an optional default. Existing
entries stay unchanged and do not serialize the default. Use `set(name, value)`
on a mapping view to replace an existing value or append a new entry.
To edit a collection that might be absent, use `entry("servers")?.or_sequence()?`
or `entry("settings")?.or_mapping()?`. These insert an empty collection when
needed and report an error if an existing value has the wrong type.

## Roll back a failed edit

A transaction commits only if its output parses and deserializes as `T`. Any
error leaves the document unchanged:

```rust
use astral_yaml_serde::Editable;

fn main() -> Result<(), astral_yaml_serde::Error> {
    let mut ports = Editable::<Vec<u16>>::parse("[80, 443] # ports\n")?;
    let before = ports.to_yaml();
    let result = ports.edit(|root| {
        let mut items = root.into_sequence()?;
        items.push(8080)?;
        items.item(0)?.set("http")?;
        Ok(())
    });

    assert_eq!(result.unwrap_err().path.as_deref(), Some("$[0]"));
    assert_eq!(ports.source(), before);
    assert_eq!(ports.value(), &[80, 443]);
    Ok(())
}
```

Mutation failures also prevent commit when the callback handles their errors.
Failed selections and reads leave the transaction usable, so you can try another field.
Deserialization errors report a path in the proposed document. If that path
follows an alias, the byte span can point to the anchor it references.

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
    let result: Result<(), UpdateError> = ports.try_edit(|root| {
        root.into_sequence()?.push(8080)?;
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
    config.edit(|root| {
        let mut settings = root.into_mapping()?;
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
