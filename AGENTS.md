- astral-yaml is a strictly conforming YAML 1.2 parser. We do not support YAML 1.1 or any non-standard extensions to YAML.

- The YAML parser should be resilient to malformed and invalid inputs.

- Runtime dependencies are limited to `memchr` in the core parser, and
  `astral-yaml`, `serde`, and `memchr` in `astral-yaml-serde`.

- All library code should be 100% safe Rust, with no use of `unsafe` whatsoever.

- All library code should be written in idiomatic Rust. For example we should always use "parse, don't validate"
  to make invalid states unrepresentable.

- All library code should have 100% test coverage.

- All Rust should be written in a reader-friendly manner, meaning it should not be golfed or overly dense.
  Changes should preserve logical separations between functions and critical sections using whitespace.

- Tests should use snapshots (via insta) where appropriate, instead of a bunch of
  asserts for expected state/structure. Snapshots must be inline; do not use
  sidecar `.snap` files. Avoid assertions that duplicate snapshot expectations.

- Tests should be table-driven where appropriate, rather than defining several nearly-identical test functions.

- Where appropriate, tests should reference the section of the YAML specification they're testing for,
  with a link to that section.
