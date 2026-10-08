# YAML conformance

`astral-yaml` supports [YAML 1.2.2](https://yaml.org/spec/1.2.2/).
`astral-yaml-serde` resolves values using its core schema.

YAML 1.1 behavior is not supported. In particular, `yes`, `no`, `on`, `off`, and
timestamps remain strings. The `<<` key is an ordinary mapping key and does not
merge mappings.

Block scalars have a few edge cases:

- EOF acts as an implicit final line break. For example, `"|\n  text"` produces
  `"text\n"`. This follows the YAML Test Suite's interpretation of an ambiguity
  in the specification ([discussion](https://github.com/yaml/yaml-test-suite/issues/118)).
- An indented `#` can be scalar content. Leading empty lines cannot be more
  indented than the first content line, even when that line starts with `#`
  ([discussion](https://github.com/yaml/yaml-test-suite/issues/37)).

See [limitations](limitations.md) for Serde-specific restrictions and
[security](../SECURITY.md#resource-limits) for resource limits.
