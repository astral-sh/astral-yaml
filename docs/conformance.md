# Conformance and limits

The parser targets [YAML 1.2.2](https://yaml.org/spec/1.2.2/). The pinned YAML Test
Suite contains 402 cases: 308 valid inputs and 94 malformed inputs. Tests compare
all semantic events for valid cases, require rejection of invalid cases, and
validate spans for every accepted UTF-8 prefix of every corpus input. No cases
are excluded. Three pinned real-world documents are also checked against
`yaml_serde` and round-tripped through the serde adapter.

Additional regressions cover requirements that the corpus does not exhaust:

| Area | Checks |
| --- | --- |
| Characters and encodings (§5) | UTF-8/16/32, malformed code units, original-byte ranges, BOMs, quoted controls, escape repertoire |
| Directives and properties (§6) | Duplicate directives, handle scope, URI escaping, tag-prefix characters, alias properties, multiline anchors |
| Flow nodes (§7) | Key/value separation, leading-colon plain keys, JSON-style adjacency, implicit-pair key limits, continuation indentation |
| Block nodes (§8) | Implicit-key length and line limits, scalar folding/chomping, indentation, compact collections, comment separation |
| Streams (§9) | Document transitions, directive scope, explicit and standalone end-marker ranges |
| Core schema (§10.3) | Decimal leading zeros, radix integers, floats, null/bool spellings, 1.1-only forms remaining strings |
| Serde | All numeric widths, derived types, bytes, enums, aliases, duplicate-key identity, ignored fields, visitor protocols, resource limits |

`yaml_serde` is a performance comparator and a useful check on shared syntax;
it is not the conformance authority. For example, it rejects the valid YAML 1.2
plain key in `{:foo}`. That case is checked directly against the grammar. The
suite event formatter also decodes URI escapes to match case `6CK3`, while the
public AST preserves the tag URI spelling required by section 5.6.

The core returns syntax and tags without constructing application values. The
serde adapter applies the core schema, validates supported tags and duplicate
mapping keys, and recognizes local tags as enum variants when an enum is
requested. Other application-specific tags remain available in the AST but are
not constructed by the adapter. YAML 1.1 merge keys, timestamps, and boolean
spellings have no special behavior.

Default parser limits are 64 MiB of UTF-8 input, 64 levels of nesting, and one
million syntax elements. `ParseOptions` can change these limits; nesting has a
hard maximum of 128. Nodes, comments, directives, BOMs, and document end markers
consume the element budget. Encoded input bounds both original and decoded byte
lengths. Serde additionally bounds node visits and inspected scalar bytes,
including alias expansion and ignored values. Cyclic aliases remain representable
in the syntax arena and produce a resource error if deserialization expands them.
Serialization also bounds nesting. Numeric construction is limited by the requested
Rust type; lexical scalar classification does not impose a machine integer limit.

`scripts/check.sh` enforces formatting, strict clippy, fixture hashes, tests, and
100% line and function coverage. Region coverage is reported separately. Fuzzing
uses bounded parser, decoder, generic-serde, and typed-serde targets; see
[the fuzzing guide](../fuzz/README.md). Coverage is evidence about exercised code,
not proof that every possible YAML input is handled correctly.
