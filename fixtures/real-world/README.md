# Real-world corpus

The corpus contains unmodified application and infrastructure configurations,
including Kubernetes resources, Compose applications, CI workflows, and OpenAPI
documents.

`manifest.json` records each file's repository, revision, original path, size,
SHA-256, category, and license. Revisions are full commit IDs. License files and
upstream notices are included alongside the inputs. Selection covered application
and infrastructure configurations. Byte-identical copies and
templates that require rendering before they are YAML were excluded.

The inputs exercise expressions, multiline scripts, plain continuation lines,
literal and folded scalars, flow collections, multiple documents, empty values,
quotes, comments, anchors, aliases, and application tags.
Airflow's configuration also exercises anchors, aliases, and `<<` mapping keys.
The parser treats `<<` as an ordinary key. YAML 1.2 core does not define merge
expansion.

Each fixture has an [inline Insta snapshot](../../crates/astral-yaml/tests/real_world/fixtures/)
in a parser integration test. Accepted inputs render as indented trees: mapping
entries stay together, sequences use dashes, and scalar values appear beside their
styles and source ranges. Ranges use one-based Unicode line/column positions with
exclusive ends. Anchors, aliases, tags, collection styles, and document markers
are retained.
Multiline values show one quoted decoded line at a time. Escapes make newlines
and tabs explicit. Closing quotes expose trailing spaces.

For example:

```text
document 1 (start: implicit, end: implicit)
mapping [block]
  "name" [plain @1:1..1:5]: "CI" [single-quoted @1:7..1:11]
  "steps" [plain @2:1..2:6]: sequence [flow]
    - "test" [plain @2:9..2:13]
```

```sh
uv run --script scripts/verify-fixtures.py
cargo test -p astral-yaml --test real_world
cargo test -p astral-yaml-serde --test real_world

# Review snapshot changes:
cargo install cargo-insta --locked
cargo insta test -p astral-yaml --test real_world --review
```

Parser tests also check LF/CRLF/CR equivalence. Serde tests roundtrip values and
check that root `Commented` values collect every comment in each constructible
single-document fixture. Other assertions check shell content, YAML styles, and
comments. Benchmark inputs stay fixed as the corpus changes.

## Syntax and construction cases

The manifest records syntax rejections separately from construction errors.
These inputs stay unmodified and remain covered by snapshots and assertions.
Default value construction rejects application tags and duplicate mapping keys.
The manifest records these failures in `serde_error`.

`sentry-compose.yaml` is a syntax rejection. The closing bracket in
`seaweedfs.healthcheck.test` is at the mapping key's indentation rather than
inside the flow continuation. YAML 1.2.2 requires greater indentation here. The
[rejection snapshot](../../crates/astral-yaml/tests/real_world/fixtures/sentry_compose.rs)
records the diagnostic, byte range, and source line.

Argo Workflows' `examples/resource-flags.yaml` has the same indentation error.
Google APIs' `google/cloud/functions/v2/cloudfunctions_v2.yaml` has a quoted
continuation at column one inside an indented mapping. Both have rejection
snapshots.

Argo CD's `applicationset/examples/template-override/default/guestbook-ui-svc.yaml`
contains a trailing tab. This is valid separation under §6.2 and is accepted.
