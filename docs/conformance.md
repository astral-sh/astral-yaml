# YAML 1.2 conformance

`astral-yaml` targets [YAML 1.2.2](https://yaml.org/spec/1.2.2/). The parser
preserves tags and aliases without constructing application values.
`astral-yaml-serde` uses the core schema and rejects duplicate mapping keys.
YAML 1.1 merge keys are not expanded. Timestamps and spellings such as `yes` and
`no` remain strings. See [limitations](limitations.md) and [resource
limits](../SECURITY.md#resource-limits).

The serde adapter accepts the core tags `!!str`, `!!null`, `!!bool`, `!!int`,
`!!float`, `!!seq`, and `!!map`, as well as the non-specific tag `!`. Core tags
must match their content. `!!null` resolves as `None` for `Option<T>`. Other
tags require an explicit opt-in, including on ignored fields and through
aliases. Enums use variant names for unit variants and singleton mappings
otherwise, unless Serde attributes select another representation.

`#[serde(with = "astral_yaml_serde::with::yaml_tag")]` on an enum field enables
local tags such as `!Text hello`. Unit variants serialize as empty tagged nodes.
Empty and null payloads deserialize as unit. The adapter also accepts ordinary
enum names and singleton mappings. For mixed tagged/untagged enums, it exposes
local tags to Serde's buffer as singleton mappings. It applies through options
and newtypes, but nested application tags require their own opt-in.

`Tagged<T>` captures an explicit local, global, or core tag with its typed
payload. Untagged input is rejected. Tag handles are expanded, and tag URIs
retain their percent-escape spelling. Core tags still validate the payload's
type.

Serialization also rejects duplicate mapping keys and core-tag/content mismatches.

`Source::from_bytes` accepts UTF-8, UTF-16, and UTF-32, with or without a BOM
and in either byte order where applicable. Source spans use decoded UTF-8 byte
offsets. `Source::original_span` maps them back to the original encoded bytes.

## Tests

The pinned [YAML Test Suite](../fixtures/yaml-test-suite/README.md) runs without
exclusions. Tests compare events, require rejection of invalid inputs, and check
spans for every UTF-8 prefix. Serde tests compare constructed values with the
suite's JSON expectations and check rejection of unsupported tags.
See [CONTRIBUTING.md](../CONTRIBUTING.md) for test commands.

## Parsing details

The parser follows the test suite's interpretation of EOF as an implicit final
line break. For example, `"|\n  text"` produces `"text\n"`. The specification is
ambiguous here. See [issue
118](https://github.com/yaml/yaml-test-suite/issues/118).

An indented `#` can be block-scalar content. Leading empty lines more indented
than that content are rejected. See suite case `S98Z` and [issue
37](https://github.com/yaml/yaml-test-suite/issues/37). A tab after the detected
block-scalar indentation is content (spec example 8.2).

## Spans and comments

[`Commented<'a, T>`](../README.md#usage) borrows a node's comments and those of its
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
