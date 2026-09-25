#!/usr/bin/env python3
"""Verify pinned real-world fixture sizes and SHA-256 digests."""
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parent.parent / "fixtures/real-world"
fixtures = json.loads((root / "manifest.json").read_text())
names = [fixture["file"] for fixture in fixtures]
if len(names) != len(set(names)) or set(names) != {path.name for path in root.glob("*.yaml")}:
    raise SystemExit("Fixture manifest must list every YAML file exactly once")

for fixture in fixtures:
    data = (root / fixture["file"]).read_bytes()
    if len(data) != fixture["bytes"] or hashlib.sha256(data).hexdigest() != fixture["sha256"]:
        raise SystemExit(f"Fixture mismatch: {fixture['file']}")
    if not (root / fixture["license"]).is_file():
        raise SystemExit(f"Missing license: {fixture['file']}")
    if "notice" in fixture and not (root / fixture["notice"]).is_file():
        raise SystemExit(f"Missing notice: {fixture['file']}")
    for suffix in [".event", ".scalars"]:
        if not (root / fixture["file"]).with_suffix(suffix).is_file():
            raise SystemExit(f"Missing syntax expectation: {fixture['file']}{suffix}")
    error = (root / fixture["file"]).with_suffix(".error")
    if error.exists() != (fixture.get("valid_yaml_1_2", True) is False):
        raise SystemExit(f"Rejection expectation disagrees with manifest: {fixture['file']}")

print("All real-world fixture checksums match")
