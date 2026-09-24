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

For encoded byte input, use `astral_yaml::Source::from_bytes(&bytes)` and parse its
decoded text. UTF-8 is borrowed; UTF-16 and UTF-32 in either byte order are decoded
with size limits. `Source::original_span` maps a parsed UTF-8 span back to the
original encoded bytes.
