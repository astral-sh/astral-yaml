# Real-world fixtures

The YAML files are unmodified snapshots from public projects. `manifest.json`
records each upstream path, exact commit, SHA-256 digest, byte size, and license.
Licenses are included alongside the fixtures.

- `uv-ci.yaml`: the uv project's CI workflow.
- `kubernetes-guestbook.yaml`: a Kubernetes Guestbook deployment stream.
- `openapi-petstore.yaml`: the OpenAPI project's expanded Petstore API description.

Run `python3 scripts/verify-fixtures.py` to check the snapshots. Workspace tests
compare the decoded values against `yaml_serde` and round-trip them through the
astral serializer.
