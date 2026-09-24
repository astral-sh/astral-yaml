# Engineering status

## Requirements and completion evidence

The project is complete only when all of these gates have evidence:

- Strict YAML 1.2 syntax and semantics: pinned official corpus, spec-derived edge
  cases, malformed-input rejection, and real-world fixtures.
- Source ranges for every parsed element, including comments: invariant tests
  across the corpus and fuzz inputs.
- `astral-yaml-serde`: serialization/deserialization tests, including derived types,
  YAML 1.2 scalar resolution, tags, aliases, and error locations.
- Safe, idiomatic Rust: workspace-wide unsafe prohibition, rustfmt, strict clippy.
- Core runtime dependencies limited to std; adapter dependencies limited to the
  serde/proc-macro stack: verified dependency trees.
- 100% test coverage: reproducible coverage measurement with no uncovered parser
  or adapter code; fuzz targets and bounded recorded fuzz runs.
- Better performance than `yaml_serde` on real-world inputs: pinned fixtures,
  equivalent workloads, reproducible release benchmarks with recorded results.
- Malformed-input resilience: explicit resource budgets, adversarial regression
  tests, and fuzz evidence.
- Logical crates, incremental commits, and this trimmed engineering reference.

## Current state

- Implemented the core parser and serde adapter, with no core runtime dependencies
  and only the serde/proc-macro stack in the adapter. Workspace code forbids unsafe.
- Core supports block/flow collections, scalar styles, streams, directives, tags,
  anchors/aliases, comments, spans, core-schema classification, and UTF-8/16/32.
  `Source::original_span` maps decoded spans to original encoded bytes.
- Official corpus: **402/402** (308 valid, 94 invalid), pinned at
  `6ad3d2c62885d82fc349026c136ef560838fdf3d`. Full events and span invariants are
  checked without excluded cases. Event formatting decodes tag URI escapes to
  match `6CK3`; AST tags preserve the original escaping per spec section 5.6,
  independently checked by regression tests.
- `cargo test --workspace` and `cargo +1.85.1 test --workspace --locked`: **30 tests
  pass**, including the entire corpus and three real-world fixture comparisons.
  Rustfmt and strict clippy pass. Fixture checksums and dependency trees checked.
- Parser limits cover input bytes, syntax elements (including comments/directives),
  and nesting (default 64, maximum configurable value 128). Serde also limits visits
  and inspected scalar bytes, rejects duplicate keys using core-schema identity,
  validates collection tags, and rejects recursive alias expansion.
- Multiline properties reuse the same arena node, preserving anchor identity.
  Focused tests cover BOMs, JSON-compatible quoted characters, URI escaping,
  implicit keys, scalar folding/continuation, machine numeric limits, and cycles.
- Latest fuzz campaigns on current code: parser **1,212,122**, serde **338,806**,
  decoder **971,775** executions; each completed in 31 seconds without findings.
  Seeds include all official inputs, two saved regressions, and the three real-world
  fixtures (407 source inputs), plus all five encodings for the decoder. Limits:
  64 KiB mutations, 2-second per-input timeout, 2 GiB RSS per process. Earlier
  campaigns found and fixed two scalar-slicing panics. Source fuzz coverage has
  **not yet been quantified**; execution counts are not a coverage claim.
- Latest line coverage: parser **98.73%**, source decoding **99.09%**, core model,
  errors, and schema **100%**; serde deserializer **90.96%**, serializer **95.34%**,
  serde errors **87.50%**. Whole workspace **95.31%**. **100% remains unmet.**
- Current release benchmark vs pinned `yaml_serde` 0.10.7: **1.63× / 1.58× / 1.55×**
  speedups on uv CI / Kubernetes Guestbook / OpenAPI Petstore, with equivalent
  value construction and destruction. See `benchmarks/README.md` for methodology,
  latency values, and reproduction. Fixtures match reference values and round-trip.

## Design and tooling

Flat arena ownership avoids recursive drop and alias expansion. Byte spans refer to
UTF-8 source; encoded input retains the original bytes for range conversion. Scalar
resolution is separate from parsing. Serde supports borrowed unescaped scalars,
derived types and enums, and emits flow-style YAML with quoted strings.

Coverage: `cargo llvm-cov --workspace --summary-only`; add `--show-missing-lines`
when generating a report. Fuzz reproduction commands are in `fuzz/README.md`.
Locally installed tools are in `/Users/ww/.cache/astral-yaml-tools/{coverage,fuzz}/bin`.
No Docker containers were used. All test and fuzz sessions above finished.

## Remaining work (goal remains active)

1. Reach measured 100% test coverage. Add meaningful tests for typed serde paths,
   byte visitors, map protocols, resource-limit boundaries, and CLI error paths.
   Refactor genuinely unreachable defensive branches by strengthening internal
   types; do not hide code with coverage exclusions or lower the gate.
2. Quantify fuzz source coverage and extend targets for typed serde methods that
   arbitrary `Value` deserialization does not exercise. Run and record new campaigns.
3. Complete the spec audit beyond the official corpus: particularly syntax rejection
   at context boundaries, exact implicit-key restrictions, tag URI/prefix rules,
   document transitions, and key validation inside ignored serde fields. Add
   regression cases for each uncovered requirement. Corpus success alone is not
   proof of full compatibility.
4. Re-run all gates, dependency checks, and release benchmarks after remaining
   correctness changes. Keep this reference current and commit logical work units.
