# Editing YAML

Use `EditableYaml` to edit a YAML document without defining a Rust type for
it. If your application already reads the document into a type `T`, use
`Editable<T>` to check that the document still deserializes as `T` after each
edit. Call `source()` on either editor to get the YAML to save.

Enable the `editable` feature on `astral-yaml-serde`. The typed example also
uses Serde's derive feature.

```toml
[dependencies]
astral-yaml-serde = { version = "0.1.0", features = ["editable"] }
serde = { version = "1", features = ["derive"] }
```

## Change an existing field

To change a service's image, parse the YAML and pass the field's path and new
value to `set_at()`.

```rust
use astral_yaml_serde::EditableYaml;

fn main() -> Result<(), astral_yaml_serde::Error> {
    let source = "image: 'api:v1' # release\nports: [80]\n";
    let mut service = EditableYaml::parse(source)?;
    service.set_at(["image"], "api:v2")?;
    print!("{}", service.source());
    Ok(())
}
```

The image keeps its quotes and comment.

```yaml
image: 'api:v2' # release
ports: [80]
```

Pass the Rust value you want the field to contain. For example,
`service.set_at(["ports"], [80, 443])?` writes a sequence. Passing
`"[80, 443]"` writes a string.

Each string in a path is one key. For example, `["service.db"]` selects a
literal key containing a dot, while `["service", "db"]` selects `db` inside
`service`. To mix keys and zero-based sequence indices, import
`astral_yaml_serde::edit::edit_path` and use a path like
`edit_path!["containers", 0, "image"]` for the first container's image.
`set_at()` returns an error if the path is missing. To add a key, edit its
parent mapping as in the next example. Both editors support the same methods.

## Edit a typed configuration

Suppose you want to add a port and supply a retry count if one is missing.
Define a type for the fields you use. The `edit()` callback starts at the
document root, so `root.into_mapping()?` gives you access to the service's
fields.

```rust
use astral_yaml_serde::Editable;
use serde::Deserialize;

#[derive(Deserialize)]
struct Service {
    ports: Vec<u16>,
    #[serde(default)]
    retries: u8,
}

fn main() -> Result<(), astral_yaml_serde::Error> {
    let source = "image: 'api:v2' # release\nports: [80]\n";
    let mut service = Editable::<Service>::parse(source)?;
    service.edit(|root| {
        let mut fields = root.into_mapping()?;
        let mut ports = fields.field("ports")?.into_sequence()?;
        ports.push(443)?;
        fields.entry("retries")?.or_insert(3)?;
        Ok(())
    })?;

    assert_eq!(service.value().ports, [80, 443]);
    assert_eq!(service.value().retries, 3);
    print!("{}", service.source());
    Ok(())
}
```

```yaml
image: 'api:v2' # release
ports: [80, 443]
"retries": 3
```

`#[serde(default)]` lets `Service` read a file with no `retries` field.
`or_insert(3)` writes the default into the YAML only when the key is missing.
To set the retry count even when one exists, use `fields.set("retries", 3)?`.

The example edits individual fields, so the YAML keeps `image` even though
`Service` omits it. If you replace a whole mapping, include every field you
want to keep. `Editable<T>` requires `T` to implement `DeserializeOwned`.
`Service` needs neither `Serialize` nor `Clone`.

## Failed edits

`edit()` commits only if the callback succeeds and the resulting document
deserializes as `T`. Here the callback adds a valid port, then sets the first
port to `"http"`. The string cannot deserialize as a `u16`, so the editor
discards both changes.

```rust
use astral_yaml_serde::Editable;

fn main() -> Result<(), astral_yaml_serde::Error> {
    let source = "[80] # ports\n";
    let mut ports = Editable::<Vec<u16>>::parse(source)?;
    let result = ports.edit(|root| {
        let mut items = root.into_sequence()?;
        items.push(443)?;
        items.item(0)?.set("http")?;
        Ok(())
    });

    assert_eq!(result.unwrap_err().path.as_deref(), Some("$[0]"));
    assert_eq!(ports.source(), source);
    assert_eq!(ports.value(), &[80]);
    Ok(())
}
```

Inside a callback, `read()` sees earlier edits. You can handle a failed read
or lookup and continue editing. A failed mutation prevents the commit even if
you catch the error.

Use `try_edit()` to return your own error type from the callback. It must
implement `From<astral_yaml_serde::Error>`. The editor passes application
errors through and returns a successful result only after committing.

## Source preservation

Parsing converts CR and CRLF line endings to LF. Edits preserve text outside
the regions they rewrite. Within those regions, spacing and scalar spelling
can change.

To add a leading comment to a node that already has an inline comment, use
`node.insert_comment(0, " explanation", CommentPlacement::Leading)?`. The
existing inline comment stays on its line.

The configuration example uses `update()` to increment a timeout, adds a
comment, and switches a sequence to block style. Run it from the repository
root. [Example source](../crates/astral-yaml-serde/examples/edit-config.rs)

```sh
cargo run -p astral-yaml-serde --features editable --example edit-config
```

## Aliases and non-string keys

Paths stop at aliases. Given `default_port: &port 80` and `port: *port`,
reading `port` returns `80`. Setting `port` to `443` replaces just the alias.
To change the shared value, edit `default_port`. An edit fails if a surviving
alias would lose its anchor or point to a different node. When deserialization
fails at an alias, the error path identifies its use in the proposed document
and the byte span can point to its target.

If a mapping has non-string keys, use `key_at()` and `value_at()` to select
its keys and values by index. Use `remove_at(index)` to delete the entry.
Indices follow the current entry order, including earlier edits in the callback.

Both Serde editors check for duplicate keys and limit alias expansion.
`parse_with_options()` configures the separate parser limits for loading and
editing.
