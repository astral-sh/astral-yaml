# Real-world corpus

These are unmodified snapshots of eight GitHub Actions workflows, a Kubernetes
guestbook, an OpenAPI document, and Airflow and Sentry Compose configurations.
`manifest.json` records the repository, revision, original path, size, SHA-256, and retained
license for each file. Workflows exercise expressions, multiline shell scripts,
plain continuation lines, literal and folded scalars, flow matrices, empty
values, quotes, comments, and reusable jobs.
Airflow's configuration also exercises anchors, aliases, and merge-key syntax
(preserved as ordinary mappings; merge expansion is outside YAML 1.2 core).

Each fixture has an [inline Insta snapshot](../../xtask/tests/real_world/fixtures/)
in its Rust test. Accepted inputs render as indented trees: mapping entries stay
together, sequences use dashes, and scalar values appear beside their styles and
source ranges. Ranges use one-based Unicode line/column positions with exclusive
ends. Anchors, aliases, tags, collection styles, and document markers are retained.
Multiline values show one quoted decoded line at a time. Escapes make newlines
and tabs explicit; closing quotes expose trailing spaces.

For example:

```text
document 1 (start: implicit, end: implicit)
mapping [block]
  "name" [plain @1:1..1:5]: "CI" [single-quoted @1:7..1:11]
  "steps" [plain @2:1..2:6]: sequence [flow]
    - "test" [plain @2:9..2:13]
```

Normal tests check the inline snapshots without Python. The separate oracle test
runs PyYAML 6.0.3's Python and LibYAML-backed parsers; their complete syntax events
and UTF-8 scalar-token ranges must agree with each other and with our accepted
ASTs. These results are exchanged in memory, not stored as snapshot files.
Schema resolution is disabled, so YAML 1.1 spellings such as `on` remain scalar
text. `scripts/check.sh` runs this oracle test as well as the normal tests.

```sh
python3 scripts/verify-fixtures.py
cargo test -p xtask --test real_world
cargo test -p xtask --test real_world -- --ignored  # requires uv

# Review and update inline expectations after an intentional change:
cargo install cargo-insta --locked
cargo insta test -p xtask --test real_world --review
```

Tests also check LF/CRLF/CR equivalence and compare constructed values with
`yaml_serde`. Both readers also consume our serializer's output. Handwritten
assertions check shell content, YAML styles, and comments. Root `Spanned` values must collect every
comment in each single-document fixture. Benchmarks retain the original three
fixtures for comparable performance measurements.

`sentry-compose.yaml` is deliberately retained as a **rejection** case. Its
`seaweedfs.healthcheck.test` closing bracket is at the mapping key's indentation
(line 302), rather than inside the flow continuation. YAML 1.2.2 productions
63, 65, 79, 137, and 138 require greater indentation in this context. The
inline [rejection snapshot](../../xtask/tests/real_world/fixtures/sentry_compose.rs)
pins the diagnostic and byte range and shows the source line with a caret.
PyYAML accepts the file; the oracle test preserves that documented disagreement.
The source remains unmodified, including its anchors and aliases.
