#!/usr/bin/env python3
"""Compare preserved throughput executables with alternating measurement order."""
import argparse
import csv
import io
import os
from pathlib import Path
import statistics
import subprocess

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
fixtures = ["uv-ci", "kubernetes-guestbook", "openapi-petstore"]
modes = ["unspanned", "spanned", "spanned-fields", "serialize"]
measurements = {}

with args.output.open("w") as output:
    writer = csv.writer(output)
    writer.writerow(["run", "revision", "fixture", "mode", "median_us"])

    for run in range(args.runs):
        for index, fixture in enumerate(fixtures):
            for offset, mode in enumerate(modes):
                order = list(executables)
                if (run + index + offset) % 2:
                    order.reverse()

                env = dict(os.environ, ASTRAL_BENCH_FIXTURE=fixture, ASTRAL_BENCH_MODE=mode)
                env.pop("ASTRAL_BENCH_PROFILE", None)

                for revision in order:
                    result = subprocess.run(
                        [str(executables[revision])], env=env, capture_output=True,
                        text=True, check=True, timeout=60,
                    )
                    rows = list(csv.DictReader(io.StringIO(result.stdout)))
                    if len(rows) != 1 or rows[0]["fixture"] != fixture or rows[0]["mode"] != mode:
                        raise SystemExit(f"Unexpected benchmark output: {result.stdout}")

                    elapsed = float(rows[0]["median_us"])
                    measurements[run, revision, fixture, mode] = elapsed
                    writer.writerow([run + 1, revision, fixture, mode, elapsed])
                    output.flush()

                print(f"Measured run {run + 1}: {fixture} {mode}", flush=True)

passed = True
print("fixture,mode,baseline_us,candidate_us,time_reduction_percent")
for fixture in fixtures:
    for mode in modes:
        baseline = [measurements[run, "baseline", fixture, mode] for run in range(args.runs)]
        candidate = [measurements[run, "candidate", fixture, mode] for run in range(args.runs)]
        ratio = statistics.median(new / old for old, new in zip(baseline, candidate))
        print(
            f"{fixture},{mode},{statistics.median(baseline):.3f},"
            f"{statistics.median(candidate):.3f},{100 * (1 - ratio):.2f}"
        )
        passed &= ratio <= 0.8

raise SystemExit(0 if passed else "Some workloads have not reached a 20% time reduction")
