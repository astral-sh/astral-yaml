# YAML conformance

`astral-yaml` parses [YAML 1.2.2](https://yaml.org/spec/1.2.2/).
`astral-yaml-serde` uses the YAML 1.2 core schema to resolve scalar values.

## Scalars and mapping keys

YAML 1.1 is unsupported. In particular, the plain scalars `yes`, `no`, `on`,
`off`, and timestamps such as `2001-01-01` resolve as strings. Use `true` or
`false` for booleans. The key `<<` is an ordinary string key, so it does not
merge mappings.

## Block scalars

A block scalar can include a final newline even when the file has none.
For example, `"|\n  text"` decodes to `"text\n"` because the parser treats
the end of the file as a line break. This follows the YAML Test Suite's
interpretation of an ambiguity in the specification. [YAML Test Suite issue #118](https://github.com/yaml/yaml-test-suite/issues/118)

An indented `#` can be part of a block scalar. Here `message` has the value
`"# heading\n"`.

```yaml
message: |
  # heading
```

When the scalar's indentation is inferred, leading empty lines cannot be more
indented than the first content line. That rule also applies when the first
content line starts with `#`. [YAML Test Suite issue #37](https://github.com/yaml/yaml-test-suite/issues/37)

See [limitations](limitations.md) for how Serde handles buffered values and
mapping keys, and [resource limits](../SECURITY.md#resource-limits) for limits
on input and processing.
