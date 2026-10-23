# Contributing

Development requires `rustup` and `uv`. Run commands from the repository root.

## Rust style

Use stable `rustfmt` and the workspace Clippy lints. For API and implementation
choices, consult [Google's Rust style guide](https://google.github.io/styleguide/rustguide.html).

Keep the fixed YAML 1.2 scalar and collection styles exhaustively matchable.
Keep `ParseOptions` constructible with `..Default::default()` so callers can
override individual limits. Document which source and byte offsets a public
span refers to.

## Checks

```sh
uv run --only-dev --locked ruff format --check scripts
uv run --only-dev --locked ruff check scripts
uv run --only-dev --locked ty check --python .venv scripts

cargo fmt --all --check
cargo fmt --manifest-path fuzz/Cargo.toml --check
cargo clippy --workspace --all-targets --all-features -- -D warnings

cargo test --workspace --all-features --locked
cargo test --workspace --no-default-features --locked
uv run --script scripts/verify-fixtures.py
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
cargo test -p astral-yaml-serde --test conformance
```

Review inline snapshots with `cargo-insta`:

```sh
cargo insta test -p astral-yaml --test real_world --review
```

## Fuzzing and benchmarks

```sh
cargo xtask fuzz
cargo bench -p astral-yaml-dev --bench yaml --locked
```

`cargo xtask fuzz` runs all targets for 60 seconds each. Additional arguments
are passed to cargo-fuzz instead.

See [fuzzing](fuzz/README.md) and [benchmarks](benchmarks/README.md) for details.
