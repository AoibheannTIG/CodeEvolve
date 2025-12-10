#!/usr/bin/env python3
# ===--------------------------------------------------------------------------------------===#
#
# Part of the CodeEvolve Project, under the Apache License v2.0.
# See https://github.com/inter-co/science-codeevolve/blob/main/LICENSE for license information.
# SPDX-License-Identifier: Apache-2.0
#
# ===--------------------------------------------------------------------------------------===#
#
# This file tests the TIG Knapsack algorithm against challenge instances.
#
# ===--------------------------------------------------------------------------------------===#

from concurrent.futures import ThreadPoolExecutor
import subprocess
import tempfile
import argparse
import os

# Get the directory where this script is located
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
# c003-runner is one level up from input/
RUNNER_DIR = os.path.join(os.path.dirname(SCRIPT_DIR), "c003-runner")
RUNNER_PATH = os.path.join(RUNNER_DIR, "target/release/c003-runner")


def run_algorithm(seed):
    try:
        quality, time, memory = None, None, None
        with tempfile.NamedTemporaryFile() as solution_file:
            cmd = [
                "/usr/bin/time",
                "-f", "Memory: %M",
                RUNNER_PATH,
                "solve",
                args.track_id,
                str(seed),
                solution_file.name,
            ]
            if args.hyperparameters:
                cmd += ["--hyperparameters", args.hyperparameters]
            if args.debug:
                print("Running command:", " ".join(cmd))
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=args.timeout,
            )
            for line in result.stdout.strip().split("\n"):
                if line.startswith("Time:"):
                    time = float(line.split(":")[1].strip())
            for line in result.stderr.strip().split("\n"):
                if line.startswith("Memory:"):
                    memory = int(line.split(":")[1].strip())

            cmd = [
                RUNNER_PATH,
                "eval",
                args.track_id,
                str(seed),
                solution_file.name,
            ]
            if args.debug:
                print("Running command:", " ".join(cmd))
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=args.timeout,
            )
            if result.returncode != 0:
                raise ValueError(f"Evaluation failed: {result.stderr.strip()}")
            for line in result.stdout.strip().split("\n"):
                if line.startswith("Quality:"):
                    quality = line.split(":")[1].strip()
        return quality, time, memory
    except Exception as e:
        print(f"Seed: {seed}, Error: {e}")
        return None, None, None


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Test TIG Knapsack algorithm")
    parser.add_argument("track_id", type=str, help="Track ID (e.g., 'n_items=500,density=25')")
    parser.add_argument("--hyperparameters", type=str, default=None)
    parser.add_argument("--workers", type=int, default=1)
    parser.add_argument("--nonces", type=int, default=1)
    parser.add_argument("--timeout", type=int, default=60)
    parser.add_argument("--debug", action="store_true")
    args = parser.parse_args()

    pool = ThreadPoolExecutor(max_workers=args.workers)
    results = list(pool.map(run_algorithm, range(args.nonces)))
    qualities = [float(q) for q, t, m in results if q is not None]
    if qualities:
        print(f"Average quality: {sum(qualities) / len(qualities)}")
        print(f"Median quality: {sorted(qualities)[len(qualities) // 2]}")
        print(f"Max quality: {max(qualities)}")
        print(f"Min quality: {min(qualities)}")
    else:
        print("No valid results to calculate statistics")

