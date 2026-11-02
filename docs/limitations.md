# Limitations

## Serde buffering

Serde buffers values when it reads untagged enums, internally tagged enums,
and flattened fields. If a plain scalar resolves to a number, boolean, or
null, the buffer loses the original text. For example, a `String` variant of
an untagged enum cannot read the number `42`. Write `"42"` or `!!str 42` to
give the variant a string.

The buffer also loses source spans, comments, and tags. `Spanned`, `Commented`,
and `Tagged` need direct access to the YAML deserializer. To capture a span or
comments around an untagged enum, wrap the whole enum in `Spanned` or
`Commented`. The same applies to a struct with flattened fields.

## Mapping keys

`astral-yaml-serde` checks for duplicate YAML keys before deserializing a
mapping.

- `{1: first, 01: second}` is an error. Both YAML keys are the integer `1`.
- `{1: first, "1": second}` passes the YAML check. One key is an integer and
  the other is a string. When read into a `BTreeMap<String, String>`, both
  become the Rust string `"1"`. The map keeps the last value, `"second"`.

`HashMap` also keeps the last value. Serde's `MapAccess` does not tell the
deserializer how the destination map compares keys. To reject collisions in
the Rust map, annotate each field that needs the check with
[`serde_with::MapPreventDuplicates`](https://docs.rs/serde_with/latest/serde_with/struct.MapPreventDuplicates.html).

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

fn main() -> Result<(), astral_yaml_serde::Error> {
    let source = r#"values: {1: first, "1": second}"#;
    assert!(astral_yaml_serde::from_str::<Config>(source).is_err());

    let config: Config = astral_yaml_serde::from_str(r#"values: {1: first, "01": second}"#)?;
    assert_eq!(config.values["1"], "first");
    assert_eq!(config.values["01"], "second");
    Ok(())
}
```

For numeric keys, the YAML check compares integer magnitudes as `u128` and
floats as `f64`, regardless of the destination key type. For scalar keys with
application tags, it compares the tag and the decoded text. If your application
treats other spellings as equal for a given tag, check for those collisions
when constructing the map. For a recursive collection key, a reference back
to an enclosing collection compares by the identity of that collection.

## Numeric values

The parser recognizes integer syntax even outside machine integer ranges.
Deserializing a number into a Rust type still requires it to fit. For example,
`256` is a YAML integer that fits in a `u16` but exceeds the range of a `u8`.

For input size, nesting, and processing limits, see [resource limits](../SECURITY.md#resource-limits).
