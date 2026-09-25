# YAML Test Suite

Vendored from the `data` branch of <https://github.com/yaml/yaml-test-suite>,
revision `6ad3d2c62885d82fc349026c136ef560838fdf3d` (also in `REVISION`).
All 402 `in.yaml` cases are included with their `test.event`, `error`, and `===`
files. The files are unmodified. `LICENSE` is the upstream MIT license from
main revision `da267a5c4782e7361e82889e76c0dc7df0e1e870`.

The conformance runner compares complete event streams for valid inputs and
requires an error for each invalid input. No cases are excluded. Source-span
invariants are checked on every successfully parsed case. Event formatting decodes
tag URI escapes to match the suite's event convention (case `6CK3`); the core AST
preserves the escaped tag spelling as required by YAML 1.2.2 section 5.6. Focused
regression tests independently verify that preservation.

```sh
cargo xtask conformance
```
