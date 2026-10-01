# Limitations

See [resource limits](resource-limits.md) for input and processing limits.

## Serde buffering

Serde buffers values for untagged enums, internally tagged enums, and flattened
fields. If a plain scalar resolves as a number, boolean, or null, the buffer
loses its text and cannot deserialize it into a string. Quote the scalar or use
`!!str` to preserve it as a string in these contexts.

`Spanned` and `Tagged` require direct access to the YAML deserializer; Serde's
intermediate buffers cannot retain their metadata. For `Spanned`, wrap the
containing enum or struct instead of a field decoded from the buffer.

## Mapping keys

Mapping-key comparison uses 128-bit integer magnitudes and 64-bit floats,
independently of the destination key type. Comparison includes application tags.
Application-tagged scalar keys are compared by their tag and scalar text; the
adapter cannot infer a custom tag's canonical form. Applications that assign
additional equivalences to these keys must check them when constructing values.

The adapter rejects duplicate YAML keys before deserializing a mapping. Serde's
`MapAccess` interface does not expose the destination key type's equality to the
deserializer, so it cannot detect collisions between distinct YAML keys that
deserialize into equal Rust keys. For example, in `{1: first, "1": second}`,
the keys are a YAML integer and string, but both deserialize to `"1"` in a
`BTreeMap<String, String>`. Serde's `BTreeMap` and `HashMap` implementations keep
the last value, so the result contains only `"1": "second"`.

To reject collisions between deserialized keys, apply a map adapter such as
[`serde_with::MapPreventDuplicates`](https://docs.rs/serde_with/latest/serde_with/struct.MapPreventDuplicates.html)
to each map that needs the check:

```rust
use serde::Deserialize;
use serde_with::{MapPreventDuplicates, serde_as};
use std::collections::BTreeMap;

#[serde_as]
#[derive(Deserialize)]
struct Config {
    #[serde_as(as = "MapPreventDuplicates<_, _>")]
    values: BTreeMap<String, String>,
}

let source = r#"values: {1: first, "1": second}"#;
assert!(astral_yaml_serde::from_str::<Config>(source).is_err());
```

## Numeric values

Numeric values must fit the requested Rust type. The parser can still recognize
numeric syntax outside machine integer ranges.
