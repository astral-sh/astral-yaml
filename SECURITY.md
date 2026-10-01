# Security

## Reporting a vulnerability

Report vulnerabilities according to [Astral's security policy](https://github.com/astral-sh/.github/blob/main/SECURITY.md).

## Processing untrusted input

`astral-yaml` and `astral-yaml-serde` forbid unsafe Rust in their own code. The
limits below constrain input size, nesting, and deserialization work; they do not
bound work performed by custom Serde `Serialize` or `Deserialize` implementations.
See [fuzzing](fuzz/README.md) for the parser and Serde fuzz targets.

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
limit. Serialization also limits nesting and checks non-string keys using the
parser's default limits.
