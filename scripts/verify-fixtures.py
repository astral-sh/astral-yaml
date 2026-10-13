#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.14"
# dependencies = []
#
# [tool.uv]
# require-build-hashes = true
# preview-features = ["build-dependency-hashes"]
# ///
"""Verify pinned real-world fixture sizes and SHA-256 digests."""

import hashlib
import json
import re
from pathlib import Path, PurePosixPath
from typing import NotRequired, TypedDict


class Fixture(TypedDict):
    file: str
    repository: str
    revision: str
    path: str
    category: str
    bytes: int
    sha256: str
    license: str
    notice: NotRequired[str]


def verify_fixture(root: Path, fixture: Fixture) -> None:
    if not re.fullmatch(r"[0-9a-f]{40}", fixture["revision"]):
        raise SystemExit(f"Fixture revision must be a commit: {fixture['file']}")

    if not re.fullmatch(r"[\w.-]+/[\w.-]+", fixture["repository"]):
        raise SystemExit(f"Invalid source repository: {fixture['file']}")

    path = PurePosixPath(fixture["path"])
    if path.is_absolute() or ".." in path.parts or not path.name:
        raise SystemExit(f"Invalid source path: {fixture['file']}")

    if not fixture["category"]:
        raise SystemExit(f"Missing category: {fixture['file']}")

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

    digests = {fixture["sha256"] for fixture in fixtures}
    if len(digests) != len(fixtures):
        raise SystemExit("Real-world fixtures must have distinct contents")

    if len(fixtures) < 500:
        raise SystemExit("The real-world corpus must contain at least 500 inputs")

    for fixture in fixtures:
        verify_fixture(root, fixture)

    repositories = {fixture["repository"] for fixture in fixtures}
    categories = {fixture["category"] for fixture in fixtures}
    print(
        f"Verified {len(fixtures)} distinct inputs from {len(repositories)} "
        f"repositories across {len(categories)} categories"
    )


if __name__ == "__main__":
    main()
