#!/usr/bin/env python3
"""Verify pinned real-world fixture sizes and SHA-256 digests."""
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent.parent / "fixtures/real-world"
for fixture in json.loads((root / "manifest.json").read_text()):
    data = (root / fixture["file"]).read_bytes()
    if len(data) != fixture["bytes"] or hashlib.sha256(data).hexdigest() != fixture["sha256"]:
        raise SystemExit(f"Fixture mismatch: {fixture['file']}")
    if not (root / fixture["license"]).is_file():
        raise SystemExit(f"Missing license: {fixture['file']}")
print("All real-world fixture checksums match")
