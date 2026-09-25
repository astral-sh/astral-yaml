# Conformance and limits

`astral-yaml` targets [YAML 1.2.2](https://yaml.org/spec/1.2.2/). The parser preserves
tags and aliases without constructing application values. `astral-yaml-serde`
uses the core schema and rejects duplicate mapping keys. YAML 1.1 merge keys are
not expanded; timestamps and spellings such as `yes` and `no` remain strings.

The serde adapter accepts local tags as enum variants when deserializing an enum.
Other application-specific tags are preserved by the parser but rejected by the
adapter. Tag URIs retain their percent-escape spelling.

`Source::from_bytes` accepts UTF-8, UTF-16, and UTF-32, with or without a BOM and in
either byte order where applicable. Source spans use decoded UTF-8 byte offsets;
`Source::original_span` maps them back to the original encoded bytes.

## Tests

All 402 cases in the pinned [YAML Test Suite](../fixtures/yaml-test-suite/README.md)
pass: 308 valid inputs and 94 invalid inputs, with no exclusions. Tests compare
events, require rejection of invalid inputs, and check spans for every UTF-8
prefix. See [CONTRIBUTING.md](../CONTRIBUTING.md) for commands and
[TRANSCRIPT.md](../TRANSCRIPT.md) for recorded test and fuzz results.

## Parser disagreements

The tests compare results with LibYAML and `yaml_serde`, with these exceptions:

- **EOF:** The parser follows the suite's interpretation of EOF as an implicit
  final line break. For example, `"|\n  text"` produces `"text\n"`. LibYAML omits
  the final break. The specification is ambiguous here; see
  [issue 118](https://github.com/yaml/yaml-test-suite/issues/118).
- **Block-scalar indentation:** An indented `#` can be scalar content. Leading
  empty lines more indented than that content must be rejected. LibYAML can
  accept them by treating the content as a comment. See suite case `S98Z` and
  [issue 37](https://github.com/yaml/yaml-test-suite/issues/37).
- **Tabs:** A tab after the detected block-scalar indentation is content
  (spec example 8.2). LibYAML can reject it when indentation is inferred.
- **Flow keys:** The parser accepts `{:foo}` as the key `:foo` with an empty value.
  `yaml_serde` rejects it.

The [Sentry Compose fixture](../fixtures/real-world/README.md) is another rejection
case: a closing flow delimiter is under-indented. PyYAML accepts it; astral-yaml
rejects it.

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

Deserialization also limits nesting, node visits, and inspected scalar bytes,
including alias expansion and ignored values. Collecting comments counts toward
the work limit, including repeated comments in parent and child wrappers. Cyclic
aliases can be parsed, but expanding them during deserialization hits a resource
limit. Serialization also limits nesting.

Numeric values must fit the requested Rust type. The parser can still recognize
numeric syntax outside machine integer ranges.
