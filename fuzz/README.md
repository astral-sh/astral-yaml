# Fuzzing

With `uv` and `rustup` on `PATH`, run:

```sh
cargo xtask fuzz
```

The command uses `uv run --only-dev --locked` to install the project's pinned
development tools, seeds all four corpora, installs nightly if missing, and adds
its `rust-src` and `llvm-tools-preview` components. It then runs `parse`, `serde`,
`decode`, and `typed` sequentially for 60 seconds each, with a 2-second input
timeout, 64 KiB input limit, and 2 GiB RSS limit. Setup and fuzzing failures stop
the command immediately. Missing `uv` is an error.

Explicit arguments are passed through to cargo-fuzz after the same setup:

```sh
cargo xtask fuzz list
cargo xtask fuzz run parse -- -max_total_time=10 -timeout=2 -max_len=65536 -rss_limit_mb=2048
cargo xtask fuzz tmin parse /absolute/path/to/crash
```

Explicit runs use the limits you supply; the default campaign's limits are not
added. All subprocesses run from the repository root, including when invoking
xtask from a crate directory, so relative corpus and artifact paths use that
root. Nightly selection applies only to the fuzz subprocesses. Existing corpus
discoveries and artifacts are retained when seeding.

The serde target checks string and arbitrary-value round trips and error ranges.
It also reads every successfully serialized string and value with `yaml_serde`,
so a shared parser/emitter bug cannot hide behind a successful self-round-trip.
The typed target covers all integer widths, floating point, booleans, characters,
strings, options, tuples, newtypes, enums, maps, byte visitors, and ignored fields.
Every type is wrapped in `Spanned<'a, T>` to check its source range, complete
comment aggregation, source ordering, and borrowed comment text. Additional paths
cover nested wrappers in collections, options, and enum payloads. A syntax-only
value exercises comment indexing for arbitrary tags and cyclic aliases without
expanding their values. The first input byte selects one of 29 types; the remaining
bytes are YAML.
The decoding target exercises all five encodings and checks original-byte spans.
The parser target checks successful-stream source ranges, arena references,
comment ordering, and error ranges. It uses all official cases as seeds.
Campaign corpora and artifacts are ignored; reproduce and minimize any finding,
then preserve it in `fixtures/regressions` with a regression assertion.

To measure the source coverage reached by the generated corpora, replay each
corpus, then generate source reports. The wrapper installs the required LLVM
tools during setup:

```sh
for target in parse serde decode typed; do
  cargo xtask fuzz coverage "$target" -- -timeout=2 -rss_limit_mb=2048
done
uv run --script scripts/fuzz-coverage.py
```

The script reports core and adapter source coverage separately for each target
and writes
`fuzz/coverage/summary.json`. It uses cargo-fuzz's default coverage build directory
and the host target. Each report uses the matching binary and profile: optimized
harnesses can produce incompatible versions of shared symbols when profiles are
combined. These corpus replays measure exercised source; they do not
perform additional mutations. Test coverage and fuzz coverage are separate
measurements, and execution counts alone do not imply source coverage.

`cargo xtask cov` measures unit and integration coverage, including conformance.
Use `cargo xtask cov --html` to inspect uncovered lines. Coverage is a
completion gate, not an assumption derived from a passing corpus.
