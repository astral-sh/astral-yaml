# Fuzzing

Install `cargo-fuzz` and a nightly Rust toolchain, then run from the repository root:

```sh
python3 scripts/seed-fuzz.py
cargo +nightly fuzz run parse -- -max_total_time=60 -timeout=2 -max_len=65536 -rss_limit_mb=2048
cargo +nightly fuzz run serde -- -max_total_time=60 -timeout=2 -max_len=65536 -rss_limit_mb=2048
cargo +nightly fuzz run decode -- -max_total_time=60 -timeout=2 -max_len=65536 -rss_limit_mb=2048
cargo +nightly fuzz run typed -- -max_total_time=60 -timeout=2 -max_len=65536 -rss_limit_mb=2048
```

The serde target checks string and arbitrary-value round trips and error ranges.
The typed target covers all integer widths, floating point, booleans, characters,
strings, options, tuples, newtypes, enums, maps, byte visitors, and ignored fields.
Every type is wrapped in `Spanned<T>` to check its source range; additional paths
cover nested wrappers in collections, options, and enum payloads. Its first input
byte selects one of 28 types; the remaining bytes are YAML.
The decoding target exercises all five encodings and checks original-byte spans.
The parser target checks successful-stream source ranges, arena references,
comment ordering, and error ranges. It uses all official cases as seeds.
Campaign corpora and artifacts are ignored; reproduce and minimize any finding,
then preserve it in `fixtures/regressions` with a regression assertion.

To measure the source coverage reached by the generated corpora, install nightly
LLVM tools, replay each corpus, then generate source reports:

```sh
rustup component add llvm-tools-preview --toolchain nightly
for target in parse serde decode typed; do
  cargo +nightly fuzz coverage "$target" -- -timeout=2 -rss_limit_mb=2048
done
python3 scripts/fuzz-coverage.py
```

The script reports core and adapter source coverage separately for each target
and writes
`fuzz/coverage/summary.json`. It uses cargo-fuzz's default coverage build directory
and the host target. Each report uses the matching binary and profile: optimized
harnesses can produce incompatible versions of shared symbols when profiles are
combined. These corpus replays measure exercised source; they do not
perform additional mutations. Test coverage and fuzz coverage are separate
measurements, and execution counts alone do not imply source coverage.

`cargo llvm-cov --workspace --summary-only` measures unit and conformance coverage.
Use `cargo llvm-cov --workspace --html` to inspect uncovered lines. Coverage is a
completion gate, not an assumption derived from a passing corpus.
