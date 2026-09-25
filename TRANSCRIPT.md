# Engineering status

## Implementation

The workspace contains `astral-yaml` (syntax and spans), `astral-yaml-serde`
(serialization/deserialization), and `astral-yaml-dev` (validation and benchmarks).
The core has no runtime dependencies. The adapter adds only serde and its proc-macro
stack. All workspace crates forbid unsafe Rust. Work is committed in incremental units.

The parser implements YAML 1.2 block/flow syntax, scalar styles, streams, directives,
tags, anchors/aliases, comments, core-schema classification, and UTF-8/16/32 input.
Every node and metadata element retains a source range, including standalone
end markers. `Source::original_span` converts UTF-8 ranges to original encoded bytes.
See [conformance and limits](docs/conformance.md) for the spec audit and API scope.

Flat arena ownership avoids recursive destruction and preserves alias identity.
Input bytes, elements, and nesting are bounded. Serde also limits visits and inspected
scalar bytes, validates core tags and duplicate keys (including ignored fields), and
bounds recursive newtype visitors. A cached line range prevents repeated prefix
scans in large flow collections. A 30,000-item array CLI diagnostic improved from
about 1.6 s to 0.02 s on warm runs on the same host.

The adapter exposes `Spanned<'a, T>` with public `value`, `span`, and `comments`
fields, `Deref`, `DerefMut`, and `into_inner`. Serialization emits only the value.
Wrappers support fields, keys, collections, options, and enum payloads.
Span offsets follow the core's decoded UTF-8 convention, including node properties;
aliases use their occurrence span. Serde's `untagged` and `flatten` buffers lose
metadata, so the wrapper must surround values decoded through those buffers.

Comment text borrows the input, including when `T` is owned. Containing structs
use `#[serde(borrow)]`; wrappers cannot satisfy `DeserializeOwned`, while ordinary
owned types continue to work. Parent wrappers collect all syntax descendants'
comments, including keys and ignored values, once each in source order. Placement
describes the direct owner. A lazy, budgeted index stores each comment once and
uses preorder subtree intervals for retrieval. Alias references are not traversed.

## Validation evidence (2026-09-25)

- **402/402 official cases pass**: 308 valid, 94 invalid, pinned at
  `6ad3d2c62885d82fc349026c136ef560838fdf3d`. Full events and span invariants are
  checked with no excluded cases. Every UTF-8 prefix of every input is also checked.
- **84 tests and three doctests pass** on stable Rust 1.98.1 and MSRV 1.85.1. Three
  pinned real-world fixtures match the reference values and round-trip through serde.
- `scripts/check.sh` passes: rustfmt (workspace and fuzz targets), strict workspace
  clippy, tests, fixture hashes, and enforced **100% line / 100% function coverage**.
  The report covers 3,317 lines and 350 functions. LLVM region coverage is **97.66%**;
  the 100% claim is specifically line and function coverage, not branch coverage.
  Strict clippy also passes for the separate fuzz workspace.
- Span regressions cover borrowing, scalar syntax, nested collections, implicit
  nulls, aliases, enum tags through transparent wrappers, document offsets,
  original-byte mapping, serialization, buffering limits, and visitor failures.
- Comment regressions cover parent/child overlap, ignored fields, attachment
  placement, complex keys, properties, scalar headers, flow delimiters, document
  boundaries, BOMs, encodings, alias cycles, and index/output resource limits.
  Compile-fail doctests enforce input lifetimes and the `DeserializeOwned` boundary.
- Dependency trees confirm the runtime dependency constraints.
- A spec audit added regressions for characters, encodings, tag URI/prefix rules,
  implicit keys, continuation indentation, flow value separation, document
  transitions, metadata ranges, and serde type/visitor semantics.
- The suite event convention decodes URI escapes in case `6CK3`; AST tags preserve
  their spelling per spec §5.6. `yaml_serde` rejects some valid 1.2 leading-colon
  keys; those cases use grammar-derived assertions instead of that comparator.

## Span and comment fuzzing (2026-09-25)

The typed target wraps all 29 type selections in `Spanned<'_, T>`, checks root
spans, comment completeness, source order, exact slices, and borrowed text pointers,
and includes nested wrappers in collections, options, and enum payloads. One
syntax-only selection exercises indexing for tags and cyclic aliases that ordinary
values reject. Its final campaign completed **1,803,697 executions** in 61 seconds
without findings. A generic-serde regression campaign also passed **973,639
executions** in 61 seconds. Both used the nightly toolchain and bounds listed below.

Replaying the final typed corpus reached **98.66%** comment index, **92.40%**
deserializer, **98.32%** serializer, and **59.02%** wrapper line coverage. Tests
cover mutation/extraction, malformed protocol paths, and resource boundaries;
workspace test coverage remains 100% for lines and functions. The parser and
decoder campaigns below are retained as the pre-span baseline, not claimed as
fresh runs.

## Fuzzing evidence (2026-09-24 baseline)

Campaigns before the span wrapper was added used nightly
`1.100.0-nightly (cea272fa3 2026-09-07)`, cargo-fuzz 0.13.2, and these bounds:
61 seconds elapsed per target, 2-second input timeout, 64 KiB mutations, 2 GiB RSS.
All four completed without findings: **7,132,929 executions** in total.

| Target | Executions | Relevant source line coverage from corpus replay |
| --- | ---: | --- |
| parse | 2,270,594 | Parser 98.14% |
| serde | 1,142,347 | Parser 98.95%; generic serde does not exercise typed methods |
| decode | 1,541,102 | Parser 99.51%; decoder 88.29% |
| typed | 2,178,886 | Deserializer 96.19%; serializer 98.32%; schema 100% |

Seeds include 407 official/regression/real-world source files, all five encodings,
and 24 serde type selections with boundary examples. Earlier campaigns exposed two
scalar-slicing panics; saved regressions cover both fixes. The typed target checks
canonical round-trips, including NaN and signed zero.

`fuzz/README.md` gives reproduction commands. `scripts/fuzz-coverage.py` generates
per-target reports and `fuzz/coverage/summary.json` from matching binaries/profiles.
Optimized harness profiles are reported separately to avoid incompatible shared
symbol versions. Fuzz coverage is independent of test coverage and is not 100%.

## Performance and reproduction

Three consecutive release benchmark runs versus pinned `yaml_serde` 0.10.7 measure
**2.18–2.20× / 2.06–2.07× / 2.08–2.09×** faster on uv CI / Kubernetes Guestbook /
OpenAPI Petstore. Every fixture in every run exceeds the 1.98× target. The fresh
baseline at `31bd945` was 1.63–1.65×. Both sides construct and drop the same
`Vec<yaml_serde::Value>`; equality is checked before timing. See
[benchmarks/README.md](benchmarks/README.md) for the changes, recorded run medians,
and unchanged measurement methodology.

The 2026-09-25 regression run after adding spans measured
**2.20× / 2.06× / 2.09×** speedups for the existing unwrapped workload. After adding
borrowed descendant comments, the same workload measured **2.17× / 2.04× / 2.06×**,
still above the 1.98× target for every fixture. Both runs are recorded in
[the new results](benchmarks/results/2026-09-25.csv); neither measures the cost of
opting into metadata.

Run `scripts/check.sh`, `cargo +1.85.1 test --workspace --locked`, the bounded fuzz
campaigns/replays, and `cargo bench -p astral-yaml-dev --bench real_world` after
substantive changes. Locally installed tools are under
`/Users/ww/.cache/astral-yaml-tools/{coverage,fuzz}/bin`.
No Docker containers were used. All validation, fuzz, and benchmark processes finished.
