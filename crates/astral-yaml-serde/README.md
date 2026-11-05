# astral-yaml-serde

astral-yaml-serde is a [Serde] adapter for [astral-yaml].

[Serde]: https://serde.rs/
[astral-yaml]: https://crates.io/crates/astral-yaml

## Usage

Use astral-yaml-serde to serialize and deserialize like
any other Serde implementation:

```rust
use astral_yaml_serde::{from_str, to_string};

let ports: Vec<u16> = from_str("- 80\n- 443\n").unwrap();
assert_eq!(ports, [80, 443]);
assert_eq!(to_string(&ports).unwrap(), "[80, 443]\n");
```

Note that astral-yaml-serde uses YAML's flow style for
serialization.

See the [documentation] for more information.

[documentation]: https://docs.rs/astral-yaml-serde

## Features

astral-yaml-serde provides a few useful adapter types. Given
`T: Deserialize`:

- `Spanned<T>` deserializes `T` along with its source span in the input.
- `Commented<'a, T>` deserializes `T` along with its comments.

These adapters can be composed, e.g. `Spanned<Commented<'a, T>>`
preserves both source spans and comments during deserialization.

With the `editable` feature enabled, `Editable<T>` provides a high-level
editing API. See the [editing guide] for more information.

[editing guide]: https://github.com/astral-sh/astral-yaml/blob/main/docs/editing.md

<div align="center">
  <a target="_blank" href="https://astral.sh" style="background:none">
    <img src="https://raw.githubusercontent.com/astral-sh/uv/main/assets/svg/Astral.svg" alt="Made by Astral">
  </a>
</div>
