# YAML Test Suite

Vendored from the `data` branch of <https://github.com/yaml/yaml-test-suite>,
revision `6ad3d2c62885d82fc349026c136ef560838fdf3d` (also in `REVISION`).
All `in.yaml` cases are included with their `test.event`, `error`, `===`, and
available `in.json` files. The files are unmodified. `LICENSE` is the upstream MIT license from
main revision `da267a5c4782e7361e82889e76c0dc7df0e1e870`.

Each case’s `===` file names the behavior under test. Cases drawn from the
specification include its example number and title. For instance, `R4YG`
covers [Example 8.2, Block Indentation Indicator](https://yaml.org/spec/1.2.2/#example-block-indentation-indicator).

The parser's conformance test compares event streams for all valid inputs and
requires an error for each invalid input. It checks source-span invariants for
every accepted input or UTF-8 prefix. Event formatting decodes
tag URI escapes to match the suite's event convention (case `6CK3`). The core AST
preserves the escaped tag spelling. [YAML 1.2.2 §5.6](https://yaml.org/spec/1.2.2/#56-miscellaneous-characters)

```sh
cargo test -p astral-yaml --test conformance
```
