# Real-world deserialization benchmark

```sh
cargo bench -p xtask --bench real_world
```

Both implementations deserialize complete streams into `Vec<yaml_serde::Value>`.
Before timing, the harness requires semantic equality. Samples include parsing,
value construction, and destruction. After warmup, the harness collects eleven
samples per implementation, each lasting at least 200 ms, with alternating order.
The reported latency is the median. This compares end-to-end deserialization;
it does not compare syntax parsing with value construction.

The comparator is pinned to `yaml_serde` 0.10.7. Fixtures, revisions, hashes, and
licenses are in `fixtures/real-world`. The workspace lockfile pins transitive
dependencies; the release profile uses thin LTO and one codegen unit.

Local results from three consecutive runs on 2026-09-24, Rust 1.98.1, macOS
arm64, including key-uniqueness and resource checks. Each range contains the
three run medians; speedups pair measurements from the same run.

| Fixture | Bytes | astral (µs) | yaml_serde (µs) | Speedup |
| --- | ---: | ---: | ---: | ---: |
| uv CI workflow | 11,602 | 65.74–65.83 | 143.58–144.39 | 2.18–2.20× |
| Kubernetes Guestbook | 3,366 | 28.16–28.39 | 57.89–58.35 | 2.06–2.07× |
| OpenAPI Petstore | 5,479 | 35.00–35.33 | 72.62–73.77 | 2.08–2.09× |

The baseline at `31bd945` measured 1.63–1.65× on the same host before this pass.
[Run medians](results/2026-09-24.csv) record that baseline and all three final
runs. Every final fixture result exceeds the 1.98× target. The harness, fixtures,
comparator, dependencies, and release profile are unchanged.

After adding optional span support, a 2026-09-25 regression run measured
**2.20× / 2.06× / 2.09×** on the same fixtures with ordinary, unwrapped values.
Adding borrowed descendant comments to `Spanned<'a, T>` measured
**2.17× / 2.04× / 2.06×**, also with unwrapped values.
[Run medians](results/2026-09-25.csv) record both checks using the same host,
toolchain, and methodology. These runs do not measure the cost of opting into
spans or comments.

The improvements remove repeated work:

- Skip numeric scans for ordinary strings and reuse resolved scalar values in
  the serde visitor. Validate explicit tags without reclassifying untagged scalars.
- Check mapping syntax for tab indentation only when a tab is present. Scan
  plain-scalar content in byte runs ending at ASCII syntax characters, and use
  byte validation for ordinary ASCII input.
- Check small mappings with canonical keys on the stack. Larger mappings retain
  a hash set, and validation storage is released before visiting child values.

These are measurements on one host, without CPU isolation or frequency
control. Re-run after parser or adapter changes; they do not establish performance
on other workloads or platforms. Validation and coverage results are recorded in
`TRANSCRIPT.md`.
