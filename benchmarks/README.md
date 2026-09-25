# Benchmarks

The results below were recorded on 2026-09-25 with Rust 1.98.1 on macOS arm64,
using the workspace's release profile. Measurements used one host without CPU
isolation or frequency control.

## Deserialization

```sh
cargo bench -p xtask --bench real_world
```

This compares `astral-yaml-serde` with `yaml_serde` 0.10.7 on three fixed
[fixtures](../fixtures/real-world/README.md). Both deserialize into
`Vec<yaml_serde::Value>`, with results checked for equality before timing.
Timings include parsing, value construction, and destruction.

Each time is the median of eleven samples lasting at least 200 ms, after warmup
and with alternating implementation order.

| Fixture | Bytes | `astral-yaml-serde` | `yaml_serde` | Speedup |
| --- | ---: | ---: | ---: | ---: |
| uv CI workflow | 11,602 | 50.64 µs | 145.10 µs | 2.87× |
| Kubernetes Guestbook | 3,366 | 22.30 µs | 59.30 µs | 2.66× |
| OpenAPI Petstore | 5,479 | 27.47 µs | 74.50 µs | 2.71× |

These are the `optimized` rows in [the recorded results](results/2026-09-25.csv).

## Spans and serialization

```sh
cargo bench -p xtask --bench throughput
```

This uses the same fixtures. `unspanned` deserializes into `yaml_serde::Value`;
`spanned` wraps each document root; `spanned-fields` also wraps each top-level
mapping value. Both wrapper modes collect descendant comments.

Deserialization includes parsing. Serialization starts from existing values.
All timings include destruction and use the median of eleven samples lasting
at least 150 ms.

Medians across the [three recorded runs](results/2026-09-25-throughput.csv), in µs:

| Fixture | Unspanned | Root wrapper | Root and field wrappers | Serialization |
| --- | ---: | ---: | ---: | ---: |
| uv CI workflow | 51.816 | 64.869 | 64.960 | 7.612 |
| Kubernetes Guestbook | 22.675 | 29.828 | 29.858 | 2.888 |
| OpenAPI Petstore | 27.976 | 28.248 | 28.087 | 3.418 |

These use the CSV's `candidate` rows. The harness also measures parsing alone
as `parse`; those measurements are not in this CSV.

## Comparing revisions

Build each revision with `cargo bench -p xtask --bench throughput --no-run` and
copy the printed executable to a separate path. Compare the two executables with:

```sh
uv run --script scripts/compare-throughput.py /path/to/baseline /path/to/candidate \
  --runs 3 --output /tmp/throughput.csv
```

The script alternates executable order and records each run. It fails if the
median paired time reduction is below 20% for any deserialization or serialization
workload. It does not compare `parse`.

Set `ASTRAL_BENCH_FIXTURE` and `ASTRAL_BENCH_MODE` to CSV identifiers to select a
fixture and operation in a throughput run. Set `ASTRAL_BENCH_PROFILE=1` to run each
selected operation for 20 seconds under a profiler.
