# Fuzzing

With `uv` and `rustup` on `PATH`, run:

```sh
cargo xtask fuzz
```

The command seeds the corpora, installs nightly if missing, and adds its
`rust-src` and `llvm-tools-preview` components. It runs cargo-fuzz through
`uv run --only-dev --locked`, using the project's pinned development tools.
Targets run sequentially for 60 seconds each, with a 2-second input timeout,
64 KiB input limit, and 2 GiB RSS limit.
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

The fuzz crate enables `editable` on `astral-yaml-serde` and the shared development helpers.

- `parse` checks source ranges, arena references, comment ordering, and error
  ranges. Its seeds include every official test-suite case.
- `decode` checks UTF-8 and both byte orders of UTF-16 and UTF-32, including
  spans in the original bytes.
- `serde` checks string and arbitrary-value round trips and error ranges.
- `editable` checks value edits and style changes on arbitrary YAML, including
  LF normalization and rollback. Its first eight bytes select edits and paths.
  Bit 5 selects between nested paths and operations starting at the root. The rest is YAML.
- `editable-model` checks valid edits against independent value and comment
  models, including nested collections, aliases, and canceled edits.
- `editable-graph` checks drafts and commits against an independent recursive
  tree and alias graph, including tags, complex keys, repeated anchors, document
  markers, and comment placements. It generates representable
  attachments and ends transaction batches at explicit layout requests. Up to
  eight initial bytes seed the graph. Byte zero also selects framing and batch
  size. Each subsequent three-byte group selects an operation and position.
  Inputs are bounded to 512 bytes and 32 operations.
- `typed` covers integer widths, floats, booleans, characters, strings, options,
  tuples, newtypes, enums, maps, byte visitors, and ignored fields. The first
  byte selects the type. The rest is YAML.

The typed target wraps each type in `Spanned<Commented<'a, T>>` and checks its source range,
descendant comments, comment order, and borrowed text. It also checks nested
wrappers in collections, options, and enum payloads. One type skips value
construction so comment indexing can be checked for arbitrary tags and cyclic
aliases.

Corpora and artifacts are gitignored. Reproduce and minimize findings, then save
them in `fixtures/regressions` with a regression assertion.

## Valid-edit model bounds

The recursive model retains live alias targets and avoids nested scalar suffixes
and trailing comments on empty nested collections: these have no attachment
boundary. Explicit style requests end a batch, since later edits can require a
different style. Block conversion retains flow delimiters where comment order or
an enclosing trailing comment requires them. Descendants of flow nodes stay
flow.

Other bounds limit the model, not the API:

| Model scope | Complementary checks |
| --- | --- |
| One inline comment per node, allowing arbitrary scalar replacements | Bounded layouts with all subsets of tag, anchor, opening, and closing inline comments |
| Generated detached comments use suffix positions | Corpus reinsertion at captured positions. Gap insertion and child-deletion tests |

The bounded layout tests compare equivalent comment edits with independent source
projections. Explicit block conversions also have parsed block witnesses with the
same content and attachments. Invalid edits remain covered by rollback tests.

## Corpus coverage

Replay each corpus, then generate source reports:

```sh
for target in parse serde decode typed editable editable-model editable-graph; do
  cargo xtask fuzz coverage "$target" -- -timeout=2 -rss_limit_mb=2048
done
uv run --script scripts/fuzz-coverage.py
```

The script reports coverage for the YAML crates separately for each target
and writes `fuzz/coverage/summary.json`. It uses cargo-fuzz's default coverage
build directory and the host target. Each report pairs a binary with its
profile. Combining profiles from optimized harnesses can mix incompatible
versions of shared symbols. Corpus replay does not perform mutations.

For coverage from unit and integration tests, run `cargo xtask cov`, or
`cargo xtask cov --html` to inspect uncovered lines. The 100% line and function
requirement applies to those tests, not to corpus replay.
