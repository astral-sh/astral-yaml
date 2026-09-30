# Conformance and limits

`astral-yaml` targets [YAML 1.2.2](https://yaml.org/spec/1.2.2/). The parser preserves
tags and aliases without constructing application values. `astral-yaml-serde`
uses the core schema and rejects duplicate mapping keys. YAML 1.1 merge keys are
not expanded; timestamps and spellings such as `yes` and `no` remain strings.

The serde adapter accepts the core tags `!!str`, `!!null`, `!!bool`, `!!int`,
`!!float`, `!!seq`, and `!!map`, as well as the non-specific tag `!`. Core tags
must match their content; `!!null` resolves as `None` for `Option<T>`. Other tags
require an explicit opt-in, including on ignored fields and through aliases.
Enums use variant names for unit variants and singleton mappings otherwise,
unless Serde attributes select another representation.

Serde buffers values for untagged enums, internally tagged enums, and flattened
fields. If a plain scalar resolves as a number, boolean, or null, the buffer
loses its text and cannot deserialize it into a string. Quote the scalar or use
`!!str` to preserve it as a string in these contexts.

`#[serde(with = "astral_yaml_serde::with::yaml_tag")]` on an enum field enables
local tags such as `!Text hello`. Unit variants serialize as empty tagged nodes;
empty and null payloads deserialize as unit. The adapter also accepts ordinary
enum names and singleton mappings. For mixed tagged/untagged enums, it exposes
local tags to Serde's buffer as singleton mappings. It applies through options
and newtypes, but nested application tags require their own opt-in.

`Tagged<T>` captures an explicit local, global, or core tag with its typed
payload; untagged input is rejected. Tag handles are expanded, and tag URIs retain
their percent-escape spelling. Core tags still validate the payload's type.
Like `Spanned`, this wrapper requires direct access to the YAML deserializer;
Serde's intermediate buffers cannot retain this metadata.

Mapping-key comparison includes application tags. Application-tagged scalar keys
are compared by their tag and scalar text; the adapter cannot infer a custom
tag's canonical form. Applications that assign additional equivalences to these
keys must check them when constructing their values.

The duplicate check compares YAML keys before deserializing them into Rust.
Distinct YAML keys can deserialize into equal Rust keys. For example, in
`{1: first, "1": second}`, the keys are a YAML integer and string, but both
deserialize to `"1"` in a `BTreeMap<String, String>`. Serde's standard maps keep
the last value, so the result contains only `"1": "second"`.

To reject collisions between deserialized keys, apply a map adapter such as
[`serde_with::MapPreventDuplicates`](https://docs.rs/serde_with/latest/serde_with/struct.MapPreventDuplicates.html)
to each map that needs the check:

```rust
use serde::Deserialize;
use serde_with::{MapPreventDuplicates, serde_as};
use std::collections::BTreeMap;

#[serde_as]
#[derive(Deserialize)]
struct Config {
    #[serde_as(as = "MapPreventDuplicates<_, _>")]
    values: BTreeMap<String, String>,
}

let source = r#"values: {1: first, "1": second}"#;
assert!(astral_yaml_serde::from_str::<Config>(source).is_err());
```

`Source::from_bytes` accepts UTF-8, UTF-16, and UTF-32, with or without a BOM and in
either byte order where applicable. Source spans use decoded UTF-8 byte offsets;
`Source::original_span` maps them back to the original encoded bytes.

## Tests

All 402 cases in the pinned [YAML Test Suite](../fixtures/yaml-test-suite/README.md)
pass: 308 valid inputs and 94 invalid inputs, with no exclusions. Tests compare
events, require rejection of invalid inputs, and check spans for every UTF-8
prefix. Of the 279 cases with JSON expectations, the Serde tests compare 263
constructed values and check that the other 16 are rejected for non-core tags.
See [CONTRIBUTING.md](../CONTRIBUTING.md) for commands and
[TRANSCRIPT.md](../TRANSCRIPT.md) for recorded test and fuzz results.

## Parsing details

The parser follows the test suite's interpretation of EOF as an implicit final
line break. For example, `"|\n  text"` produces `"text\n"`. The specification is
ambiguous here; see [issue 118](https://github.com/yaml/yaml-test-suite/issues/118).

An indented `#` can be block-scalar content. Leading empty lines more indented
than that content are rejected; see suite case `S98Z` and
[issue 37](https://github.com/yaml/yaml-test-suite/issues/37). A tab after the
detected block-scalar indentation is content (spec example 8.2).

## Spans and comments

[`Spanned<'a, T>`](../README.md#usage) borrows a node's comments and those of its
syntax descendants, including keys and ignored fields. Each wrapper collects
comments once, in source order. Parent and child lists overlap.

Each comment has one direct owner, determined by YAML syntax. Its placement stays
relative to that owner when collected by a parent wrapper:

- Adjacent standalone comments lead the next field's value or sequence item at
  the same block indentation, or within the current flow collection.
- Same-line comments follow the preceding node. Comments after colons, sequence
  dashes, properties, and block-scalar headers belong to the corresponding value.
  Comments within or immediately following explicit keys belong to those keys.
- Comments after the final child belong to the enclosing collection. Comments
  after closing flow delimiters belong to that collection.
- Blank-line-separated comments and comments inside empty collections belong to
  the enclosing collection or document root as detached comments.

Comments before the first document belong to its root. Comments between documents
or after the last document belong to the preceding root. Directive and marker
comments belong to their document's root. Use `Stream::comments()` for streams
containing only comments.

Comment text excludes `#` and preserves whitespace after it. Comment spans include
`#` and exclude the line break. Comments can lie outside their owner's span.

Alias wrappers use the span and comments at the `*name` occurrence. Wrapped
children of aliased collections use metadata from the anchor's definition.
Comment collection does not follow alias targets, including cyclic references.

## Resource limits

The parser defaults to:

- 64 MiB of UTF-8 input.
- 64 levels of collection nesting.
- 1,000,000 syntax elements: nodes, comments, directives, BOMs, and document end
  markers.

`ParseOptions` controls these limits; nesting must be between 1 and 128.
For encoded input, `Source::with_limit` bounds both the original and decoded byte
lengths.

Deserialization also limits nesting, node visits, and inspected scalar and tag bytes,
including alias expansion and ignored values. Collecting comments counts toward
the work limit, including repeated comments in parent and child wrappers. Cyclic
aliases can be parsed, but expanding them during deserialization hits a resource
limit. Serialization also limits nesting.

Numeric values must fit the requested Rust type. The parser can still recognize
numeric syntax outside machine integer ranges.

Mapping-key comparison uses 128-bit integer magnitudes and 64-bit floats,
independently of the destination key type. Serialization also rejects duplicate
keys and core-tag/content mismatches. Non-string keys are checked using the
parser's default limits.
