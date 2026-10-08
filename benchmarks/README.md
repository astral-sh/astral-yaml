# Benchmarks

Run the parser, deserializer, and serializer benchmarks from the repository root:

```sh
cargo bench -p astral-yaml-dev --bench yaml --locked
```

Add `--features editable` for editing benchmarks. Pass a filter after `--` to
select a workload, such as `-- uv-ci`.

To compare deserialization with other implementations, use the `compare` target
(requires Rust 1.89 or later):

```sh
cargo bench -p astral-yaml-dev --features compare --bench compare --locked
```

Pull requests receive CodSpeed reports. Add the `benchmarks:compare` label to
include comparisons with other implementations. Local `cargo bench` timings
measure elapsed time and are not directly comparable to CodSpeed's simulation.
