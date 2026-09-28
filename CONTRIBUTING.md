# Contributing

Development requires `rustup` and `uv`. Run commands from the repository root.

## Checks

```sh
uv run --only-dev --locked ruff format --check scripts
uv run --only-dev --locked ruff check scripts
uv run --only-dev --locked ty check --python .venv scripts

cargo fmt --all --check
cargo fmt --manifest-path fuzz/Cargo.toml --check
cargo clippy --workspace --all-targets --all-features -- -D warnings

cargo test --workspace --all-features --locked
uv run --script scripts/verify-fixtures.py
cargo test -p astral-yaml --test real_world --locked -- --ignored
cargo xtask cov
```

## Coverage

We require 100% line and function coverage for `astral-yaml` and
`astral-yaml-serde`.

Use `cargo xtask cov --html` to generate a report at
`target/coverage/html/index.html`.

## Conformance and snapshots

Run the pinned YAML Test Suite:

```sh
cargo test -p astral-yaml --test conformance
```

The `saphyr` feature runs the [serde-saphyr comparison](docs/serde-saphyr.md) and
requires Rust 1.89. The default workspace build still supports Rust 1.85.

The `yaml_serde` comparison runs with `cargo test -p astral-yaml-dev --test yaml_serde`.
Both comparison suites use our test value type or concrete Rust types.

Review inline snapshots with `cargo-insta`:

```sh
cargo insta test -p astral-yaml --test real_world --review
```

## Fuzzing and benchmarks

```sh
cargo xtask fuzz
cargo bench -p astral-yaml-dev --all-features --bench yaml --locked
```

`cargo xtask fuzz` runs all four targets for 60 seconds each. Additional arguments
are passed to cargo-fuzz instead.

See [fuzzing](fuzz/README.md) and [benchmarks](benchmarks/README.md) for details.
