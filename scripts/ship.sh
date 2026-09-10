#!/usr/bin/env bash
# Usage: scripts/ship.sh 06 "lab-06: cookies, sessions, mail"
set -euo pipefail
n=$(printf "%02d" "$((10#$1))"); msg="${2:-lab-$n: build out}"
scripts/check-lab.sh "$n"
git add -A
git commit -m "$msg" || echo "nothing new to commit"
git push
echo "Waiting for CI and Docs..."
sleep 10
gh run watch --exit-status "$(gh run list --workflow=ci.yml -L 1 --json databaseId -q '.[0].databaseId')"
gh run watch --exit-status "$(gh run list --workflow=docs.yml -L 1 --json databaseId -q '.[0].databaseId')"
echo "✅ https://penrynjohnp.github.io/Topcoat-Workshop/02-labs/lab-$n.html"