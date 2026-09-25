#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.14"
# dependencies = []
# ///
"""Verify pinned real-world fixture sizes and SHA-256 digests."""

import hashlib
import json
from pathlib import Path
from typing import NotRequired, TypedDict


class Fixture(TypedDict):
    file: str
    bytes: int
    sha256: str
    license: str
    notice: NotRequired[str]


def verify_fixture(root: Path, fixture: Fixture) -> None:
    data = (root / fixture["file"]).read_bytes()
    if (
        len(data) != fixture["bytes"]
        or hashlib.sha256(data).hexdigest() != fixture["sha256"]
    ):
        raise SystemExit(f"Fixture mismatch: {fixture['file']}")

    if not (root / fixture["license"]).is_file():
        raise SystemExit(f"Missing license: {fixture['file']}")

    if "notice" in fixture and not (root / fixture["notice"]).is_file():
        raise SystemExit(f"Missing notice: {fixture['file']}")


def main() -> None:
    root = Path(__file__).resolve().parent.parent / "fixtures/real-world"
    fixtures: list[Fixture] = json.loads(
        (root / "manifest.json").read_text(encoding="utf-8")
    )
    names = [fixture["file"] for fixture in fixtures]
    if len(names) != len(set(names)) or set(names) != {
        path.name for path in root.glob("*.yaml")
    }:
        raise SystemExit("Fixture manifest must list every YAML file exactly once")

    for fixture in fixtures:
        verify_fixture(root, fixture)

    print("All real-world fixture checksums match")


if __name__ == "__main__":
    main()
