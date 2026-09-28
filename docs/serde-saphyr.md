# Comparison with serde-saphyr

Compared on 2026-09-28 against [serde-saphyr 1.3.0](https://github.com/bourumir-wyngs/serde-saphyr/tree/93e634c680c4ae4bff6a0b8f841077868725547d)
and its parser, granit-parser 1.3.0 (`e9941f06f9a1c43dba847808af356ec383f874e3`).
The dependency versions are pinned in `Cargo.lock`.

```sh
cargo test -p astral-yaml-dev --features saphyr --test saphyr --locked
cargo bench -p astral-yaml-dev --features saphyr --bench saphyr --locked
```

The comparison requires Rust 1.89 and runs in CI. The public crates retain their
Rust 1.85 minimum. See [benchmarks](../benchmarks/README.md#serde-saphyr) for timings
and measurement details.

## Conformance

Both parsers were tested against all 402 unmodified cases in the
[pinned YAML Test Suite](../fixtures/yaml-test-suite/README.md).

| Check | Astral | serde-saphyr / granit |
| --- | ---: | ---: |
| Valid syntax | 308/308 | 308/308 |
| Invalid syntax rejected | 94/94 | 93/94 |
| Valid event streams match the suite | 308/308 | 308/308 |
| Constructible JSON expectations match | 244/244 | 263/263 |
| Accepted real-world fixtures produce equal values | 11/11 | 11/11 |

Granit accepts `9C9N`, an under-indented flow sequence. Its upstream test
[explicitly permits this](https://github.com/bourumir-wyngs/serde-saphyr/blob/93e634c680c4ae4bff6a0b8f841077868725547d/tests/test_suite/y_9C9N.rs).
The Sentry Compose fixture exposes the same syntax difference. serde-saphyr later
rejects that fixture because its alias-to-anchor ratio exceeds the default budget.

Event comparison normalizes granit's numeric anchor IDs, implicit empty scalars
(represented as `~`), and document-end markers (which omit explicitness).
Collection styles, document starts, tags, aliases, and other scalar contents are
compared. Astral's event comparison needs none of these normalizations.

Of the 279 valid cases with JSON expectations, 35 contain explicit tags. Astral
rejects those inputs without an adapter. serde-saphyr accepts core tags and rejects
16 cases containing other tags with the comparison settings. The syntax tests
still check all cases. serde-saphyr's multi-document API drops null documents, so
its expected results omit those documents. Astral retains them.

Additional tests cover upstream regressions involving null-like strings, numeric
tags, root block scalars, aliases, duplicate and complex keys, enums, floating-point
limits, and Unicode escapes. Each serializer's output is read by both libraries
for 1,024 generated values, plus typed integer and floating-point boundaries.

## Fixes from this comparison

- JSON surrogate-pair escapes now decode to one Unicode scalar. Lone, reversed,
  and malformed surrogates remain errors.
- Finite floating-point literals that overflow `f32` or `f64` now return an error.
  Explicit `.inf` and `.nan` remain supported. Decimal and radix-integer `f32`
  values are converted at their requested precision to avoid double rounding.
- Local tags selecting scalar enum variants work inside `Option<T>` when the
  field opts in with `with::yaml_tag`.

These regressions also have tests in the public crates, without the comparison
feature. Line and function coverage remains 100% for both YAML crates.

## Behavior differences

The value comparisons use serde-saphyr's `strict_booleans: true`,
`merge_keys: AsOrdinary`, and `reject_unsupported_tags: true`. Comment capture and
resource limits retain their defaults. These settings do not make the APIs
identical:

- serde-saphyr accepts numeric extensions such as `0b11`, `+0xB`, `0XB`, and
  `1_000` even with these settings. Astral treats them as strings under the core
  schema. serde-saphyr's defaults also expand merge keys and accept YAML 1.1
  booleans.
- Astral rejects `{null: a, ~: b}` as duplicate keys; serde-saphyr 1.3.0 accepts it.
- Both accept `.inf` and `.nan` as typed floats. serde-saphyr rejects them through
  `deserialize_any` by default; Astral passes them to the visitor.
- Astral reads UTF-8, UTF-16, and UTF-32 with or without a BOM. The tested
  serde-saphyr reader accepts UTF-8 and BOM-prefixed UTF-16, but rejects BOM-less
  UTF-16 and UTF-32 in either byte order.
- Astral rejects explicit tags by default. `with::yaml_tag` opts an enum field
  into local tags, including mixed tagged/untagged enums. serde-saphyr accepts
  local enum tags by default; its strict tag setting rejects them. Enum tests
  use Astral's adapter and serde-saphyr's defaults.
- Core-tag tests use Astral's `Tagged<T>` wrapper, which captures the tag and
  typed payload. serde-saphyr accepts core tags without a wrapper. Its default
  generic deserialization drops application tags; Astral requires an explicit
  opt-in even when deserializing into `yaml_serde::Value`.
- serde-saphyr provides extensions such as binary decoding and includes. Those
  are outside Astral's current core-schema interface.

Span and comment wrappers have different ownership and attachment semantics and
are not compared by the timing benchmark.
