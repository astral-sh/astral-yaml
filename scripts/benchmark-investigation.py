# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///

import json
import re
import shutil
import statistics
import subprocess
import tempfile
from pathlib import Path


def medians(output: str) -> dict[str, float]:
    result: dict[str, float] = {}
    group = ""

    for line in output.splitlines():
        match = re.search(r"[├╰]─ (\S+)", line)
        if match is None:
            continue

        name = match[1]
        if name.startswith(("parse", "deserialize", "serialize")):
            group = name

        columns = line.rsplit("│", 5)
        if len(columns) != 6 or not columns[-1].strip().isdigit():
            continue

        value, unit = columns[2].split()
        key = group if name == group else f"{group}/{name}"
        result[key] = float(value) * {"ns": 0.001, "µs": 1, "ms": 1000}[unit]

    assert len(result) == 16, result
    return result


def run(command: list[str], directory: Path) -> str:
    print(f"Running {command} in {directory}", flush=True)
    result = subprocess.run(
        command,
        cwd=directory,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
        timeout=600,
    )
    return result.stdout


def build_and_profile(label: str, directory: Path, output: Path) -> Path:
    build = run(
        [
            "cargo",
            "bench",
            "-p",
            "astral-yaml-dev",
            "--bench",
            "yaml",
            "--locked",
            "--no-run",
            "--message-format=json",
        ],
        directory,
    )
    for line in build.splitlines():
        artifact = json.loads(line)
        if artifact.get("executable") and artifact["target"]["name"] == "yaml":
            binary = output / label
            shutil.copy2(artifact["executable"], binary)

    run(
        [
            "uv",
            "run",
            "--only-dev",
            "--locked",
            "cargo",
            "codspeed",
            "build",
            "-p",
            "astral-yaml-dev",
            "--bench",
            "yaml",
            "--locked",
            "-m",
            "simulation",
        ],
        directory,
    )
    profile = output / f"{label}-profile"
    profile.mkdir(exist_ok=True)
    run(
        [
            "codspeed",
            "run",
            "--skip-upload",
            "--mode",
            "simulation",
            "--profile-folder",
            str(profile),
            "--setup-cache-dir",
            str(Path.home() / ".cache/codspeed-action"),
            "--",
            "uv",
            "run",
            "--only-dev",
            "--locked",
            "cargo",
            "codspeed",
            "run",
            "-p",
            "astral-yaml-dev",
            "--bench",
            "yaml",
            "-m",
            "simulation",
        ],
        directory,
    )
    return output / label


def main() -> None:
    root = Path.cwd()
    output = root / "target/investigation"
    output.mkdir(parents=True, exist_ok=True)
    (output / "cpu.txt").write_text(run(["lscpu"], root))
    (output / "rust.txt").write_text(run(["rustc", "-vV"], root))
    baseline = Path(tempfile.mkdtemp(prefix="yaml-baseline-"))
    run(["git", "worktree", "add", "--detach", str(baseline), "0dbb599"], root)
    binaries: dict[str, Path] = {}

    parser = root / "crates/astral-yaml/src/parser.rs"
    original = parser.read_text()
    try:
        for label, directory in (("candidate", root), ("baseline", baseline)):
            binaries[label] = build_and_profile(label, directory, output)
        for patch in sorted((root / "scripts/benchmark-variants").glob("*.patch")):
            parser.write_text(original)
            run(["git", "apply", str(patch)], root)
            (output / f"{patch.stem}.rs").write_text(parser.read_text())
            binaries[patch.stem] = build_and_profile(patch.stem, root, output)
    finally:
        parser.write_text(original)

    results: dict[str, list[dict[str, float]]] = {name: [] for name in binaries}
    for repetition in range(2):
        pair = list(binaries)
        if repetition % 2:
            pair.reverse()

        for label in pair:
            measured = run(
                [
                    str(binaries[label]),
                    "--bench",
                    "--min-time",
                    "0.25",
                    "--max-time",
                    "0.3",
                    "--sample-size",
                    "64",
                    "--color",
                    "never",
                ],
                root,
            )
            (output / f"{label}-{repetition + 1}.txt").write_text(measured)
            results[label].append(medians(measured))

    (output / "medians.json").write_text(json.dumps(results, indent=2) + "\n")
    for label in binaries:
        if label == "baseline":
            continue

        reductions = []
        print(f"\nVariant: {label}")
        for name in results["baseline"][0]:
            old = statistics.median(item[name] for item in results["baseline"])
            new = statistics.median(item[name] for item in results[label])
            reduction = 100 * (1 - new / old)
            reductions.append(reduction)
            print(f"{name}: {old:.3f} -> {new:.3f} us; {reduction:.1f}% less time")

        print(f"Mean time reduction: {statistics.mean(reductions):.2f}%")
    for binary in binaries.values():
        binary.unlink()


if __name__ == "__main__":
    main()
