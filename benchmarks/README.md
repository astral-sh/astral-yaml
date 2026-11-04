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
