# astral-yaml

A fast, safe Rust YAML 1.2 parser.

> [!WARNING]
> This README is human-edited, but all code changes, PR summaries, and additional
> documentation were authored entirely by [Codex](https://openai.com/codex/).
> Use at your own risk.

Core parts:

- `astral-yaml`: parsing, syntax, source spans, and comments.
- `astral-yaml-serde`: serialization and deserialization.

The parser targets [YAML 1.2.2](https://yaml.org/spec/1.2.2/). See
[conformance](docs/conformance.md) and [limitations](docs/limitations.md) for
supported behavior, and [SECURITY.md](SECURITY.md) for resource limits and
vulnerability reporting.

## Usage

The primary interfaces are:

- `astral-yaml` provides a low-level stream of syntax nodes with byte ranges,
  comment states, scalar styles, tags, and anchors. Aliases are not expanded.

- `astral-yaml-serde` provides a high-level serde-compatible serialization
  and deserialization interface. The serializer uses flow style and quoted
  strings.

Use `Spanned<T>` to retain a value's source range and `Commented<'a, T>`
to borrow its comments. The wrappers compose in either order:

```rust
use astral_yaml_serde::{Commented, Spanned, from_str};
use serde::Deserialize;

#[derive(Deserialize)]
struct Config<'a> {
    #[serde(borrow)]
    timeout: Spanned<Commented<'a, u64>>,
}

let source = "timeout: 30 # seconds\n";
let config: Config<'_> = from_str(source).unwrap();
assert_eq!(**config.timeout, 30);
assert_eq!(config.timeout.span.slice(source), Some("30"));
assert_eq!(config.timeout.comments[0].comment.text, " seconds");
```

`Spanned<T>` implements `DeserializeOwned` when `T` does. `Commented` borrows
the input even for owned payloads. Both serialize only their value.

Enable `astral-yaml-serde`'s non-default `editable` feature to edit a document with
`EditableYaml`, or use `Editable<T>` to check every edit against a Rust type.
See [editing YAML](docs/editing.md) for examples and preservation rules.

Enums use variant names for unit variants and singleton mappings for variants
with payloads. Serde's representation attributes, including `untagged`, still
apply. Core tags such as `!!str` and `!!int` work by default. Application tags
require an explicit opt-in.

Use `with::yaml_tag` on an enum field to opt into local YAML tags:

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
enum Message {
    Text(String),
    #[serde(untagged)]
    Number(u32),
}

#[derive(Deserialize, Serialize)]
struct Config {
    #[serde(with = "astral_yaml_serde::with::yaml_tag")]
    message: Message,
}

let config: Config = astral_yaml_serde::from_str("message: !Text hello").unwrap();
```

The adapter supports mixed tagged/untagged enums. It applies to the annotated
value through options and newtypes. Nested values require their own opt-in.

Use `Tagged<T>` to retain an explicit local, global, or core tag with a typed payload:

```rust
use astral_yaml_serde::{Tagged, from_str};

let text: Tagged<String> = from_str("!Text hello").unwrap();
assert_eq!(text.tag, "!Text");
assert_eq!(*text, "hello");
```

`Tagged<T>` owns its tag and supports `DeserializeOwned` when `T` does. It rejects
untagged input and preserves the tag when serialized. Use
`Spanned<Commented<'a, Tagged<T>>>` to capture the source span and comments as well.

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and checks.

## Performance

See [benchmarks](benchmarks/README.md) for workloads, comparison settings, and
commands. Pull requests include CodSpeed performance reports.

<div align="center">
  <a target="_blank" href="https://astral.sh" style="background:none">
    <img src="https://raw.githubusercontent.com/astral-sh/uv/main/assets/svg/Astral.svg" alt="Made by Astral">
  </a>
</div>
