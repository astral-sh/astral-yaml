"""Exercise the Python bindings for a profile-guided wheel build."""

from io import BytesIO, StringIO
from pathlib import Path

import astral_yaml

fixtures = Path(__file__).resolve().parents[1] / "fixtures" / "real-world"
documents = [
    (fixtures / "openapi-petstore.yaml").read_text(encoding="utf-8"),
    (fixtures / "ruff-ci.yaml").read_text(encoding="utf-8"),
    """\
paths: &p ["/home", "/etc", "☃"]
mirror: *p
meta: {n: -143, ok: false, scale: 0.985, empty: null}
""",
]
for document in documents:
    encoded = document.encode()
    for _ in range(120):
        value = astral_yaml.loads(document)
        astral_yaml.load(BytesIO(encoded))
        astral_yaml.dumps(value)
        astral_yaml.dump(value, StringIO())
