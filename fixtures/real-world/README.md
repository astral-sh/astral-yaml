# Real-world corpus

These are unmodified snapshots of eight GitHub Actions workflows, a Kubernetes
guestbook, an OpenAPI document, and Airflow and Sentry Compose configurations.
`manifest.json` records each file's repository, revision, original path, size,
SHA-256, and license. Workflows exercise expressions, multiline shell scripts,
plain continuation lines, literal and folded scalars, flow matrices, empty
values, quotes, comments, and reusable jobs.
Airflow's configuration also exercises anchors, aliases, and `<<` mapping keys.
The parser treats `<<` as an ordinary key; YAML 1.2 core does not define merge
expansion.

Each fixture has an [inline Insta snapshot](../../crates/astral-yaml/tests/real_world/fixtures/)
in a parser integration test. Accepted inputs render as indented trees: mapping
entries stay together, sequences use dashes, and scalar values appear beside their
styles and source ranges. Ranges use one-based Unicode line/column positions with
exclusive ends. Anchors, aliases, tags, collection styles, and document markers
are retained.
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

Snapshot tests do not require Python. An optional test compares syntax events and
UTF-8 scalar-token ranges with PyYAML 6.0.3's Python and LibYAML parsers. Both must
agree with each other and with each accepted AST. The test generates these
expectations at runtime. It does not construct values or resolve schema types,
so YAML 1.1 spellings such as `on` remain scalar text. `scripts/check.sh` runs
this comparison as well as both crates' normal tests.

```sh
uv run --script scripts/verify-fixtures.py
cargo test -p astral-yaml --test real_world
cargo test -p astral-yaml-serde --test real_world
cargo test -p astral-yaml --test real_world -- --ignored  # requires uv

# Review snapshot changes:
cargo install cargo-insta --locked
cargo insta test -p astral-yaml --test real_world --review
```

Parser tests also check LF/CRLF/CR equivalence. Serde tests compare values with
`yaml_serde` and read serialized output with both readers. They check that root
`Spanned` values collect every comment in each accepted single-document fixture.
Other assertions check shell content, YAML styles, and comments. Benchmarks use
the original three fixtures so results remain comparable.

`sentry-compose.yaml` is a rejection case. The closing bracket in
`seaweedfs.healthcheck.test` is at the mapping key's indentation
(line 302), rather than inside the flow continuation. YAML 1.2.2 productions
63, 65, 79, 137, and 138 require greater indentation in this context. The
inline [rejection snapshot](../../crates/astral-yaml/tests/real_world/fixtures/sentry_compose.rs)
records the diagnostic, byte range, and source line. PyYAML accepts the file;
the comparison test checks this disagreement. The fixture is unmodified.
