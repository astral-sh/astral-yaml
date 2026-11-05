# astral-yaml

astral-yaml is a YAML 1.2 parser.

This is a low(er)-level API. Most users probably want [astral-yaml-serde]
for high level serialization, deserialization, and editing operations.

[astral-yaml-serde]: https://crates.io/crates/astral-yaml-serde

## Usage

```rust
let source = "'042' # identifier\n";
let stream = astral_yaml::parse(source).unwrap();
let root = stream.node(stream.documents()[0].root).unwrap();

assert_eq!(root.span.slice(source), Some("'042'"));
assert_eq!(stream.comments()[0].text, " identifier");
```

See the [documentation] for more information.

[documentation]: https://docs.rs/astral-yaml

<div align="center">
  <a target="_blank" href="https://astral.sh" style="background:none">
    <img src="https://raw.githubusercontent.com/astral-sh/uv/main/assets/svg/Astral.svg" alt="Made by Astral">
  </a>
</div>
