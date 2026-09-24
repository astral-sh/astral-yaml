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

Initial local result, 2026-09-24, Rust 1.98.1, macOS arm64:

| Fixture | Bytes | astral (µs) | yaml_serde (µs) | Speedup |
| --- | ---: | ---: | ---: | ---: |
| uv CI workflow | 11,602 | 78.19 | 147.08 | 1.88× |
| Kubernetes Guestbook | 3,366 | 30.84 | 59.50 | 1.93× |
| OpenAPI Petstore | 5,479 | 39.35 | 73.93 | 1.88× |

These are initial measurements on one host, without CPU isolation or frequency
control. Re-run after parser or adapter changes; they do not establish performance
on other workloads or platforms. Completion still requires the remaining
correctness and coverage gates in `TRANSCRIPT.md`.
