#!/usr/bin/env python3
"""Report source coverage using each fuzz target's matching binary and profile."""
import json
import os
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parent.parent


def rustc(*args):
    return subprocess.check_output(
        ["rustup", "run", "nightly", "rustc", *args], text=True, timeout=10
    ).strip()


host = next(
    line.removeprefix("host: ")
    for line in rustc("-vV").splitlines()
    if line.startswith("host: ")
)
llvm = Path(rustc("--print", "sysroot")) / "lib/rustlib" / host / "bin"
extension = ".exe" if os.name == "nt" else ""
sources = sorted((root / "crates/astral-yaml/src").glob("*.rs"))
sources += sorted((root / "crates/astral-yaml-serde/src").glob("*.rs"))
reports = {}
for name in ["parse", "serde", "decode", "typed"]:
    profile = root / "fuzz/coverage" / name / "coverage.profdata"
    binary = (
        root / "target" / host / "coverage" / host / "release" / (name + extension)
    )
    for path in [profile, binary]:
        if not path.is_file():
            raise SystemExit(
                f"Missing {path}; run cargo xtask fuzz coverage for all four targets first"
            )
    arguments = [str(binary), f"-instr-profile={profile}", *map(str, sources)]
    print(f"Fuzz target: {name}", flush=True)
    for command in [["report"], ["export", "-summary-only"]]:
        result = subprocess.run(
            [str(llvm / ("llvm-cov" + extension)), *command, *arguments],
            capture_output=True, text=True, check=True, timeout=60,
        )
        if result.stderr:
            raise SystemExit(result.stderr)
        if command[0] == "report":
            print(result.stdout)
        else:
            reports[name] = json.loads(result.stdout)
(root / "fuzz/coverage/summary.json").write_text(
    json.dumps(reports, indent=2) + "\n"
)
