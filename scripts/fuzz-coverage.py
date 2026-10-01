#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.14"
# dependencies = []
# ///
"""Report source coverage using each fuzz target's matching binary and profile."""

import json
import os
import subprocess
from pathlib import Path

type JsonValue = (
    None | bool | int | float | str | list[JsonValue] | dict[str, JsonValue]
)


def rustc(*args: str) -> str:
    return subprocess.check_output(
        ["rustup", "run", "nightly", "rustc", *args], text=True, timeout=10
    ).strip()


def run_llvm_cov(executable: Path, arguments: list[str]) -> str:
    result = subprocess.run(
        [str(executable), *arguments],
        capture_output=True,
        text=True,
        check=True,
        timeout=60,
    )
    if result.stderr:
        raise SystemExit(result.stderr)

    return result.stdout


def main() -> None:
    root = Path(__file__).resolve().parent.parent
    host = next(
        line.removeprefix("host: ")
        for line in rustc("-vV").splitlines()
        if line.startswith("host: ")
    )
    llvm = Path(rustc("--print", "sysroot")) / "lib/rustlib" / host / "bin"
    extension = ".exe" if os.name == "nt" else ""
    llvm_cov = llvm / ("llvm-cov" + extension)
    sources = sorted((root / "crates/astral-yaml/src").glob("*.rs"))
    sources += sorted((root / "crates/astral-yaml-serde/src").glob("*.rs"))
    reports: dict[str, JsonValue] = {}

    for name in ("parse", "serde", "decode", "typed", "editable"):
        profile = root / "fuzz/coverage" / name / "coverage.profdata"
        binary = (
            root / "target" / host / "coverage" / host / "release" / (name + extension)
        )
        for path in (profile, binary):
            if not path.is_file():
                raise SystemExit(
                    f"Missing {path}; run cargo xtask fuzz coverage for all four targets first"
                )

        arguments = [str(binary), f"-instr-profile={profile}", *map(str, sources)]
        print(f"Fuzz target: {name}", flush=True)
        print(run_llvm_cov(llvm_cov, ["report", *arguments]))
        summary = run_llvm_cov(llvm_cov, ["export", "-summary-only", *arguments])
        reports[name] = json.loads(summary)

    (root / "fuzz/coverage/summary.json").write_text(
        json.dumps(reports, indent=2) + "\n", encoding="utf-8"
    )


if __name__ == "__main__":
    main()
