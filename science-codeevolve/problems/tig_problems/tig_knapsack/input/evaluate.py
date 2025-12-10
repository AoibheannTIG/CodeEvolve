# ===--------------------------------------------------------------------------------------===#
#
# Part of the CodeEvolve Project, under the Apache License v2.0.
# See https://github.com/inter-co/science-codeevolve/blob/main/LICENSE for license information.
# SPDX-License-Identifier: Apache-2.0
#
# ===--------------------------------------------------------------------------------------===#
#
# This file implements the evaluator for the TIG Knapsack problem.
# It builds and evaluates Rust-based quadratic knapsack algorithms.
#
# ===--------------------------------------------------------------------------------------===#

import os
import subprocess
import math
import json
import sys
from typing import List, Dict
from argparse import Namespace

# Get the directory where this evaluator script is located
# Use absolute path to handle temp directory copies
EVALUATOR_DIR = "/root/science-codeevolve/problems/tig_problems/tig_knapsack/input"
# c003-runner is one level up from input/
RUNNER_DIR = "/root/science-codeevolve/problems/tig_problems/tig_knapsack/c003-runner"
RUNNER_PATH = os.path.join(RUNNER_DIR, "target/release/c003-runner")

NUM_SEEDS = 1000
TRACK_ID = "n_items=500,density=25"
TIMEOUT = 120


def build_algorithm() -> bool:
    """Build the Rust algorithm using cargo."""
    try:
        original_dir = os.getcwd()
        os.chdir(RUNNER_DIR)
        subprocess.run(["cargo", "build", "--release"], check=True, capture_output=True)
        os.chdir(original_dir)
        return True
    except subprocess.CalledProcessError as e:
        os.chdir(original_dir)
        return False


def run_algorithm(seed: int, args: Namespace):
    """Run the algorithm for a single seed and return quality, time, memory."""
    import tempfile
    
    try:
        quality, time_val, memory = None, None, None
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
            
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=args.timeout,
            )
            for line in result.stdout.strip().split("\n"):
                if line.startswith("Time:"):
                    time_val = float(line.split(":")[1].strip())
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
        return quality, time_val, memory
    except Exception as e:
        return None, None, None


def performance_scale(x: float, max_btb: float) -> float:
    """Smoothly scale performance based on better-than-baseline metric."""
    if max_btb <= 0:
        return 0.0

    numerator = math.exp(3000.0 * x) - 1.0
    denominator = math.exp(3000.0 * max_btb) - 1.0

    if denominator == 0.0:
        return 0.0

    return max(0.0, min(numerator / denominator, 1.0))


def calculate_scores(qualities: List[float], times: List[float], memories: List[float]) -> Dict[str, float]:
    """Calculate normalized scores from raw metrics."""
    # Performance score: normalize by the highest achievable btb (~0.0014)
    MAX_BTB = 0.001
    avg_btb = sum(qualities) / len(qualities)
    if avg_btb < 0:
        performance_score = (0.5 + float(avg_btb)) / 10.0
    else:
        performance_score = performance_scale(avg_btb, MAX_BTB)
    
    eval_time = sum(times) 
    # Logarithmic speed score
    if eval_time > 0:
        speed_score = max(0.0, min(1.0, 0.1 * math.log10(6000.0 / eval_time)))
    else:
        speed_score = 1.0
    memory = sum(memories)
    combined_score = performance_score
    
    return {
        "avg_btb": float(avg_btb),
        "combined_score": float(combined_score),
        "eval_time": float(eval_time),
        "memory": float(memory)
    }


def evaluate(program_path: str, results_path: str) -> None:
    """
    Evaluate a TIG Knapsack algorithm.
    
    Args:
        program_path: Path to the Rust source file containing the algorithm
        results_path: Path where JSON results should be written
    """
    try:
        abs_program_path = os.path.abspath(program_path)
        original_dir = os.getcwd()

        # Copy algorithm code to c003-runner/src/algorithm/mod.rs
        with open(abs_program_path, "r") as src:
            content = src.read()
        algo_path = os.path.join(RUNNER_DIR, "src/algorithm/mod.rs")
        with open(algo_path, "w") as f:
            f.write(content)

        # Build the algorithm
        if not build_algorithm():
            with open(results_path, "w") as f:
                json.dump({
                    "avg_btb": 0.0,
                    "combined_score": 0.0,
                    "eval_time": 0.0,
                    "memory": 0.0,
                    "error": "Build failed"
                }, f, indent=4)
            return

        # Set up args for running
        args = Namespace(
            track_id=TRACK_ID,
            hyperparameters=None,
            workers=1,
            nonces=NUM_SEEDS,
            timeout=TIMEOUT,
            debug=False,
        )
        
        # Run algorithm for each seed and collect results
        qualities = []
        times = []
        memories = []
        
        for seed in range(NUM_SEEDS):
            quality, time_seconds, memory = run_algorithm(seed, args)
            if quality is not None and time_seconds is not None and memory is not None:
                qualities.append(float(quality) / 1000000.0)
                times.append(float(time_seconds))
                memories.append(float(memory))

        os.chdir(original_dir)

        if not qualities:
            with open(results_path, "w") as f:
                json.dump({
                    "avg_btb": 0.0,
                    "combined_score": 0.0,
                    "eval_time": 0.0,
                    "memory": 0.0,
                    "error": "No successful evaluations"
                }, f, indent=4)
            return

        scores = calculate_scores(qualities, times, memories)
        
        with open(results_path, "w") as f:
            json.dump({
                "avg_btb": scores["avg_btb"],
                "combined_score": scores["combined_score"],
                "eval_time": scores["eval_time"],
                "memory": scores["memory"]
            }, f, indent=4)
            
    except Exception as e:
        with open(results_path, "w") as f:
            json.dump({
                "avg_btb": 0.0,
                "combined_score": 0.0,
                "eval_time": 0.0,
                "memory": 0.0,
                "error": str(e)
            }, f, indent=4)


if __name__ == "__main__":
    program_path = sys.argv[1]
    results_path = sys.argv[2]
    
    evaluate(program_path, results_path)

