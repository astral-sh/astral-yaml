# Contributing

Development requires `rustup` and `uv`. Run commands from the repository root.

## Checks

```sh
uv run --only-dev --locked ruff format --check scripts
uv run --only-dev --locked ruff check scripts
uv run --only-dev --locked ty check --python .venv scripts

cargo fmt --all --check
cargo fmt --manifest-path fuzz/Cargo.toml --check
cargo clippy --workspace --all-targets -- -D warnings

cargo test --workspace --locked
uv run --script scripts/verify-fixtures.py
cargo test -p astral-yaml --test real_world --locked -- --ignored
cargo xtask cov
```

The first `uv run` creates `.venv`, including the type stubs used by `ty`.
Python scripts use PEP 723 metadata; run them with
`uv run --script scripts/<name>.py`.

`cargo test --workspace` includes doctests. The `--ignored` test compares fixture
syntax events and scalar spans with PyYAML's Python and LibYAML parsers. It uses
`uv` to run the comparison script with its pinned PyYAML version.

## Coverage

`cargo xtask cov` enforces 100% line and function coverage for `astral-yaml` and
`astral-yaml-serde`. It uses the active compiler's LLVM tools, installing
`llvm-tools-preview` through rustup if needed. Reports exclude development tooling
and separate test modules.

Builds, profiles, and `summary.json` go under `target/coverage`.
`cargo xtask cov --html` also writes `target/coverage/html/index.html`, even when
coverage falls below the threshold. These paths follow Cargo's configured target
directory.

## Conformance and snapshots

Run the pinned YAML Test Suite:

```sh
cargo test -p astral-yaml --test conformance
```

Fixture tests use inline `insta` snapshots. With `cargo-insta` installed:

```sh
cargo insta test -p astral-yaml --test real_world --review
```

See [conformance and limits](docs/conformance.md) and
[fixture documentation](fixtures/real-world/README.md).

## Fuzzing and benchmarks

```sh
cargo xtask fuzz
cargo bench -p xtask --bench real_world
```

`cargo xtask fuzz` installs its tools, seeds the corpora, and runs all four targets
for 60 seconds each. With arguments, it forwards them to cargo-fuzz after setup.

See the [fuzzing guide](fuzz/README.md) for limits and corpus replay, and the
[benchmark notes](benchmarks/README.md) for measurement and comparisons.
