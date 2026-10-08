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
}

fn main() -> Result<(), astral_yaml_serde::Error> {
    let source = r"timeout: 30 # seconds
servers: [primary]
extra: 'retained'
";
    let mut config = Editable::<Config>::parse(source)?;
    config.edit(|editor| {
        editor.root_mut().field("timeout")?.set(60)?;
        editor.root_mut().field("servers")?.push("backup")?;
        Ok(())
    })?;

    assert_eq!(config.value().timeout, 60);
    assert_eq!(config.value().servers, ["primary", "backup"]);
    print!("{}", config.source());
    Ok(())
}
```

Output:

```yaml
timeout: 60 # seconds
servers: [primary, "backup"]
extra: 'retained'
```

## Roll back a failed edit

A transaction commits only if its output parses and deserializes as `T`. Any
error leaves the document unchanged:

```rust
use astral_yaml_serde::Editable;

fn main() -> Result<(), astral_yaml_serde::Error> {
    let mut ports = Editable::<Vec<u16>>::parse("[80, 443] # ports\n")?;
    let before = ports.to_yaml();
    let result = ports.edit(|editor| {
        let mut items = editor.root_mut();
        items.reborrow().push(8080)?;
        items.item(0)?.set("http")?;
        Ok(())
    });

    assert!(result.is_err());
    assert_eq!(ports.source(), before);
    assert_eq!(ports.value(), &[80, 443]);
    Ok(())
}
```

Loading normalizes CR and CRLF line endings to LF. Edits preserve source outside
rewritten regions, but may change whitespace or scalar spelling inside them.

For raw YAML fragments or arbitrary mapping keys, use `edit_syntax()`. For editing
without a typed view, enable `astral-yaml`'s `editable` feature and use
`astral_yaml::edit::EditableDocument`. The
[configuration example](../crates/astral-yaml-serde/examples/edit-config.rs) also
shows comment and style changes.
