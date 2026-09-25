# Contributing

Install `rustup` and `uv`, then run the commands below from the repository root.
Python scripts declare PEP 723 metadata; run them with
`uv run --script scripts/<name>.py`.

## Checks

Run formatting, linting, type checks, fixture verification, tests, and coverage:

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

The first `uv run` sets up the development environment in `.venv`, which `ty`
uses to resolve Python imports. The ignored test compares fixture syntax and
spans with PyYAML's Python and LibYAML parsers; it requires `uv` to run the
script with its pinned dependency.

## Coverage

`cargo xtask cov` enforces 100% line and function coverage for `astral-yaml` and
`astral-yaml-serde`. It uses the active compiler's LLVM tools, installing
`llvm-tools-preview` through rustup if needed. Development helpers, xtask, and
separate test modules are excluded from the reports. Doctests run under
`cargo test --workspace`.

To also generate an HTML report:

```sh
cargo xtask cov --html
```

Coverage builds, fresh profiles, and `summary.json` go under `target/coverage`.
`--html` also writes `target/coverage/html/index.html`, even when coverage falls
below the threshold. These paths follow Cargo's configured target directory.

## Conformance and snapshots

Run the pinned YAML Test Suite with:

```sh
cargo test -p astral-yaml --test conformance
```

The parser's fixture tests use inline `insta` snapshots. With `cargo-insta`
installed, review updates with:

```sh
cargo insta test -p astral-yaml --test real_world --review
```

See [conformance and limits](docs/conformance.md) for supported behavior and the
[real-world corpus](fixtures/real-world/README.md) for snapshot contents and
fixture provenance.

## Fuzzing and benchmarks

```sh
cargo xtask fuzz
cargo bench -p xtask --bench real_world
```

`cargo xtask fuzz` sets up the tools and corpora, then runs all four targets for
60 seconds each. Extra arguments are forwarded to cargo-fuzz. See the
[fuzzing guide](fuzz/README.md) for limits and corpus replay, and the
[benchmark notes](benchmarks/README.md) for measurement and comparison commands.
