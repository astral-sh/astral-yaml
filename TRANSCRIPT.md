# Validation record

Recorded test, fuzz, and benchmark results. See [conformance](docs/conformance.md),
[limitations](docs/limitations.md), and [resource limits](docs/resource-limits.md)
for API behavior. Development commands are in the [contributing guide](CONTRIBUTING.md).

## Tests (2026-09-25)

- All 402 cases in YAML Test Suite revision
  `6ad3d2c62885d82fc349026c136ef560838fdf3d` pass: 308 valid and 94 invalid.
  Tests compare event streams and check spans, including every UTF-8 prefix of
  every input.
- 105 tests and three doctests pass on Rust 1.98.1. All workspace targets,
  including xtask and benchmarks, check on MSRV 1.85.1.
- Rust and Python formatting, clippy, Ruff, ty, fixture hashes, tests, the PyYAML
  comparison, and coverage pass. Clippy also passes for the
  separate fuzz workspace.
- `astral-yaml` and `astral-yaml-serde` have 100% line and function coverage:
  3,078 lines and 337 functions. Region coverage is 97.76%. Development tooling
  and separate test modules are excluded.

The twelve [real-world fixtures](fixtures/real-world/README.md) contain 223,181
source bytes. Eleven parse successfully; Sentry's Compose file is a rejection
case for an under-indented closing flow delimiter. PyYAML's Python and LibYAML
parsers agree on syntax events and scalar ranges for the eleven accepted files.
They accept Sentry's file, which astral-yaml rejects. Tests also compare LF, CRLF,
and CR input, alias targets, shell text, scalar styles, and root comment lists.

Generated tests check 73,728 block scalar presentations and 3,258 quoted scalar
presentations. Serializer tests check all 1,112,064 Unicode scalar values, 23,040
f32/f64 sign/exponent/mantissa combinations, wide integers, long keys, and
collection-valued keys. The comparison accounted for the
[EOF and indentation interpretations](docs/conformance.md#parsing-details).

Span and comment tests cover borrowing, implicit nulls, aliases, enum tags,
original-byte mapping, parent/child overlap, ignored fields, document boundaries,
and resource limits. Compile-fail doctests check that wrappers cannot outlive
the input or satisfy `DeserializeOwned`.

## Fuzzing

The recorded runs used nightly `1.100.0-nightly (cea272fa3 2026-09-07)` and
cargo-fuzz 0.13.2. Each target ran for 61 seconds with a 2-second input timeout,
64 KiB input limit, and 2 GiB RSS limit. The counts below
are separate runs, not cumulative totals. No findings were reported in these runs.

Coverage percentages come from corpus replay with each target's matching binary
and profile. They measure fuzz coverage, separately from the test coverage above.
See the [fuzzing guide](fuzz/README.md#corpus-coverage) for reproduction commands.

### Default xtask run

The default `cargo xtask fuzz` command completed 6,293,732 executions:

| Target | Executions |
| --- | ---: |
| parse | 2,336,872 |
| serde | 504,187 |
| decode | 2,104,724 |
| typed | 1,347,949 |

### Expanded corpus (2026-09-25)

Seeds included all 416 official, regression, and real-world source files, all
five encodings, and all 29 typed-serde selectors. The four targets completed
4,101,751 executions. The serde target also read emitted strings and values with
`yaml_serde`.

| Target | Executions | Line coverage from corpus replay |
| --- | ---: | --- |
| parse | 1,591,706 | Parser 98.82% |
| serde | 337,350 | Parser 98.98% |
| decode | 1,262,857 | Parser 99.53%; decoder 89.19% |
| typed | 909,838 | Deserializer 92.92%; serializer 98.37%; comments 98.67%; wrapper 59.02% |

### After optimization (2026-09-25)

These runs include the serializer's UTF-8 copying optimization and completed
8,371,834 executions:

| Target | Executions | Line coverage from corpus replay |
| --- | ---: | --- |
| parse | 3,078,457 | Parser 98.20% |
| serde | 1,184,711 | Parser 98.98% |
| decode | 2,129,596 | Parser 99.53%; decoder 88.29% |
| typed | 1,979,070 | Deserializer 92.92%; serializer 98.37%; comments 98.67%; wrapper 59.02% |

### Spans and comments, before optimization (2026-09-25)

The typed target completed 1,803,697 executions; the serde target completed
973,639. The typed target checked all 29 type selections with `Spanned`, including
nested wrappers and comment indexing for tags and cyclic aliases.

Typed corpus replay reached 98.66% comment-index, 92.40% deserializer, 98.32%
serializer, and 59.02% wrapper line coverage. The parser and decoder results below
are from the pre-span baseline.

### Before spans (2026-09-24)

These runs completed 7,132,929 executions. Seeds included 407 source files, all
five encodings, and 24 serde type selections. Earlier runs found two scalar-slicing
panics; both have saved regression inputs.

| Target | Executions | Line coverage from corpus replay |
| --- | ---: | --- |
| parse | 2,270,594 | Parser 98.14% |
| serde | 1,142,347 | Parser 98.95% |
| decode | 1,541,102 | Parser 99.51%; decoder 88.29% |
| typed | 2,178,886 | Deserializer 96.19%; serializer 98.32%; schema 100% |

## Performance

The [benchmark notes](benchmarks/README.md) contain the workloads, toolchain,
measurement procedure, and results, with links to the raw CSV files. Recorded
runs cover the initial deserialization optimization, the cost of adding optional
span/comment support to unwrapped values, and the later throughput optimization.

A separate 30,000-item array diagnostic went from about 1.6 seconds to 0.02 seconds
on warm runs on the same host after caching line boundaries. This was a local
diagnostic, not a measurement from either benchmark harness.
