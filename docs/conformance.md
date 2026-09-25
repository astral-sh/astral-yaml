# Conformance and limits

The parser targets [YAML 1.2.2](https://yaml.org/spec/1.2.2/). The pinned YAML Test
Suite contains 402 cases: 308 valid inputs and 94 malformed inputs. Tests compare
all semantic events for valid cases, require rejection of invalid cases, and
validate spans for every accepted UTF-8 prefix of every corpus input. No cases
are excluded. The [real-world corpus](../fixtures/real-world/README.md) contains
twelve pinned files, including eight GitHub Actions workflows. Eleven valid files
have readable tree and scalar-range snapshots stored inline in `astral-yaml`'s
integration tests with Insta. A separate oracle test in `xtask` compares complete
syntax events and UTF-8 scalar ranges with both PyYAML's Python parser and LibYAML,
without stored expectations.
Tests also check all three YAML line endings and compare constructed values and
serialized output with `yaml_serde` in `astral-yaml-serde`'s integration tests.
Sentry's Compose file is retained as an explicit rejection case because a closing
flow delimiter is under-indented.

Additional regressions cover requirements that the corpus does not exhaust:

| Area | Checks |
| --- | --- |
| Characters and encodings (§5) | UTF-8/16/32, malformed code units, original-byte ranges, BOMs, quoted controls, escape repertoire |
| Directives and properties (§6) | Duplicate directives, handle scope, URI escaping, tag-prefix characters, alias properties, multiline anchors |
| Flow nodes (§7) | Key/value separation, leading-colon plain keys, JSON-style adjacency, implicit-pair key limits, continuation indentation |
| Block nodes (§8) | Implicit-key length and line limits, scalar folding/chomping, indentation, compact collections, comment separation |
| Streams (§9) | Document transitions, directive scope, explicit and standalone end-marker ranges |
| Core schema (§10.3) | Decimal leading zeros, radix integers, floats, null/bool spellings, 1.1-only forms remaining strings |
| Serde | All numeric widths, derived types, bytes, enums, aliases, optional source spans, duplicate-key identity, ignored fields, visitor protocols, resource limits |

`yaml_serde` is a performance comparator and a useful check on shared syntax;
it is not the conformance authority. For example, it rejects the valid YAML 1.2
plain key in `{:foo}`. That case is checked directly against the grammar. The
suite event formatter also decodes URI escapes to match case `6CK3`, while the
public AST preserves the tag URI spelling required by section 5.6.

Generated tests exercise 73,728 block scalar presentations and 3,258 quoted scalar
presentations, including whitespace, folding, chomping, escapes, and EOF. These
comparisons account for documented differences instead of treating the reference
parser as authoritative:

- The pinned suite and its maintainer's guidance treat EOF as an implicit final
  line break ([upstream issue 118](https://github.com/yaml/yaml-test-suite/issues/118)).
  We retain that behavior. LibYAML does not insert the break; the generated
  comparison supplies it explicitly to the reference reader. This remains an
  upstream specification ambiguity, not evidence of universal agreement.
- An indented `#` can begin block scalar content. Over-indented leading empty
  lines must be rejected, including before such content (suite `S98Z` and
  [upstream issue 37](https://github.com/yaml/yaml-test-suite/issues/37)). LibYAML
  can incorrectly treat the content as a comment instead.
- A tab following detected indentation is block scalar content (spec example
  8.2). LibYAML can reject it; direct tests preserve the specified interpretation.

Serializer checks send every Unicode scalar value through both readers, exercise
all f32/f64 exponent fields with boundary mantissas and both signs, and verify
wide integers, signed zero, infinities, NaN, long keys, and collection-valued keys.
The serde fuzz target also uses the independent reader for emitted strings and
values, preventing a shared parser/emitter defect from hiding in a self-round-trip.

The core returns syntax and tags without constructing application values. The
serde adapter applies the core schema, validates supported tags and duplicate
mapping keys, and recognizes local tags as enum variants when an enum is
requested. Other application-specific tags remain available in the AST but are
not constructed by the adapter. YAML 1.1 merge keys, timestamps, and boolean
spellings have no special behavior.

`astral_yaml_serde::Spanned<'a, T>` retains the UTF-8 source span of a deserialized
node, borrows its comments, and dereferences to `T`. Wrappers compose with fields,
collections, and enum payloads. Their lifetime is tied to the input even when `T`
is owned; ordinary owned types continue to satisfy `DeserializeOwned`. Serialization
emits the value alone. Serde buffering for `untagged` and `flatten` loses metadata;
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

Stream-leading comments enter the first document's attachment context. Gaps
between documents and the stream suffix belong to the preceding document; directive
and marker comments belong to their document root. Comment-only streams retain
their comments through `Stream::comments()`. Text excludes `#` and preserves its
whitespace; spans include `#` and exclude line breaks. Attaching comments does not
expand node spans, so a comment can lie outside its owner's span.

Aliases are leaves for aggregation. Alias spans and comments identify the use
site; wrapped children of aliased collections retain definition-site metadata.
Aggregation does not follow alias references, including cyclic references.

Default parser limits are 64 MiB of UTF-8 input, 64 levels of nesting, and one
million syntax elements. `ParseOptions` can change these limits; nesting has a
hard maximum of 128. Nodes, comments, directives, BOMs, and document end markers
consume the element budget. Encoded input bounds both original and decoded byte
lengths. Serde additionally bounds node visits and inspected scalar bytes,
including alias expansion and ignored values. Comment indexing is lazy and charges
each indexed line, node, and ownership record against the same work budget. Every
comment emitted into a wrapper is charged again, including parent/child overlap.
Cyclic aliases remain representable in the syntax arena and produce a resource
error if deserialization expands them.
Serialization also bounds nesting. Numeric construction is limited by the requested
Rust type; lexical scalar classification does not impose a machine integer limit.

`scripts/check.sh` enforces formatting, strict clippy, fixture hashes, tests, and
100% line and function coverage. Region coverage is reported separately. Fuzzing
uses bounded parser, decoder, generic-serde, and typed-serde targets; see
[the fuzzing guide](../fuzz/README.md). Coverage is evidence about exercised code,
not proof that every possible YAML input is handled correctly.
