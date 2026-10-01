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

- 64 MiB of UTF-8 input.
- 64 levels of collection nesting.
- 1,000,000 syntax elements: nodes, comments, directives, BOMs, and document end
  markers.

Users may configure higher limits, subject to astral-yaml's upper limits.
See the documentation for more information on those limits and how to
configure them.
