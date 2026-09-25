# astral-yaml

A pure Rust YAML 1.2 parser under development. The core parser uses only `std`;
serde support lives in a separate crate. Workspace code forbids unsafe Rust.

- `astral-yaml`: syntax, source spans, comments, and parsing.
- `astral-yaml-serde`: serialization and deserialization.
- `astral-yaml-dev`: conformance and performance tools (not published).

The implementation targets the [YAML 1.2.2 specification](https://yaml.org/spec/1.2.2/)
and [YAML Test Suite](https://github.com/yaml/yaml-test-suite). YAML 1.1 schema
compatibility is outside the scope of this project.

See [conformance and limits](docs/conformance.md) for supported behavior and
[TRANSCRIPT.md](TRANSCRIPT.md) for measured engineering status.

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

With `cargo-llvm-cov` installed, `scripts/check.sh` also checks fixture hashes and
enforces 100% line and function coverage. [Fuzzing instructions](fuzz/README.md)
cover corpus seeding, bounded campaigns, and source-coverage reports.

Parse syntax with `astral_yaml::parse(source)`. The returned stream retains byte
ranges, comments, scalar styles, tags, and anchors. Collection children and aliases
refer to arena node IDs; aliases are not expanded by the parser.

Use `astral_yaml_serde::from_str::<T>(source)` for a `Deserialize` type and
`astral_yaml_serde::to_string(&value)` for a `Serialize` value. The emitter uses flow
style and quotes strings to preserve their type under YAML 1.2 core resolution.

Wrap a value or field in `astral_yaml_serde::Spanned<T>` to retain its source range:

```rust
use astral_yaml_serde::{Spanned, from_str};
use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    timeout: Spanned<u64>,
}

let source = "timeout: 30\n";
let config: Config = from_str(source).unwrap();
assert_eq!(*config.timeout, 30);
assert_eq!(config.timeout.span.slice(source), Some("30"));
```

`Spanned<T>` exposes `value` and `span`, implements `Deref` and `DerefMut` for `T`,
and serializes as the value alone. It supports borrowed values and nested wrappers.
Spans include node properties and delimiters; an alias uses its `*name` occurrence.
Serde buffering for `untagged` and `flatten` discards spans, so wrap the containing
enum or struct instead of fields decoded from a buffer.

For encoded byte input, use `astral_yaml::Source::from_bytes(&bytes)` and parse its
decoded text. UTF-8 is borrowed; UTF-16 and UTF-32 in either byte order are decoded
with size limits. `Source::original_span` maps a parsed UTF-8 span back to the
original encoded bytes.
