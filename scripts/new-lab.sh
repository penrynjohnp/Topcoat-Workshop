#!/usr/bin/env bash
# Usage: scripts/new-lab.sh 02 view-macro  — scaffolds labs/lab-02-view-macro from lab-01 and adds it to the workspace
set -euo pipefail
n=$(printf "%02d" "$((10#$1))"); slug=$2; dst="labs/lab-${n}-${slug}"
[ -e "$dst" ] && { echo "$dst exists"; exit 1; }
cp -r labs/lab-01-hello-topcoat "$dst"
sed -i "s/lab01-/lab${n}-/" "$dst"/*/Cargo.toml
sed -i "s/^# Lab 01.*/# Lab ${n} — TITLE/" "$dst/README.md"
sed -i "s|^\]|    \"$dst/starter\",\n    \"$dst/solution\",\n]|" Cargo.toml
echo "scaffolded $dst and added to workspace"
