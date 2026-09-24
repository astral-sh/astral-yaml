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

## Validation evidence (2026-09-24)

- **402/402 official cases pass**: 308 valid, 94 invalid, pinned at
  `6ad3d2c62885d82fc349026c136ef560838fdf3d`. Full events and span invariants are
  checked with no excluded cases. Every UTF-8 prefix of every input is also checked.
- **55 tests pass** on stable Rust 1.98.1 and MSRV 1.85.1. Three pinned real-world
  fixtures match the reference values and round-trip through serde.
- `scripts/check.sh` passes: rustfmt (workspace and fuzz targets), strict workspace
  clippy, tests, fixture hashes, and enforced **100% line / 100% function coverage**.
  The report covers 2,693 lines and 279 functions. LLVM region coverage is **97.30%**;
  the 100% claim is specifically line and function coverage, not branch coverage.
  Strict clippy also passes for the separate fuzz workspace.
- Dependency trees confirm the runtime dependency constraints.
- A spec audit added regressions for characters, encodings, tag URI/prefix rules,
  implicit keys, continuation indentation, flow value separation, document
  transitions, metadata ranges, and serde type/visitor semantics.
- The suite event convention decodes URI escapes in case `6CK3`; AST tags preserve
  their spelling per spec §5.6. `yaml_serde` rejects some valid 1.2 leading-colon
  keys; those cases use grammar-derived assertions instead of that comparator.

## Fuzzing evidence

Latest campaigns on the final implementation used nightly
`1.100.0-nightly (cea272fa3 2026-09-07)`, cargo-fuzz 0.13.2, and these bounds:
31 seconds elapsed per target, 2-second input timeout, 64 KiB mutations, 2 GiB RSS.
All four completed without findings: **3,896,794 executions** in total.

| Target | Executions | Relevant source line coverage from corpus replay |
| --- | ---: | --- |
| parse | 1,200,210 | Parser 98.10% |
| serde | 675,102 | Parser 98.93%; generic serde does not exercise typed methods |
| decode | 764,051 | Parser 99.51%; decoder 88.29% |
| typed | 1,257,431 | Deserializer 95.06%; serializer 98.30%; schema 100% |

Seeds include 407 official/regression/real-world source files, all five encodings,
and 24 serde type selections with boundary examples. Earlier campaigns exposed two
scalar-slicing panics; saved regressions cover both fixes. The typed target checks
canonical round-trips, including NaN and signed zero.

`fuzz/README.md` gives reproduction commands. `scripts/fuzz-coverage.py` generates
per-target reports and `fuzz/coverage/summary.json` from matching binaries/profiles.
Optimized harness profiles are reported separately to avoid incompatible shared
symbol versions. Fuzz coverage is independent of test coverage and is not 100%.

## Performance and reproduction

Final release medians versus pinned `yaml_serde` 0.10.7 are **1.61× / 1.62× / 1.62×**
faster on uv CI / Kubernetes Guestbook / OpenAPI Petstore. Both sides construct and
drop the same `Vec<yaml_serde::Value>`; equality is checked before timing. See
[benchmarks/README.md](benchmarks/README.md) for values and methodology.

Run `scripts/check.sh`, `cargo +1.85.1 test --workspace --locked`, the bounded fuzz
campaigns/replays, and `cargo bench -p astral-yaml-dev --bench real_world` after
substantive changes. Locally installed tools are under
`/Users/ww/.cache/astral-yaml-tools/{coverage,fuzz}/bin`.
No Docker containers were used. All validation, fuzz, and benchmark processes finished.
