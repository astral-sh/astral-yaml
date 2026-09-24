# Real-world deserialization benchmark

```sh
cargo bench -p astral-yaml-dev --bench real_world
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

Current local result, 2026-09-24, Rust 1.98.1, macOS arm64 (including key-uniqueness and resource checks):

| Fixture | Bytes | astral (µs) | yaml_serde (µs) | Speedup |
| --- | ---: | ---: | ---: | ---: |
| uv CI workflow | 11,602 | 91.31 | 147.10 | 1.61× |
| Kubernetes Guestbook | 3,366 | 36.84 | 59.66 | 1.62× |
| OpenAPI Petstore | 5,479 | 45.99 | 74.62 | 1.62× |

These are measurements on one host, without CPU isolation or frequency
control. Re-run after parser or adapter changes; they do not establish performance
on other workloads or platforms. Validation and coverage results are recorded in
`TRANSCRIPT.md`.
