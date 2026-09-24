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
| uv CI workflow | 11,602 | 88.41 | 144.44 | 1.63× |
| Kubernetes Guestbook | 3,366 | 37.09 | 58.49 | 1.58× |
| OpenAPI Petstore | 5,479 | 47.44 | 73.40 | 1.55× |

These are measurements on one host, without CPU isolation or frequency
control. Re-run after parser or adapter changes; they do not establish performance
on other workloads or platforms. Completion still requires the remaining
correctness and coverage gates in `TRANSCRIPT.md`.
