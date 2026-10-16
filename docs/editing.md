# Editing YAML

Enable the `editable` feature on `astral-yaml-serde`. The typed examples also
use Serde's derive feature:

```toml
[dependencies]
astral-yaml-serde = { version = "0.1.0", features = ["editable"] }
serde = { version = "1", features = ["derive"] }
```

`EditableYaml` edits a document without requiring a Rust type for the whole
document. It still checks the document using Serde's YAML rules, including
duplicate-key checks and alias limits. To change an existing value, pass its
path to `set_at()`:

```rust
use astral_yaml_serde::{EditableYaml, edit::edit_path};

fn main() -> Result<(), astral_yaml_serde::Error> {
    let mut deployment = EditableYaml::parse("containers:\n  - image: 'api:v1' # rollout\n")?;
    deployment.set_at(edit_path!["containers", 0, "image"], "api:v2")?;
    assert_eq!(
        deployment.source(),
        "containers:\n  - image: 'api:v2' # rollout\n"
    );
    Ok(())
}
```

Path keys are literal strings and sequence indices are zero-based. For example,
`["service.db"]` selects one key, while `["service", "db"]` selects a nested
value. Use `edit_path!` when a path mixes keys and indices. A missing path is an
error, so `set_at()` will not create a field for a misspelled key.

## Edit a typed configuration

`Editable<T>` keeps the YAML source and its deserialized value in sync. `T` must
implement `DeserializeOwned`, but does not need to implement `Serialize` or
`Clone`. An `edit()` callback starts at the root and can make several changes in
one transaction:

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
    let source = "timeout: 30 # seconds\nservers: [primary]\nextra: 'retained'\n";
    let mut config = Editable::<Config>::parse(source)?;
    config.edit(|root| {
        let mut settings = root.into_mapping()?;
        settings
            .field("timeout")?
            .update(|seconds: u64| seconds + 30)?;
        settings.entry("retries")?.or_insert(3)?;
        settings.field("servers")?.into_sequence()?.push("backup")?;
        Ok(())
    })?;

    assert_eq!(config.value().timeout, 60);
    assert_eq!(config.value().servers, ["primary", "backup"]);
    assert_eq!(config.value().retries, 3);
    print!("{}", config.source());
    Ok(())
}
```

The output retains the comment and the field not modeled by `Config`:

```yaml
timeout: 60 # seconds
servers: [primary, "backup"]
extra: 'retained'
"retries": 3
```

The mapping view lets the callback edit sibling fields without navigating from
the root each time. `field()` requires the string key to exist, while `get()`
returns `None` for a missing key. `entry(name)?.or_insert(value)?` inserts a
default only when the key is absent. An existing entry is unchanged and the
default is not serialized.

To replace or add an entry, use `set(name, value)` on the mapping view. Use
`ensure_mapping()` or `ensure_sequence()` to edit a collection that may be
absent. They create an empty collection for a missing key and return an error if
an existing value is not the requested collection type. `key_at()` and
`value_at()` select the key or value of an existing entry by index, including
entries with non-string keys.

`update()` reads, transforms, and replaces the selected value. Editing the field
itself preserves the rest of the document, including fields absent from `T`. If
you replace a whole mapping, include any fields you want to retain in the
replacement. Use `edit_at()` to start a callback at a selected node, or `read()`
to inspect a value before changing it. Reads see earlier changes in the
transaction.

## Failed edits

An edit commits only if the callback succeeds and the resulting YAML
deserializes as `T`. A failure leaves both the source and the typed value
unchanged:

```rust
use astral_yaml_serde::Editable;

fn main() -> Result<(), astral_yaml_serde::Error> {
    let mut ports = Editable::<Vec<u16>>::parse("[80, 443] # ports\n")?;
    let result = ports.edit(|root| {
        let mut items = root.into_sequence()?;
        items.push(8080)?;
        items.item(0)?.set("http")?;
        Ok(())
    });

    assert_eq!(result.unwrap_err().path.as_deref(), Some("$[0]"));
    assert_eq!(ports.source(), "[80, 443] # ports\n");
    assert_eq!(ports.value(), &[80, 443]);
    Ok(())
}
```

A failed mutation prevents commit even if the callback handles the error. A
failed selection or read does not, so the callback can try another field.
Deserialization errors report a path in the proposed document. If the path
follows an alias, the byte span can point to the anchor it references.

`try_edit()` lets the callback return an application error that implements
`From<astral_yaml_serde::Error>`. Application errors are returned unchanged. A
successful callback result is returned only after commit.

## YAML fragments and source preservation

Use `set_yaml()` to replace a node with a YAML fragment while retaining the
node's anchor and compatible style. Use `replace_yaml()` to take the fragment's
anchor and style instead. Both retain the selected node's outer comments:

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
    assert_eq!(
        config.source(),
        "name: 'backup' # service\nports:\n  - 8080\n  - 8443\n"
    );
    Ok(())
}
```

Loading normalizes CR and CRLF line endings to LF. Edits preserve text outside
rewritten regions, but may change whitespace or scalar spelling within them.
`parse_with_options()` sets syntax limits for loading and subsequent edits.
Serde's separate alias-expansion limits still apply.

Navigation does not follow aliases. Reading an alias follows its target, while
setting or updating that position replaces the alias itself. Commit fails if an
alias loses its anchor or changes its target.

The [configuration example](../crates/astral-yaml-serde/examples/edit-config.rs)
also shows comment and style changes.

Use `edit_syntax()` for operations that need node handles, such as inserting an
entry with a non-string key or explicitly editing an alias. For syntax-only
editing without Serde's checks, enable `astral-yaml`'s `editable` feature and
use `astral_yaml::edit::EditableDocument`.
