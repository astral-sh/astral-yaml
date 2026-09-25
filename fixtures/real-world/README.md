# Real-world corpus

These are unmodified snapshots of eight GitHub Actions workflows, a Kubernetes
guestbook, an OpenAPI document, and Airflow and Sentry Compose configurations.
`manifest.json` records the repository, revision, original path, size, SHA-256, and retained
license for each file. Workflows exercise expressions, multiline shell scripts,
plain continuation lines, literal and folded scalars, flow matrices, empty
values, quotes, comments, and reusable jobs.
Airflow's configuration also exercises anchors, aliases, and merge-key syntax
(preserved as ordinary mappings; merge expansion is outside YAML 1.2 core).

The `.event` files describe the complete ordered syntax trees, including scalar
values and styles, collection styles, tags, anchors, and aliases. The `.scalars`
files contain UTF-8 byte ranges for each nonempty scalar token in source order.
Both are generated independently with PyYAML 6.0.3. Its Python and LibYAML-backed
parsers must agree before an expectation can be written. Schema resolution is
disabled, so YAML 1.1 spellings such as `on` remain scalar text.

```sh
uv run scripts/real-world-oracle.py          # verify committed expectations
uv run scripts/real-world-oracle.py --write  # regenerate after a reviewed update
python3 scripts/verify-fixtures.py
cargo test -p xtask --test real_world
```

Tests compare accepted ASTs and scalar spans against these expectations, check
LF/CRLF/CR equivalence, and compare constructed values with `yaml_serde`. Both
readers also consume our serializer's output. Handwritten assertions check shell
content, YAML styles, and comments. Root `Spanned` values must collect every
comment in each single-document fixture. Benchmarks retain the original three
fixtures for comparable performance measurements.

`sentry-compose.yaml` is deliberately retained as a **rejection** case. Its
`seaweedfs.healthcheck.test` closing bracket is at the mapping key's indentation
(line 302), rather than inside the flow continuation. YAML 1.2.2 productions
63, 65, 79, 137, and 138 require greater indentation in this context. The
`.error` file pins the diagnostic and byte range. PyYAML accepts the file;
its snapshots document that disagreement, not an expected AST for our parser.
The source remains unmodified, including its anchors and aliases.
