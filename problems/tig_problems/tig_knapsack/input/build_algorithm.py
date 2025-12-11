#!/usr/bin/env python3
# ===--------------------------------------------------------------------------------------===#
#
# Part of the CodeEvolve Project, under the Apache License v2.0.
# See https://github.com/inter-co/science-codeevolve/blob/main/LICENSE for license information.
# SPDX-License-Identifier: Apache-2.0
#
# ===--------------------------------------------------------------------------------------===#
#
# This file builds the TIG Knapsack Rust algorithm using cargo.
#
# ===--------------------------------------------------------------------------------------===#

import os
import subprocess

# Get the directory where this script is located
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
# c003-runner is one level up from input/
RUNNER_DIR = os.path.join(os.path.dirname(SCRIPT_DIR), "c003-runner")


def main():
    original_dir = os.getcwd()
    os.chdir(RUNNER_DIR)
    subprocess.run(["cargo", "build", "--release"], check=True)
    os.chdir(original_dir)


if __name__ == "__main__":
    main()

