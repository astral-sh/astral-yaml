#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.14"
# dependencies = []
# ///
"""Compare two throughput benchmark binaries, alternating their execution order."""

import argparse
import csv
import io
import os
import statistics
import subprocess
from pathlib import Path

FIXTURES = ("uv-ci", "kubernetes-guestbook", "openapi-petstore")
MODES = ("unspanned", "spanned", "spanned-fields", "serialize")

type Measurements = dict[tuple[int, str, str, str], float]


def measure(executable: Path, fixture: str, mode: str) -> float:
    env = dict(os.environ, ASTRAL_BENCH_FIXTURE=fixture, ASTRAL_BENCH_MODE=mode)
    env.pop("ASTRAL_BENCH_PROFILE", None)

    result = subprocess.run(
        [str(executable)],
        env=env,
        capture_output=True,
        text=True,
        check=True,
        timeout=60,
    )
    rows = list(csv.DictReader(io.StringIO(result.stdout)))
    if len(rows) != 1 or rows[0]["fixture"] != fixture or rows[0]["mode"] != mode:
        raise SystemExit(f"Unexpected benchmark output: {result.stdout}")

    return float(rows[0]["median_us"])


def collect_measurements(
    executables: dict[str, Path], runs: int, output_path: Path
) -> Measurements:
    measurements: Measurements = {}

    with output_path.open("w", encoding="utf-8", newline="") as output:
        writer = csv.writer(output)
        writer.writerow(["run", "revision", "fixture", "mode", "median_us"])

        for run in range(runs):
            for index, fixture in enumerate(FIXTURES):
                for offset, mode in enumerate(MODES):
                    order = list(executables)
                    if (run + index + offset) % 2:
                        order.reverse()

                    for revision in order:
                        elapsed = measure(executables[revision], fixture, mode)
                        measurements[run, revision, fixture, mode] = elapsed
                        writer.writerow([run + 1, revision, fixture, mode, elapsed])
                        output.flush()

                    print(f"Measured run {run + 1}: {fixture} {mode}", flush=True)

    return measurements


def report(measurements: Measurements, runs: int) -> bool:
    passed = True
    print("fixture,mode,baseline_us,candidate_us,time_reduction_percent")

    for fixture in FIXTURES:
        for mode in MODES:
            baseline = [
                measurements[run, "baseline", fixture, mode] for run in range(runs)
            ]
            candidate = [
                measurements[run, "candidate", fixture, mode] for run in range(runs)
            ]
            ratio = statistics.median(
                new / old for old, new in zip(baseline, candidate, strict=True)
            )
            print(
                f"{fixture},{mode},{statistics.median(baseline):.3f},"
                f"{statistics.median(candidate):.3f},{100 * (1 - ratio):.2f}"
            )
            passed &= ratio <= 0.8

    return passed


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline", type=Path)
    parser.add_argument("candidate", type=Path)
    parser.add_argument("--runs", type=int, default=3)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    if args.runs < 1:
        parser.error("--runs must be positive")

    executables = {
        "baseline": args.baseline.resolve(strict=True),
        "candidate": args.candidate.resolve(strict=True),
    }
    measurements = collect_measurements(executables, args.runs, args.output)
    if not report(measurements, args.runs):
        raise SystemExit("Some workloads have not reached a 20% time reduction")


if __name__ == "__main__":
    main()
