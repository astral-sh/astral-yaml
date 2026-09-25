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
[conformance and limits](docs/conformance.md) for supported behavior.

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

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and checks.

## Performance

The [latest benchmarks](benchmarks/results/2026-09-25.csv) compare
`astral-yaml-serde` with `yaml_serde` 0.10.7:

| Fixture | Bytes | `astral-yaml-serde` | `yaml_serde` | Speedup |
| --- | ---: | ---: | ---: | ---: |
| uv CI workflow | 11,602 | 50.64 µs | 145.10 µs | 2.87× |
| Kubernetes Guestbook | 3,366 | 22.30 µs | 59.30 µs | 2.66× |
| OpenAPI Petstore | 5,479 | 27.47 µs | 74.50 µs | 2.71× |

Both implementations deserialize into `Vec<yaml_serde::Value>`, including parsing,
value construction, and destruction. These are median times from eleven samples,
measured on 2026-09-25 with Rust 1.98.1 on macOS arm64.

See [benchmarks](benchmarks/README.md) for reproduction commands and measurements
with `Spanned`.
