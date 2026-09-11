#!/usr/bin/env bash
# Usage: scripts/ship.sh <NN|slipway|docs> ["commit message"]
set -euo pipefail
what="$1"; msg="${2:-}"
case "$what" in
  slipway)
    cargo build  --manifest-path slipway/Cargo.toml
    cargo test   --manifest-path slipway/Cargo.toml
    cargo clippy --manifest-path slipway/Cargo.toml -- -D warnings
    msg="${msg:-slipway: update capstone}"; url="https://github.com/penrynjohnp/Topcoat-Workshop/tree/main/slipway" ;;
  docs)
    command -v mdbook >/dev/null && mdbook build docs && ! grep -rl '{{#include' docs/book/html && rm -rf docs/book
    msg="${msg:-docs: update book}"; url="https://penrynjohnp.github.io/Topcoat-Workshop/" ;;
  *)
    n=$(printf "%02d" "$((10#$what))"); scripts/check-lab.sh "$n"
    msg="${msg:-lab-$n: build out}"; url="https://penrynjohnp.github.io/Topcoat-Workshop/02-labs/lab-$n.html" ;;
esac
git add -A
git commit -m "$msg" || echo "nothing new to commit"
sha=$(git rev-parse HEAD)
git push
echo "Waiting for workflows on $sha..."
sleep 15
for wf in ci.yml docs.yml; do
  id=$(gh run list --workflow=$wf --commit="$sha" -L 1 --json databaseId -q '.[0].databaseId')
  [ -n "$id" ] && gh run watch --exit-status "$id" || echo "no $wf run for this commit"
done
echo "✅ $url"
