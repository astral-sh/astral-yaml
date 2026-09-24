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

- Initialized an empty directory as a Git repository and Rust workspace.
- Established core, serde, and development-tool crate boundaries and lint policy.
- Toolchain available: Rust 1.98.1; nightly installed. Coverage and cargo-fuzz tools
  still need installation.
- Compatibility, coverage, fuzzing, serde behavior, and performance are unproven.

## Design

Use byte ranges into UTF-8 input and an arena of nodes. Collection children and
aliases use node IDs, avoiding recursive ownership and unbounded recursive drop.
Keep syntax parsing separate from YAML 1.2 scalar resolution and serde conversion.
Borrow scalar text when decoding is unnecessary. Preserve comments separately from
semantic nodes, plus spans on syntax properties and stream/document boundaries.

## Next work

Implement the source model and parser, pin the official suite, and establish a
conformance runner that reports every mismatch without an expected-failure filter.
