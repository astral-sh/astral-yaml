# astral-yaml

A pure Rust YAML 1.2 parser under development. The core parser uses only `std`;
serde support lives in a separate crate. Workspace code forbids unsafe Rust.

- `astral-yaml`: syntax, source spans, comments, and parsing.
- `astral-yaml-serde`: serialization and deserialization.
- `astral-yaml-dev`: conformance and performance tools (not published).

The implementation targets the [YAML 1.2.2 specification](https://yaml.org/spec/1.2.2/)
and [YAML Test Suite](https://github.com/yaml/yaml-test-suite). YAML 1.1 schema
compatibility is outside the scope of this project.

See [TRANSCRIPT.md](TRANSCRIPT.md) for measured progress and outstanding work.

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
