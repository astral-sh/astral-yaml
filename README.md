# astral-yaml

A fast, safe Rust YAML 1.2 parser.

> [!WARNING]
> This README is human-edited, but all code changes, PR summaries, and additional
> documentation were authored entirely by [Codex](https://openai.com/codex/).
> Use at your own risk.

Core parts:

- `astral-yaml`: parsing, syntax, source spans, and comments.
- `astral-yaml-serde`: serialization and deserialization.
- `astral-yaml-python`: Python bindings.

The parser targets [YAML 1.2.2](https://yaml.org/spec/1.2.2/).

See [conformance](docs/conformance.md) and [limitations](docs/limitations.md) for
supported behavior, and [SECURITY.md](SECURITY.md) for resource limits and
vulnerability reporting.

## Usage

The primary interfaces are:

- `astral-yaml` provides a low-level stream of syntax nodes with byte ranges,
  comment states, scalar styles, tags, and anchors. Aliases are not expanded.

- `astral-yaml-serde` provides a high-level serde-compatible serialization
  and deserialization interface. The serializer uses flow style and quoted
  strings.

- [`astral-yaml-python`](crates/astral-yaml-python/README.md) provides the
  `astral_yaml` module for reading and writing YAML with Python's built-in types.

See each crate's documentation for more information on usage.

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and checks.

## Performance

See [benchmarks](benchmarks/README.md) for workloads, comparison settings, and
commands. Pull requests include CodSpeed performance reports.

<div align="center">
  <a target="_blank" href="https://astral.sh" style="background:none">
    <img src="https://raw.githubusercontent.com/astral-sh/uv/main/assets/svg/Astral.svg" alt="Made by Astral">
  </a>
</div>
