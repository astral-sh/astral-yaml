# Fuzzing

With `uv` and `rustup` on `PATH`, run:

```sh
cargo xtask fuzz
```

The command seeds all seven corpora, installs nightly if missing, and adds its
`rust-src` and `llvm-tools-preview` components. It runs cargo-fuzz through
`uv run --only-dev --locked`, using the project's pinned development tools.
The `parse`, `serde`, `decode`, `typed`, `editable`, `editable-model`, and `editable-graph` targets
run sequentially for 60 seconds each, with a 2-second input timeout, 64 KiB input limit, and 2 GiB RSS limit.
Setup or fuzzing failures stop the command. Missing `uv` is an error.

Extra arguments are passed to cargo-fuzz after the same setup:

```sh
cargo xtask fuzz list
cargo xtask fuzz run parse -- -max_total_time=10 -timeout=2 -max_len=65536 -rss_limit_mb=2048
cargo xtask fuzz tmin parse /absolute/path/to/crash
```

Explicit runs use only the limits you supply. Subprocesses run from the repository
root, so relative corpus and artifact paths use that root even when xtask is
invoked from a crate directory. Nightly selection applies only to fuzz subprocesses.
Seeding preserves existing corpus discoveries and artifacts.

## Targets

- `parse` checks source ranges, arena references, comment ordering, and error
  ranges. Its seeds include every official test-suite case.
- `decode` checks UTF-8 and both byte orders of UTF-16 and UTF-32, including
  spans in the original bytes.
- `serde` checks string and arbitrary-value round trips and error ranges.
- `editable` checks bounded syntax and value-based edit sequences, typed-view
  consistency, LF source normalization, and complete rollback. Its first eight bytes select edits and
  navigation paths; bit 5 selects the value-based API. The rest is YAML.
- `editable-model` checks valid edits against an independent value and comment
  model after each batch. Input line-ending variation is checked on load; all
  subsequent edits must retain LF source and the modeled scalar values. Nested
  mappings also appear in compact sequences up to three levels deep, so scalar
  and comment operations exercise enclosing indentation and suffix boundaries.
- `editable-graph` checks recursive sequences and mappings, arbitrary complex
  keys, self references, and repeated anchor names against an independent graph.
  It checks the draft after each modeled operation and the committed document
  after each batch, including scalar types, child order, anchors, alias target
  identity, and directly owned leading comments. Comment insertion, replacement,
  and removal compose with structural edits and layout requests. Its first byte
  selects the initial graph and batch size; each following
  three-byte group selects an operation and position. Inputs are limited to 512
  bytes and 32 operations. Destructive edits are generated only when all retained
  aliases still have targets. Canceled item and comment insertions must retain the
  exact source.
- `typed` covers integer widths, floats, booleans, characters, strings, options,
  tuples, newtypes, enums, maps, byte visitors, and ignored fields. The first byte
  selects one of 29 types; the rest is YAML.

The typed target wraps each type in `Spanned<Commented<'a, T>>` and checks its source range,
descendant comments, comment order, and borrowed text. It also checks nested
wrappers in collections, options, and enum payloads. One type skips value
construction so comment indexing can be checked for arbitrary tags and cyclic
aliases.

Corpora and artifacts are gitignored. Reproduce and minimize findings, then save
them in `fixtures/regressions` with a regression assertion.

## Corpus coverage

Replay each corpus, then generate source reports:

```sh
for target in parse serde decode typed editable editable-model editable-graph; do
  cargo xtask fuzz coverage "$target" -- -timeout=2 -rss_limit_mb=2048
done
uv run --script scripts/fuzz-coverage.py
```

The script reports coverage for the two YAML crates separately for each target
and writes `fuzz/coverage/summary.json`. It uses cargo-fuzz's default coverage build
directory and the host target. Each report pairs a binary with its profile;
combining profiles from optimized harnesses can mix incompatible versions of
shared symbols. Corpus replay does not perform mutations.

For coverage from unit and integration tests, run `cargo xtask cov`, or
`cargo xtask cov --html` to inspect uncovered lines. The 100% line and function
requirement applies to those tests, not to corpus replay.
