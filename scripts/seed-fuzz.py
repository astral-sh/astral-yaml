#!/usr/bin/env python3
"""Seed the parser fuzzer with the pinned official corpus and regression cases."""
from hashlib import sha256
from pathlib import Path

root = Path(__file__).resolve().parent.parent
corpora = [root / "fuzz/corpus" / name for name in ["parse", "serde"]]
for corpus in corpora:
    corpus.mkdir(parents=True, exist_ok=True)
inputs = list((root / "fixtures/yaml-test-suite").rglob("in.yaml"))
inputs += list((root / "fixtures/regressions").glob("*.yaml"))
inputs += list((root / "fixtures/real-world").glob("*.yaml"))
for path in inputs:
    data = path.read_bytes()
    for corpus in corpora:
        (corpus / sha256(data).hexdigest()).write_bytes(data)
print(f"Seeded {len(inputs)} inputs in each of {len(corpora)} corpora")

decoded = root / "fuzz/corpus/decode"
decoded.mkdir(parents=True, exist_ok=True)
for path in inputs:
    text = path.read_bytes().decode("utf-8")
    for encoding in ["utf-8", "utf-16-be", "utf-16-le", "utf-32-be", "utf-32-le"]:
        data = ("\ufeff" + text).encode(encoding)
        (decoded / sha256(data).hexdigest()).write_bytes(data)
print("Seeded all five input encodings")
