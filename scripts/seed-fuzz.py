#!/usr/bin/env python3
"""Seed the parser fuzzer with the pinned official corpus and regression cases."""
from hashlib import sha256
from pathlib import Path

root = Path(__file__).resolve().parent.parent
corpus = root / "fuzz/corpus/parse"
corpus.mkdir(parents=True, exist_ok=True)
inputs = list((root / "fixtures/yaml-test-suite").rglob("in.yaml"))
inputs += list((root / "fixtures/regressions").glob("*.yaml"))
for path in inputs:
    data = path.read_bytes()
    (corpus / sha256(data).hexdigest()).write_bytes(data)
print(f"Seeded {len(inputs)} inputs in {corpus}")
