# astral-yaml

A pure Rust YAML 1.2 parser under development. The core parser uses only `std`;
serde support lives in a separate crate. Workspace code forbids unsafe Rust.

- `astral-yaml`: syntax, source spans, comments, and parsing.
- `astral-yaml-serde`: serialization and deserialization.
- `xtask/`: development commands and benchmarks (not published).

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

Conformance tests live in the parser crate and run directly against its API:
`cargo test -p astral-yaml --test conformance`.

The workspace uses the [xtask pattern](https://github.com/matklad/cargo-xtask) for
development commands:

```sh
cargo xtask events path/to/input.yaml
cargo bench -p xtask --bench real_world
```

`cargo xtask --help` shows usage. Input paths are relative to the current directory.

With `cargo-llvm-cov` and `uv` installed, `scripts/check.sh` also checks fixture
hashes, compares real-world syntax and spans with independent parsers, and enforces
100% line and function coverage. [Fuzzing instructions](fuzz/README.md)
cover corpus seeding, bounded campaigns, and source-coverage reports.

The parser's integration tests use inline `insta` snapshots of readable syntax
trees and diagnostics. With `cargo-insta` installed, run
`cargo insta test -p astral-yaml --test real_world --review` to review updates.

Parse syntax with `astral_yaml::parse(source)`. The returned stream retains byte
ranges, comments, scalar styles, tags, and anchors. Collection children and aliases
refer to arena node IDs; aliases are not expanded by the parser.

Use `astral_yaml_serde::from_str::<T>(source)` for a `Deserialize` type and
`astral_yaml_serde::to_string(&value)` for a `Serialize` value. The emitter uses flow
style and quotes strings to preserve their type under YAML 1.2 core resolution.

Wrap a value or field in `astral_yaml_serde::Spanned<'a, T>` to retain its source
range and borrow its comments:

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

`Spanned<'a, T>` exposes `value`, `span`, and `comments`, implements `Deref` and
`DerefMut` for `T`, and serializes as the value alone. Its comments include syntax
descendants, once per wrapper, in source order; parent and child lists overlap.
Each `AttachedComment` contains a borrowed core `Comment` and a `CommentPlacement`:
`Leading`, `Inline`, `Trailing`, or `Detached`. Placement describes the direct
owner, including when a comment appears in an ancestor's list.

The input must outlive the wrapper, even when `T` is owned. This makes the wrapper
incompatible with `DeserializeOwned`; ordinary owned types still work with the
existing deserialization APIs. `Spanned` remains `Clone`, but is no longer `Copy`.
Spans include node properties and delimiters; an alias uses its `*name` occurrence.
Alias wrappers collect use-site comments without importing target comments.
Serde buffering for `untagged` and `flatten` discards metadata, so wrap the
containing enum or struct instead of fields decoded from a buffer.

For encoded byte input, use `astral_yaml::Source::from_bytes(&bytes)` and parse its
decoded text. UTF-8 is borrowed; UTF-16 and UTF-32 in either byte order are decoded
with size limits. `Source::original_span` maps a parsed UTF-8 span back to the
original encoded bytes.
