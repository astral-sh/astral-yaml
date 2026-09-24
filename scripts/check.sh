#!/bin/sh
# Run from the repository root with cargo-llvm-cov installed.
set -eu
cargo fmt --all --check
cargo fmt --manifest-path fuzz/Cargo.toml --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --locked
python3 scripts/verify-fixtures.py
cargo llvm-cov --workspace --locked --fail-under-lines 100 --fail-under-functions 100 --summary-only
