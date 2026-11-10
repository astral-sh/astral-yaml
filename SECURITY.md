# Security

## Reporting a vulnerability

Report vulnerabilities according to [Astral's security policy](https://github.com/astral-sh/.github/blob/main/SECURITY.md).

## Attacker model

An attacker should not be able to induce arbitrary code execution or uncontrolled resource usage
through astral-yaml. Being able to do either constitutes a potential security vulnerability.

## Processing untrusted input

The astral-yaml crates forbid unsafe Rust in their own code. The
limits below constrain input size, nesting, and deserialization work.

The parser is also fuzzed. See [fuzzing](fuzz/README.md) for the parser and Serde fuzz targets.

## Resource limits

The parser defaults to:

- 64 MiB of UTF-8 input and a separate 64 MiB budget for expanded shorthand tags.
- 64 levels of collection nesting.
- 1,000,000 syntax elements: nodes, comments, directives, BOMs, and document end
  markers.

Users may configure higher limits, subject to astral-yaml's upper limits.
See the documentation for more information on those limits and how to
configure them.

The Python bindings apply the parser's default limits to loading and dumping.
Loading also bounds the work spent on numeric mapping keys that share a
Python hash. Aliases reuse the Python object created for their anchor, so
repeated aliases do not multiply the number of constructed values.
