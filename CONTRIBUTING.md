# Contributing

Development requires `rustup` and `uv`. Run commands from the repository root.

## Python bindings

Use uv to build and test the [Python package](crates/astral-yaml-python/README.md).
The package supports CPython 3.11 and newer. The development tools use Python
3.14 or newer.

```sh
uv sync --locked --python 3.14
uv run --locked pytest
uv build
```

To select a Python interpreter for Rust tests that embed Python, set
`PYO3_PYTHON` to the path reported by `uv python find 3.14`.

## Rust style

Use stable `rustfmt` and the workspace Clippy lints. For API and implementation
choices, consult [Google's Rust style guide](https://google.github.io/styleguide/rustguide.html).

Stable rustfmt sorts imports. Clippy flags redundant imports, unnecessary
import braces, and redundant path qualifiers. During review, check that each
module’s imports share a `use` statement and that calls to free functions are
qualified when the module helps identify what they do. Import traits inside
the functions that use them unless they are used throughout the module.

Keep the fixed YAML 1.2 scalar and collection styles exhaustively matchable.
Keep `ParseOptions` constructible with `..Default::default()` so callers can
override individual limits. Document which source and byte offsets a public
span refers to.

## Checks

```sh
uv run --only-dev --locked ruff format --check scripts astral_yaml.pyi benchmarks crates/astral-yaml-python/tests
uv run --only-dev --locked ruff check scripts astral_yaml.pyi benchmarks crates/astral-yaml-python/tests
uv sync --locked --script scripts/release.py
uv run --only-dev --locked ty check --python "$(uv python find --script scripts/release.py)" scripts
uv run --only-dev --locked ty check --python-version 3.11 astral_yaml.pyi benchmarks crates/astral-yaml-python/tests

cargo fmt --all --check
cargo fmt --manifest-path fuzz/Cargo.toml --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo clippy --manifest-path fuzz/Cargo.toml --all-targets --locked -- -D warnings

cargo test --workspace --all-features --locked
cargo test --workspace --no-default-features --locked
uv run --script scripts/verify-fixtures.py
cargo xtask cov
```

## Coverage

We require 100% line and function coverage for `astral-yaml`,
`astral-yaml-serde`, and `astral-yaml-python`. `cargo xtask cov` installs an
instrumented Python extension and runs pytest under coverage.py. It combines
the native coverage from that run with coverage from the Rust tests.

The gate counts each executable source line once across both runs using
`target/coverage/coverage.info`. LLVM's `summary.json` also includes statistics
for compiled copies of functions.
Use `cargo xtask cov --html` to see annotated source for all three crates in
`target/coverage/html/index.html`.

For Python coverage, run `uv run --locked coverage run -m pytest` followed by
`uv run --locked coverage report`. This measures the Python code shipped in
the package, currently the initializer that re-exports the native functions.
`cargo xtask cov` measures the Rust implementation of those functions.

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

## Releases

Use the same version (e.g. `0.2.0`) for both workflows on `main`.

1. Run [Prepare release](https://github.com/astral-sh/astral-yaml/actions/workflows/release-prepare.yml) and merge its PR.
2. Run [Release](https://github.com/astral-sh/astral-yaml/actions/workflows/release.yml) and approve `release-gate`.
