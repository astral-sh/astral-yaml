# Benchmarks

The benchmarks use [CodSpeed's Divan adapter](https://codspeed.io/docs/reference/codspeed-rust/divan).
Run Astral's benchmarks from the repository root:

```sh
cargo bench -p astral-yaml-dev --bench yaml --locked
```

To compare implementations, enable the `compare` feature and select its target:

```sh
cargo bench -p astral-yaml-dev --features compare --bench compare --locked
```

Pass a filter after `--` to select benchmarks, or `--test` to run each once:

```sh
cargo bench -p astral-yaml-dev --bench yaml --locked -- uv-ci
cargo bench -p astral-yaml-dev --bench yaml --locked -- --test
```

## Workloads

Each workload processes all documents in one of the three fixed
[fixtures](../fixtures/real-world/README.md):

| Benchmark | Operation |
| --- | --- |
| `astral::parse` | Parse the YAML stream. |
| `astral::deserialize` | Parse and deserialize into `Vec<astral_yaml_dev::Value>`. |
| `astral::deserialize_spanned` | Parse and deserialize, collecting each document root's span and descendant comments. |
| `astral::deserialize_spanned_fields` | Also wrap each top-level mapping value in `Spanned`. |
| `astral::serialize` | Serialize previously deserialized values. |

`astral::deserialize_services_128` deserializes a generated configuration into
typed structs. All of these workloads run on Rust 1.85.

The `compare` target requires Rust 1.89 and compares deserialization with
`yaml_serde` and serde-saphyr. Names put the fixture before the implementation so
results for the same input appear together. For example, omitting the file path:

```text
deserialize::uv-ci[astral-yaml]
deserialize::uv-ci[serde-saphyr]
deserialize::uv-ci[yaml_serde]
```

Its `deserialize_services_128` group compares Astral and serde-saphyr on the
same typed-struct workload.

Fixture generation and correctness checks run outside measurements. Every
measurement includes destruction of the output. Span collection includes
descendant comments; serde-saphyr uses the same
[comparison settings](../docs/serde-saphyr.md#behavior-differences) as its tests.

## CodSpeed

[`benchmark.yml`](../.github/workflows/benchmark.yml) runs on pull requests and
pushes to `main`, and supports manual runs for backtesting. It uses CodSpeed's
simulation mode on a GitHub-hosted Linux runner and authenticates uploads with
OIDC. Enable the repository in CodSpeed to receive the reports; no token secret
is needed.

Runs include only Astral's benchmarks unless the PR has the `benchmarks:compare`
label or a manual dispatch enables **Compare implementations**. Adding or
removing a PR label reruns the workflow with the current selection. Pushes to
`main` run only Astral's benchmarks. To establish or refresh comparison baselines,
dispatch the workflow on `main` with **Compare implementations** enabled.

To check the instrumented suite locally:

```sh
uv run --only-dev --locked cargo codspeed build -p astral-yaml-dev --bench yaml --locked -m simulation
uv run --only-dev --locked cargo codspeed run -p astral-yaml-dev --bench yaml -m simulation
```

To include comparisons:

```sh
uv run --only-dev --locked cargo codspeed build -p astral-yaml-dev --features compare --bench yaml --bench compare --locked -m simulation
uv run --only-dev --locked cargo codspeed run -p astral-yaml-dev --bench yaml --bench compare -m simulation
```

Outside the CodSpeed runner, this checks that benchmarks execute but does not
collect performance measurements. Use `cargo bench` for local wall-clock timings.

## Second optimization run (2026-09-28)

Commit `0dbb599` is the baseline. The benchmark target and fixtures were unchanged.
Both executables used Rust 1.98.1 and the workspace release profile on macOS 26.7
with an Apple M5 Max (18 cores, 128 GiB RAM). CPU placement, frequency, and
background load were not controlled.

Each version ran five times, alternating order within each pair. Divan used
64 iterations per sample, a minimum of one second per workload, and a maximum
of 1.2 seconds. The table shows the median of each version's five run medians.
The mean time reduction across the 16 equally weighted workloads was **11.56%**.

| Workload | Before (µs) | After (µs) | Time reduction |
| --- | ---: | ---: | ---: |
| `deserialize/kubernetes-guestbook` | 14.460 | 13.780 | 4.7% |
| `deserialize/openapi-petstore` | 17.340 | 15.820 | 8.8% |
| `deserialize/uv-ci` | 32.780 | 27.780 | 15.3% |
| `deserialize_services_128` | 122.700 | 110.400 | 10.0% |
| `deserialize_spanned/kubernetes-guestbook` | 20.160 | 18.870 | 6.4% |
| `deserialize_spanned/openapi-petstore` | 17.340 | 16.010 | 7.7% |
| `deserialize_spanned/uv-ci` | 43.960 | 36.580 | 16.8% |
| `deserialize_spanned_fields/kubernetes-guestbook` | 21.170 | 19.920 | 5.9% |
| `deserialize_spanned_fields/openapi-petstore` | 17.450 | 16.010 | 8.3% |
| `deserialize_spanned_fields/uv-ci` | 44.400 | 36.760 | 17.2% |
| `parse/kubernetes-guestbook` | 8.323 | 7.407 | 11.0% |
| `parse/openapi-petstore` | 10.220 | 8.440 | 17.4% |
| `parse/uv-ci` | 19.570 | 14.490 | 26.0% |
| `serialize/kubernetes-guestbook` | 2.262 | 2.147 | 5.1% |
| `serialize/openapi-petstore` | 2.844 | 2.608 | 8.3% |
| `serialize/uv-ci` | 5.894 | 4.934 | 16.3% |

[Run medians](results/2026-09-28-hillclimb2.csv).

## First optimization run (2026-09-28)

Commit `e63f764` is the baseline for this run. The Astral benchmark target and
fixtures were unchanged. Both executables used Rust 1.98.1 and
the workspace release profile on macOS 26.7 with an Apple M5 Max (18 cores,
128 GiB RAM). CPU placement and frequency were not controlled.

Each version ran five times, alternating order within each pair. Divan used
64 iterations per sample, a minimum of one second per workload, and a maximum
of 1.2 seconds. The table shows the median of each version's five run medians.
All 16 workloads took at least 15% less time.

| Workload | Before (µs) | After (µs) | Time reduction |
| --- | ---: | ---: | ---: |
| `deserialize/kubernetes-guestbook` | 18.090 | 14.690 | 18.8% |
| `deserialize/openapi-petstore` | 22.810 | 17.360 | 23.9% |
| `deserialize/uv-ci` | 43.430 | 32.950 | 24.1% |
| `deserialize_services_128` | 152.700 | 121.200 | 20.6% |
| `deserialize_spanned/kubernetes-guestbook` | 24.770 | 20.060 | 19.0% |
| `deserialize_spanned/openapi-petstore` | 22.950 | 17.380 | 24.3% |
| `deserialize_spanned/uv-ci` | 56.660 | 44.220 | 22.0% |
| `deserialize_spanned_fields/kubernetes-guestbook` | 25.870 | 21.270 | 17.8% |
| `deserialize_spanned_fields/openapi-petstore` | 23.100 | 17.590 | 23.9% |
| `deserialize_spanned_fields/uv-ci` | 56.870 | 44.420 | 21.9% |
| `parse/kubernetes-guestbook` | 11.710 | 8.373 | 28.5% |
| `parse/openapi-petstore` | 15.450 | 10.170 | 34.2% |
| `parse/uv-ci` | 29.730 | 19.460 | 34.5% |
| `serialize/kubernetes-guestbook` | 2.940 | 2.259 | 23.2% |
| `serialize/openapi-petstore` | 3.515 | 2.859 | 18.7% |
| `serialize/uv-ci` | 8.033 | 5.861 | 27.0% |

[Run medians](results/2026-09-28-hillclimb.csv). To use the same sampling settings:

```sh
cargo bench -p astral-yaml-dev --bench yaml --locked -- --sample-size 64 --min-time 1 --max-time 1.2
```

## Recorded timings

The following results come from the previous timing harnesses. Value workloads
used `yaml_serde::Value`; current workloads use `astral_yaml_dev::Value`. The
typed workload is unchanged. These wall-clock measurements are not comparable
to CodSpeed's simulated timings.

The `real_world` and `throughput` results are from 2026-09-25, using Rust 1.98.1
and the workspace's release profile on one macOS arm64 host, without CPU isolation
or frequency control. All timings cover the entire fixture and include destruction
of the output.

### Deserialization

The benchmark checked that both implementations produced equal values before
timing parsing and deserialization. Times are medians of eleven samples of at
least 200 ms, after warmup, with alternating implementation order.

| Fixture | Bytes | `astral-yaml-serde` | `yaml_serde` 0.10.7 | Speedup |
| --- | ---: | ---: | ---: | ---: |
| uv CI workflow | 11,602 | 50.64 µs | 145.10 µs | 2.87× |
| Kubernetes Guestbook | 3,366 | 22.30 µs | 59.30 µs | 2.66× |
| OpenAPI Petstore | 5,479 | 27.47 µs | 74.50 µs | 2.71× |

Source: [`optimized` rows](results/2026-09-25.csv).

### serde-saphyr

This compares serde-saphyr 1.3.0 with Astral, using
[comparison settings](../docs/serde-saphyr.md#behavior-differences). Both
deserialize the three fixtures into vectors of values. A generated
configuration of 128 services also measures deserialization into typed structs.
Results are checked for equality before timing; parsing and destruction are
included. Comment capture and resource limits remain enabled.

Measured on 2026-09-28 with Rust 1.98.1, the workspace's release profile, and
macOS 26.7 on an Apple M5 Max (18 cores, 128 GiB RAM). CPU placement and frequency
were not controlled. Each run took eleven samples of at least 200 ms per
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

### Spans and serialization

`unspanned` deserialized each document into `yaml_serde::Value`. `spanned` added
`Spanned` to each document root; `spanned-fields` also wrapped each top-level mapping
value. Both collected descendant comments. Deserialization included parsing;
serialization started from existing values.

Each run took the median of eleven samples of at least 150 ms, after warmup.
The table shows the median of three runs, in µs.

| Fixture | `unspanned` | `spanned` | `spanned-fields` | `serialize` |
| --- | ---: | ---: | ---: | ---: |
| uv CI workflow | 51.816 | 64.869 | 64.960 | 7.612 |
| Kubernetes Guestbook | 22.675 | 29.828 | 29.858 | 2.888 |
| OpenAPI Petstore | 27.976 | 28.248 | 28.087 | 3.418 |

Source: [`candidate` rows](results/2026-09-25-throughput.csv), which omit `parse`.
