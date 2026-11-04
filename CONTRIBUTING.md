# Contributing

Development requires `rustup` and `uv`. Run commands from the repository root.

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
uv run --only-dev --locked ruff format --check scripts
uv run --only-dev --locked ruff check scripts
uv sync --locked --script scripts/release.py
uv run --only-dev --locked ty check --python "$(uv python find --script scripts/release.py)" scripts

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

## Releases

For the first release, skip to step 3 if `main` already has the intended version.

1. Run [Prepare release](https://github.com/astral-sh/astral-yaml/actions/workflows/release-prepare.yml)
   from `main` with the new Cargo version, omitting the `v` prefix.
2. Review and merge the version-bump PR.
3. Run [Release](https://github.com/astral-sh/astral-yaml/actions/workflows/release.yml)
   from `main` with that exact version.
4. Approve the `release-gate` deployment. The workflow publishes `astral-yaml`
   and `astral-yaml-serde` from the `release` environment, then tags the
   published commit and creates the GitHub release.
