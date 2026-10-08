# Benchmarks

The benchmarks use [CodSpeed's Divan adapter](https://codspeed.io/docs/reference/codspeed-rust/divan).
Run Astral's benchmarks from the repository root:

```sh
cargo bench -p astral-yaml-dev --bench yaml --locked
```

Add `--features editable` to include the editing benchmarks.

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
| `astral::deserialize_spanned` | Parse and deserialize, collecting each document root's span. |
| `editable::{parse,unchanged_output,replace_scalar,insert_entry}` | Owned syntax construction, unchanged output, and value-based edit transactions on a small configuration. Edit cases include construction. |
| `astral::deserialize_spanned_fields` | Also wrap each top-level mapping value in `Spanned`. |
| `astral::serialize` | Serialize previously deserialized values. |

`astral::deserialize_services_128` deserializes a generated configuration into
typed structs. All of these workloads run on Rust 1.85.

The `compare` target requires Rust 1.89 and compares deserialization with
`deser-yaml`, `yaml_serde`, and serde-saphyr. Names put the fixture before the
implementation so results for the same input appear together. For example,
omitting the file path:

```text
deserialize::uv-ci[astral-yaml]
deserialize::uv-ci[deser-yaml]
deserialize::uv-ci[serde-saphyr]
deserialize::uv-ci[yaml_serde]
```

Its `deserialize_services_128` group compares Astral, `deser-yaml`, and
serde-saphyr on the same typed-struct workload.

Fixture generation and correctness checks run outside measurements. Every
measurement includes destruction of the output. The dynamic workloads use
`astral_yaml_dev::Value` for the Serde implementations and `deser_value::Value`
for `deser-yaml`. The latter also retains scalar spellings and formatting hints.
`Spanned` collects source ranges without collecting descendant comments. `deser-yaml` uses its default
features and YAML 1.2 with merge keys disabled and location tracking off. It
does not collect comments. Astral's parser records spans and comments even for
unspanned deserialization. serde-saphyr uses `strict_booleans: true`,
`merge_keys: AsOrdinary`, and `reject_unsupported_tags: true`. Comment capture
and resource limits use its defaults.

## CodSpeed

[`benchmark.yml`](../.github/workflows/benchmark.yml) runs on pull requests and
pushes to `main`, and supports manual runs for backtesting. It uses CodSpeed's
simulation mode on a GitHub-hosted Linux runner and authenticates uploads with
OIDC. Enable the repository in CodSpeed to receive the reports. No token secret
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
uv run --only-dev --locked cargo codspeed build -p astral-yaml-dev --features compare,editable --bench yaml --bench compare --locked -m simulation
uv run --only-dev --locked cargo codspeed run -p astral-yaml-dev --bench yaml --bench compare -m simulation
```

Outside the CodSpeed runner, this checks that benchmarks execute but does not
collect performance measurements. Use `cargo bench` for local wall-clock timings.

## Implementation comparison (2026-09-30)

These measurements use `deser-yaml` 0.9.1, `yaml_serde` 0.10.7, and
serde-saphyr 1.3.0, with Astral's source at `d00cbaf`. They ran on macOS 26.7
with an Apple M5 Max (18 cores, 128 GiB RAM), Rust 1.98.1, and the workspace
release profile. CPU placement and frequency were not controlled.

The table shows the median of six run medians. Divan used 64 iterations per
sample, a minimum of one second per workload, and a maximum of 1.2 seconds. Runs
alternated between ascending and descending benchmark order. Times include
parsing, construction, and destruction. The three fixture rows use dynamic
values with the representation difference described above. The service row uses
the same structs with native derives for Serde and Deser.

| Workload | Bytes | Astral | `deser-yaml` | `yaml_serde` | serde-saphyr |
| --- | ---: | ---: | ---: | ---: | ---: |
| uv CI workflow | 11,602 | 26.685 µs | 70.925 µs | 136.850 µs | 189.650 µs |
| Kubernetes Guestbook | 3,366 | 12.470 µs | 35.680 µs | 54.555 µs | 101.800 µs |
| OpenAPI Petstore | 5,479 | 14.805 µs | 44.885 µs | 69.250 µs | 114.300 µs |
| 128 services (typed) | 26,686 | 107.950 µs | 229.600 µs | — | 887.050 µs |

To repeat one round, use `--sort name`. Use `--sortr name` to reverse it:

```sh
cargo bench -p astral-yaml-dev --features compare --bench compare --locked -- --sample-size 64 --min-time 1 --max-time 1.2 --sort name
```
