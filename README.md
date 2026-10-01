# astral-yaml

A fast, safe Rust YAML 1.2 parser.

> [!WARNING]
> This README is human-edited, but all code changes, PR summaries, and additional
> documentation were authored entirely by GPT-6 Astra in [Codex](https://openai.com/codex/).
> Use at your own risk.

Core parts:

- `astral-yaml`: parsing, syntax, source spans, and comments.
- `astral-yaml-serde`: serialization and deserialization.

The parser targets [YAML 1.2.2](https://yaml.org/spec/1.2.2/). See
[conformance](docs/conformance.md), [limitations](docs/limitations.md), and
[resource limits](docs/resource-limits.md) for supported behavior.

## Usage

There are two primary interfaces:

- `astral-yaml` provides a low-level stream of syntax nodes with byte ranges,
  comment states, scalar styles, tags, and anchors. Aliases are not expanded.

- `astral-yaml-serde` provides a high-level serde-compatible serialization
  and deserialization interface. The serializer uses flow style and quoted
  strings.


One of astral-yaml's specialties is that it's _span aware_, and can convey
span information even within the serde layer. For any deserializable `T`,
wrap a value or field in `Spanned<'a, T>` to retain its source range and
comments:

```rust
use astral_yaml_serde::{Spanned, from_str};
use serde::Deserialize;

#[derive(Deserialize)]
struct Config<'a> {
    #[serde(borrow)]
    timeout: Spanned<'a, u64>,
}

let source = "timeout: 30 # seconds\n";
let config: Config<'_> = from_str(source).unwrap();
assert_eq!(*config.timeout, 30);
assert_eq!(config.timeout.span.slice(source), Some("30"));
assert_eq!(config.timeout.comments[0].comment.text, " seconds");
```

Note that `Spanned` has an interior lifetime, so it cannot be used with `DeserializeOwned`.

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
value through options and newtypes; nested values require their own opt-in.

Use `Tagged<T>` to retain an explicit local, global, or core tag with a typed payload:

```rust
use astral_yaml_serde::{Tagged, from_str};

let text: Tagged<String> = from_str("!Text hello").unwrap();
assert_eq!(text.tag, "!Text");
assert_eq!(*text, "hello");
```

`Tagged<T>` owns its tag and supports `DeserializeOwned` when `T` does. It rejects
untagged input and preserves the tag when serialized. Use
`Spanned<'a, Tagged<T>>` to capture the source span and comments as well.

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and checks.

## Performance

The [deserialization benchmarks](benchmarks/README.md#implementation-comparison-2026-09-30)
compare Astral with `deser-yaml` 0.9.1, `yaml_serde` 0.10.7, and serde-saphyr 1.3.0:

| Workload | Astral | `deser-yaml` | `yaml_serde` | serde-saphyr |
| --- | ---: | ---: | ---: | ---: |
| uv CI workflow | 26.69 µs | 70.93 µs | 136.85 µs | 189.65 µs |
| Kubernetes Guestbook | 12.47 µs | 35.68 µs | 54.56 µs | 101.80 µs |
| OpenAPI Petstore | 14.81 µs | 44.89 µs | 69.25 µs | 114.30 µs |
| 128 services (typed) | 107.95 µs | 229.60 µs | — | 887.05 µs |

The fixture workloads deserialize into dynamic values; `deser-yaml` uses its own
value type, which retains additional formatting information. The service workload
uses the same structs for each implementation. These are medians of six run
medians on an Apple M5 Max with Rust 1.98.1, including parsing and output
destruction. See [benchmarks](benchmarks/README.md) for settings and more results.
