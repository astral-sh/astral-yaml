# Fuzzing

Install `cargo-fuzz` and a nightly Rust toolchain, then run from the repository root:

```sh
python3 scripts/seed-fuzz.py
cargo +nightly fuzz run parse -- -max_total_time=60 -timeout=2 -max_len=65536 -rss_limit_mb=2048
cargo +nightly fuzz run serde -- -max_total_time=60 -timeout=2 -max_len=65536 -rss_limit_mb=2048
cargo +nightly fuzz run decode -- -max_total_time=60 -timeout=2 -max_len=65536 -rss_limit_mb=2048
```

The serde target checks string and arbitrary-value round trips and error ranges.
The decoding target exercises all five encodings and checks original-byte spans.
The parser target checks successful-stream source ranges, arena references,
comment ordering, and error ranges. It uses all official cases as seeds.
Campaign corpora and artifacts are ignored; reproduce and minimize any finding,
then preserve it in `fixtures/regressions` with a regression assertion.

`cargo llvm-cov --workspace --summary-only` measures unit and conformance coverage.
Use `cargo llvm-cov --workspace --html` to inspect uncovered lines. Coverage is a
completion gate, not an assumption derived from a passing corpus.
