# Benchmarks

Run the parser, deserializer, and serializer benchmarks from the repository root:

```sh
cargo bench -p astral-yaml-dev --bench yaml --locked
```

Add `--features editable` for editing benchmarks. Pass a filter after `--` to
select a workload, such as `-- uv-ci`.

To compare parsing and deserialization with other implementations, use the `compare` target
(requires Rust 1.89 or later):

```sh
cargo bench -p astral-yaml-dev --features compare --bench compare --locked
```

The `parse` group counts documents, nodes, and comments across each fixture.
`astral-yaml` builds a syntax tree and reads the counts from it. `granit-parser`
counts events as it reads the stream, with strict indentation and comments enabled.
The benchmark checks that both parsers return the same counts before timing them.
The deserialization groups measure construction of values and typed structs separately.

Pull requests receive CodSpeed reports. Add the `benchmarks:compare` label to
include comparisons with other implementations. Local `cargo bench` timings
measure elapsed time and are not directly comparable to CodSpeed's simulation.

## Python bindings

Run the Python benchmarks locally with a release build of the extension:

```sh
uv run --locked --python 3.14 --no-editable pytest benchmarks --codspeed
```

Add `--group compare` before `pytest` to include PyYAML's safe loader and
dumper in both pure-Python and LibYAML versions. The `benchmarks:compare`
label enables the same comparisons in CI.

For local comparisons, read the per-call medians in nanoseconds from
`benchmarks[].stats.median_ns` in `.codspeed/results_*.json`.

Each public API is measured on a small configuration, the uv CI workflow, and
the Airflow Compose file, which uses anchors and aliases. `loads` reads text and
`load` reads a fresh binary stream. The dump benchmarks start from Python
objects constructed before timing. `dump` writes to a fresh text stream.

All loaders receive the original documents. PyYAML interprets the uv workflow's
`on` key as a boolean and expands the Airflow file's `<<` merge keys.
astral-yaml reads both as strings under the YAML 1.2 core schema. All dumpers
receive the same astral-yaml-loaded values, including shared objects. PyYAML
uses flow style, preserves insertion order, and allows Unicode. The emitters
choose different quoting and line wrapping.

For a profile-guided wheel, run the following on a host that can load the
target extension:

```sh
rustup component add llvm-tools-preview
MATURIN_PGO=1 UV_NO_DEFAULT_GROUPS=true uv build --wheel
```

Maturin builds an instrumented extension, runs the four APIs on a separate
training corpus, then builds the optimized wheel. The benchmark fixtures are
reserved for measuring the result.
