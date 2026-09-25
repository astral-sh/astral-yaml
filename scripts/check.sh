#!/bin/sh
# Run from the repository root with cargo-llvm-cov and uv installed.
set -eu
uv run --only-dev --locked ruff format --check scripts
uv run --only-dev --locked ruff check scripts
uv run --only-dev --locked ty check --python .venv scripts
cargo fmt --all --check
cargo fmt --manifest-path fuzz/Cargo.toml --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked
uv run --script scripts/verify-fixtures.py
cargo test -p astral-yaml --test real_world --locked -- --ignored
cargo llvm-cov --workspace --locked --fail-under-lines 100 --fail-under-functions 100 --summary-only
