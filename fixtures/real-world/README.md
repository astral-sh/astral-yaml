# Real-world corpus

The corpus contains 632 distinct, unmodified files from 33 repositories across
23 categories. Inputs include Kubernetes resources, Compose applications,
Argo workflows, Istio and Envoy configurations, Prometheus and OpenTelemetry
configurations, Ansible tasks, OpenAPI documents, CloudFormation templates,
Windows package manifests, Spack configurations, Spring applications,
Home Assistant services, encrypted SOPS examples, and CI workflows.

`manifest.json` records each file's repository, revision, original path, size,
SHA-256, category, and license. Revisions are full commit IDs. License files and
upstream notices are included alongside the inputs. Selection covered application
and infrastructure configurations. Byte-identical copies and
templates that require rendering before they are YAML were excluded.

The inputs exercise expressions, multiline scripts, plain continuation lines,
literal and folded scalars, flow collections, multiple documents, empty values,
quotes, comments, anchors, aliases, and application tags.
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
agree with each accepted AST, apart from the recorded Python-parser rejection
below. The test generates expectations at runtime. It does not construct values
or resolve schema types,
so YAML 1.1 spellings such as `on` remain scalar text. The
[development checks](../../CONTRIBUTING.md#checks) include this comparison as well
as both crates' normal tests.

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
`Spanned` values collect every comment in each constructible single-document fixture.
Other assertions check shell content, YAML styles, and comments. Benchmarks use
the original three fixtures so results remain comparable.

## Expected rejections and parser differences

The manifest records syntax rejections separately from construction errors.
These inputs stay unmodified and remain covered by snapshots and assertions.
Of the 632 inputs, 629 have accepted syntax. Forty require application tag
handling and one has duplicate mapping keys; default value construction rejects
those 41 inputs with the recorded `serde_error`.

`sentry-compose.yaml` is a syntax rejection. The closing bracket in
`seaweedfs.healthcheck.test` is at the mapping key's indentation
(line 302), rather than inside the flow continuation. YAML 1.2.2 productions
63, 65, 79, 137, and 138 require greater indentation in this context. The
inline [rejection snapshot](../../crates/astral-yaml/tests/real_world/fixtures/sentry_compose.rs)
records the diagnostic, byte range, and source line. PyYAML accepts the file;
the comparison test checks this disagreement. The fixture is unmodified.

Argo Workflows' `examples/resource-flags.yaml` has the same indentation error.
Google APIs' `google/cloud/functions/v2/cloudfunctions_v2.yaml` has a quoted
continuation at column one inside an indented mapping. Both have rejection
snapshots; PyYAML accepts both.

Argo CD's `applicationset/examples/template-override/default/guestbook-ui-svc.yaml`
contains a trailing tab. This is valid separation under §6.2. LibYAML and Astral
accept it; PyYAML's Python parser rejects it. `python_error` pins that rejection,
and the syntax and span comparison still runs against LibYAML.

Three inputs use plain decimal integers with leading zeroes. YAML 1.2 core
resolves `0755` as the integer 755. `yaml_serde` leaves it as a string.
`leading_zero_integers` records the number of these differences; comparison
tests check their values and reject other differences.
