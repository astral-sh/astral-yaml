# Real-world deserialization benchmark

```sh
cargo bench -p xtask --bench real_world
```

Both implementations deserialize streams into `Vec<yaml_serde::Value>`.
The harness checks equality before timing. Samples include parsing,
value construction, and destruction. After warmup, the harness collects eleven
samples per implementation, each lasting at least 200 ms, with alternating order.
The reported latency is the median.

The comparator is pinned to `yaml_serde` 0.10.7. Fixtures, revisions, hashes, and
licenses are in `fixtures/real-world`. The workspace lockfile pins transitive
dependencies; the release profile uses thin LTO and one codegen unit.

Three consecutive runs on 2026-09-24 used Rust 1.98.1 on macOS arm64, with
duplicate-key and resource checks enabled. Each range contains the
three run medians; speedups pair measurements from the same run.

| Fixture | Bytes | astral (µs) | yaml_serde (µs) | Speedup |
| --- | ---: | ---: | ---: | ---: |
| uv CI workflow | 11,602 | 65.74–65.83 | 143.58–144.39 | 2.18–2.20× |
| Kubernetes Guestbook | 3,366 | 28.16–28.39 | 57.89–58.35 | 2.06–2.07× |
| OpenAPI Petstore | 5,479 | 35.00–35.33 | 72.62–73.77 | 2.08–2.09× |

The baseline at `31bd945` measured 1.63–1.65× on the same host.
[Run medians](results/2026-09-24.csv) record the baseline and all three runs.
Those comparisons used the same harness, fixtures, comparator, dependencies,
and release profile.

After adding optional span support, a 2026-09-25 regression run measured
2.20× / 2.06× / 2.09× on the same fixtures with ordinary, unwrapped values.
Adding borrowed descendant comments to `Spanned<'a, T>` measured
2.17× / 2.04× / 2.06×, also with unwrapped values.
[Run medians](results/2026-09-25.csv) record both checks using the same host,
toolchain, and methodology. These runs do not measure the cost of opting into
spans or comments.

The changes measured in the 2026-09-24 runs:

- Skip numeric scans for ordinary strings and reuse resolved scalar values in
  the serde visitor. Validate explicit tags without reclassifying untagged scalars.
- Check mapping syntax for tab indentation only when a tab is present. Scan
  plain-scalar content in byte runs ending at ASCII syntax characters, and use
  byte validation for ordinary ASCII input.
- Check small mappings with canonical keys on the stack. Larger mappings retain
  a hash set, and validation storage is released before visiting child values.

These measurements used one host without CPU isolation or frequency control.
Re-run for the workloads and platforms you need to compare.

## Comparing implementation revisions

```sh
cargo bench -p xtask --bench throughput
```

The throughput harness measures parsing, ordinary deserialization, root
`Spanned<Value>` deserialization, root and field wrappers, and serialization on
the same three fixtures. Deserialization includes parsing and dropping the result.
Serialization starts from values constructed outside timing and includes dropping
the output. Each result is the median of eleven samples lasting at least 150 ms.
Values, serialization round-trips, and root comment lists are checked before timing.

Save the executable printed by `cargo bench -p xtask --bench throughput --no-run`
for each revision, then compare them without rebuilding between measurements:

```sh
uv run --script scripts/compare-throughput.py /path/to/baseline /path/to/candidate \
  --runs 3 --output /tmp/throughput.csv
```

The script alternates execution order for each fixture and operation and records
every run. It reports median times and the median of paired candidate/baseline
ratios, and exits unsuccessfully if any deserialization or serialization workload
improves by less than 20%. The harness measures parsing separately; the comparison
script does not include it.

For focused measurements, set `ASTRAL_BENCH_FIXTURE` and `ASTRAL_BENCH_MODE` to
the names in the CSV output. Setting `ASTRAL_BENCH_PROFILE=1` runs each selected
operation for 20 seconds for use with a CPU profiler.

The 2026-09-25 comparison uses baseline `66c9da9`, which added the harness without
changing the implementation at `9ec4ce8`. Both executables used the same fixtures,
Rust 1.98.1, and release settings on macOS arm64. The measured changes:

- Cache implicit-key lookahead and line boundaries; scan ordinary scalar text in
  byte runs. Default tag prefixes no longer require per-document allocations.
- Share deserializer state and reuse nodes and scalar classifications already
  resolved by the caller. Small string-key mappings compare borrowed text directly.
- Store comment-index slots and line information in flat vectors, and omit empty
  comment ranges from the lookup table.
- Copy ordinary UTF-8 text into serialized output in chunks and format floats
  directly into the output buffer.

Three alternating runs on 2026-09-25 produced these median reductions in elapsed
time versus that baseline:

| Fixture | Unspanned | Root span | Root and field spans | Serialization |
| --- | ---: | ---: | ---: | ---: |
| uv CI workflow | 22.82% | 22.90% | 22.56% | 48.60% |
| Kubernetes Guestbook | 21.16% | 21.65% | 22.53% | 23.41% |
| OpenAPI Petstore | 22.36% | 22.54% | 22.12% | 45.81% |

All 36 paired measurements exceeded a 20% time reduction; the smallest was 20.08%.
The [raw measurements](results/2026-09-25-throughput.csv) record each pair.
The reference benchmark measured 2.87× / 2.66× / 2.71× speedups over `yaml_serde`,
recorded as `optimized` in [the reference results](results/2026-09-25.csv).
