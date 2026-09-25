# Conformance and limits

The parser targets [YAML 1.2.2](https://yaml.org/spec/1.2.2/). It returns syntax and
tags without constructing application values. The serde adapter applies the core
schema, checks supported tags and duplicate mapping keys, and accepts local tags
as enum variants when an enum is requested. Other application-specific tags remain
in the AST but are not constructed by the adapter. YAML 1.1 merge keys, timestamps,
and boolean spellings have no special behavior.

## Tests

The pinned [YAML Test Suite](../fixtures/yaml-test-suite/README.md) has 402 cases:
308 valid inputs and 94 invalid inputs. Run it with
`cargo test -p astral-yaml --test conformance`. Tests compare every semantic event
for valid cases, require rejection of invalid cases, and check source spans for
every accepted UTF-8 prefix. No cases are excluded.

The [real-world corpus](../fixtures/real-world/README.md) has twelve pinned files.
Eleven have inline tree and scalar-range snapshots; Sentry's Compose file has a
rejection snapshot for an under-indented closing flow delimiter. An optional test
compares syntax events and scalar ranges with PyYAML's Python and LibYAML parsers.
Tests also check LF, CRLF, and CR input, and compare constructed values and
serialized output with `yaml_serde`.

Additional tests cover:

| Area | Checks |
| --- | --- |
| Characters and encodings (§5) | UTF-8/16/32, malformed code units, original-byte ranges, BOMs, quoted controls, escape repertoire |
| Directives and properties (§6) | Duplicate directives, handle scope, URI escaping, tag-prefix characters, alias properties, multiline anchors |
| Flow nodes (§7) | Key/value separation, leading-colon plain keys, JSON-style adjacency, implicit-pair key limits, continuation indentation |
| Block nodes (§8) | Implicit-key length and line limits, scalar folding/chomping, indentation, compact collections, comment separation |
| Streams (§9) | Document transitions, directive scope, explicit and standalone end-marker ranges |
| Core schema (§10.3) | Decimal leading zeros, radix integers, floats, null/bool spellings, 1.1-only forms remaining strings |
| Serde | All numeric widths, derived types, bytes, enums, aliases, optional source spans, duplicate-key identity, ignored fields, visitor protocols, resource limits |

Generated tests exercise 73,728 block scalar presentations and 3,258 quoted scalar
presentations, including whitespace, folding, chomping, escapes, and EOF. These
use `yaml_serde` as a comparator, with the exceptions below.

Serializer tests send every Unicode scalar value through both readers, check all
f32/f64 exponent fields with boundary mantissas and both signs, and cover wide
integers, signed zero, infinities, NaN, long keys, and collection-valued keys.
The serde fuzz target also reads serialized strings and values with `yaml_serde`
to check for defects shared by astral-yaml's parser and serializer.

## Parser disagreements

`yaml_serde` rejects the valid YAML 1.2 plain key in `{:foo}`. That case is checked
against the grammar instead. The suite event formatter decodes URI escapes to
match case `6CK3`; the public AST preserves tag URI spelling as required by §5.6.

The generated scalar comparisons account for these differences:

- The pinned suite treats EOF as an implicit final line break
  ([upstream issue 118](https://github.com/yaml/yaml-test-suite/issues/118)).
  The parser follows that interpretation. LibYAML does not insert the break, so
  the comparison supplies it to the reference reader. Issue 118 discusses the
  ambiguity in the specification's EOF productions.
- An indented `#` can begin block scalar content. Over-indented leading empty
  lines must be rejected, including before such content (suite `S98Z` and
  [upstream issue 37](https://github.com/yaml/yaml-test-suite/issues/37)). LibYAML
  can instead accept the input by treating that content as a comment.
- A tab following detected indentation is block scalar content (spec example
  8.2). LibYAML can reject it; direct tests check that the tab is kept as content.

## Spans and comments

`astral_yaml_serde::Spanned<'a, T>` retains the UTF-8 source span of a deserialized
node, borrows its comments, and dereferences to `T`. It can wrap fields, collections,
and enum payloads. The input must outlive the wrapper even when `T` is owned;
ordinary owned types can still satisfy `DeserializeOwned`. Serialization emits
the value alone. Serde buffering for `untagged` and `flatten` loses metadata;
wrap the containing enum or struct instead of fields decoded from a buffer.

Comments have one direct syntax owner, independent of the application's Rust type.
A wrapper collects its own and its syntax descendants' comments, including keys
and ignored fields, once per wrapper in source order. Nested wrapper lists overlap.
Placement always describes the relationship to the direct owner:

- Adjacent standalone comments lead the next field's value or sequence item at
  the same block indentation, or within the current flow collection.
- Same-line comments follow the preceding node. Comments after colons, sequence
  dashes, properties, and block-scalar headers belong to the corresponding value.
  Comments within or immediately following explicit keys belong to those keys.
- Comments after the final child belong to the enclosing collection. Comments
  after closing flow delimiters belong to that collection.
- Blank-line-separated comments and comments inside empty collections belong to
  the enclosing collection or document root as detached comments.

Comments before the first document belong to that document. Gaps between documents
and the stream suffix belong to the preceding document; directive and marker
comments belong to their document root. Comment-only streams retain
their comments through `Stream::comments()`. Text excludes `#` and preserves its
whitespace; spans include `#` and exclude line breaks. Attaching comments does not
expand node spans, so a comment can lie outside its owner's span.

Alias wrappers use the span and comments at the `*name` occurrence. Wrapped
children of aliased collections use metadata from the anchor's definition.
Comment collection does not follow alias targets, including cyclic references.

## Resource limits

Default parser limits are 64 MiB of UTF-8 input, 64 levels of nesting, and one
million syntax elements. `ParseOptions` can change these limits; nesting cannot
exceed 128. Nodes, comments, directives, BOMs, and document end markers count
toward the element limit. For encoded input, the byte limit applies to both the
original and decoded lengths. Serde also limits node visits and inspected scalar
bytes, including alias expansion and ignored values. Comment indexing is lazy;
each indexed line, node, and ownership record uses the same work budget. Every
comment emitted into a wrapper is charged again, including parent/child overlap.
Cyclic aliases remain representable in the syntax arena and produce a resource
error if deserialization expands them.
Serialization also bounds nesting. Numeric construction is limited by the requested
Rust type; lexical scalar classification does not impose a machine integer limit.

`scripts/check.sh` enforces formatting, clippy, Python linting and types, fixture
hashes, tests, and 100% line and function coverage for the two YAML crates. Region
coverage is reported separately. See the [fuzzing guide](../fuzz/README.md) for
target limits and corpus replay.
