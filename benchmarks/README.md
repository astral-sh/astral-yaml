# Benchmarks

Run from the repository root:

```sh
cargo bench -p xtask --bench real_world
cargo bench -p xtask --bench throughput
```

`real_world` compares deserialization with `yaml_serde`. `throughput` measures
parsing, deserialization with and without spans, and serialization. Both use the
same three [fixtures](../fixtures/real-world/README.md).

Current value workloads use `astral_yaml_dev::Value`. The recorded value timings
below used `yaml_serde::Value` and predate that change. Typed workloads are unchanged.

The `real_world` and `throughput` results are from 2026-09-25, using Rust 1.98.1
and the workspace's release profile on one macOS arm64 host, without CPU isolation
or frequency control. All timings cover the entire fixture and include destruction
of the output.

## Deserialization

The benchmark checks that both implementations produce equal values before
timing parsing and deserialization. Times are medians of eleven samples of at
least 200 ms, after warmup, with alternating implementation order.

| Fixture | Bytes | `astral-yaml-serde` | `yaml_serde` 0.10.7 | Speedup |
| --- | ---: | ---: | ---: | ---: |
| uv CI workflow | 11,602 | 50.64 µs | 145.10 µs | 2.87× |
| Kubernetes Guestbook | 3,366 | 22.30 µs | 59.30 µs | 2.66× |
| OpenAPI Petstore | 5,479 | 27.47 µs | 74.50 µs | 2.71× |

Source: [`optimized` rows](results/2026-09-25.csv).

## serde-saphyr

```sh
cargo bench -p astral-yaml-dev --features saphyr --bench saphyr --locked
```

This compares serde-saphyr 1.3.0 with Astral, using
[comparison settings](../docs/serde-saphyr.md#behavior-differences). Both
deserialize the three fixtures into vectors of values. A generated
configuration of 128 services also measures deserialization into typed structs.
Results are checked for equality before timing; parsing and destruction are
included. Comment capture and resource limits remain enabled.

Measured on 2026-09-28 with Rust 1.98.1, the workspace's release profile, and
macOS 26.7 on an Apple M5 Max (18 cores, 128 GiB RAM). CPU placement and frequency
were not controlled. Each run takes eleven samples of at least 200 ms per
implementation after warmup, alternating order. The table shows the median of
three run medians.

| Fixture | Bytes | Output | Astral | serde-saphyr | Speedup |
| --- | ---: | --- | ---: | ---: | ---: |
| uv CI workflow | 11,602 | Values | 50.134 µs | 200.364 µs | 4.00× |
| Kubernetes Guestbook | 3,366 | Values | 22.356 µs | 107.813 µs | 4.82× |
| OpenAPI Petstore | 5,479 | Values | 27.144 µs | 120.758 µs | 4.45× |
| 128 services (generated) | 26,686 | Typed structs | 150.786 µs | 885.376 µs | 5.87× |

[Raw samples](results/2026-09-28-saphyr.csv) record time per iteration in ns and
iteration counts. These measurements include the correctness fixes described in
the [comparison](../docs/serde-saphyr.md); no performance optimizations were made.

## Spans and serialization

`unspanned` deserializes each document into `astral_yaml_dev::Value`. `spanned` adds
`Spanned` to each document root; `spanned-fields` also wraps each top-level mapping
value. Both collect descendant comments. Deserialization includes parsing;
serialization starts from existing values.

Each run takes the median of eleven samples of at least 150 ms, after warmup.
The table shows the median of three runs, in µs.

| Fixture | `unspanned` | `spanned` | `spanned-fields` | `serialize` |
| --- | ---: | ---: | ---: | ---: |
| uv CI workflow | 51.816 | 64.869 | 64.960 | 7.612 |
| Kubernetes Guestbook | 22.675 | 29.828 | 29.858 | 2.888 |
| OpenAPI Petstore | 27.976 | 28.248 | 28.087 | 3.418 |

Source: [`candidate` rows](results/2026-09-25-throughput.csv), which omit `parse`.

To select a workload, set `ASTRAL_BENCH_FIXTURE` to `uv-ci`,
`kubernetes-guestbook`, or `openapi-petstore`, and `ASTRAL_BENCH_MODE` to a mode
above. For profiling, `ASTRAL_BENCH_PROFILE=1` runs each selected operation for
20 seconds.

## Comparing revisions

Build each revision with `cargo bench -p xtask --bench throughput --no-run`.
Save each executable at a separate path, then run:

```sh
uv run --script scripts/compare-throughput.py /path/to/baseline /path/to/candidate \
  --runs 3 --output /tmp/throughput.csv
```

The script alternates executable order and writes the measurements to CSV,
excluding `parse`. For each workload, it pairs baseline and candidate times from
each run and requires a median time reduction of at least 20%. It exits with an
error if any workload misses that target.
