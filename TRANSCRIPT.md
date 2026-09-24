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

- Core parser implements block/flow collections, all scalar styles, stream markers,
  directives, tags, anchors/aliases, comments, byte spans, and core-schema classification.
- Official YAML Test Suite: **402/402** cases pass (308 valid, 94 invalid), pinned
  at `6ad3d2c62885d82fc349026c136ef560838fdf3d`. Full events are compared and span
  invariants checked; no expected-failure list or case exclusions.
- `cargo test --workspace`: eight focused tests plus the complete corpus pass.
- Resource guards cover input length, node count, and nesting (default 64, hard
  maximum 128). Deep block/flow tests reject 1,000/10,000 nested collections.
- Parser runtime dependencies: none; verified with `cargo tree`.
- Initial parser fuzz campaign found two malformed multiline-scalar slicing panics;
  both are fixed, with checked-in regressions. A subsequent AddressSanitizer run
  completed 3,536,992 executions in 31 seconds with no findings (8 KiB max input,
  per-input timeout 2 seconds, 2 GiB RSS limit, seeds from all official cases).
- Initial coverage before adapter implementation: parser 96.11% lines, core model,
  errors, and schema 100% lines. Workspace 94.83% lines. **100% is not yet met.**
- Serde adapter is in progress; its completion gates remain open.
- Full compatibility is not established by corpus success alone. Coverage,
  fuzzing, real-world fixtures, serde, and performance remain unproven.

## Design

Use byte ranges into UTF-8 input and an arena of nodes. Collection children and
aliases use node IDs, avoiding recursive ownership and unbounded recursive drop.
Keep syntax parsing separate from YAML 1.2 scalar resolution and serde conversion.
Borrow scalar text when decoding is unnecessary. Preserve comments separately from
semantic nodes, plus spans on syntax properties and stream/document boundaries.

## Next work

Run coverage and corpus-seeded fuzzing; fix findings and preserve regressions.
Implement and test the serde adapter, add pinned real-world fixtures and equivalent
`yaml_serde` benchmarks. Audit spec requirements beyond corpus coverage, including
byte-input encodings and BOMs, complete tag URI validation, key uniqueness during
composition, and behavior at all configured limits. Do not declare completion from
corpus success alone.
