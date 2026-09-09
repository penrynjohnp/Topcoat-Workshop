#!/usr/bin/env bash
# Usage: scripts/check-lab.sh 01  — builds starter + solution, tests and lints the solution
set -euo pipefail
n=$(printf "%02d" "$((10#$1))")
dir=$(ls -d labs/lab-${n}-* | head -1)
echo "Checking $dir"
cargo build --manifest-path "$dir/starter/Cargo.toml"
cargo build --manifest-path "$dir/solution/Cargo.toml"
cargo test  --manifest-path "$dir/solution/Cargo.toml"
cargo clippy --manifest-path "$dir/solution/Cargo.toml" -- -D warnings
