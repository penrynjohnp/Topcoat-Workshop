#!/usr/bin/env bash
# Usage: scripts/ship.sh <NN|slipway|docs> ["commit message"]
#
# Verifies, commits, pushes, and watches CI for one lab, the capstone, or the book.
set -euo pipefail

what="${1:?usage: ship.sh <NN|slipway|docs> [\"commit message\"]}"
msg="${2:-}"
repo_url="https://github.com/penrynjohnp/Topcoat-Workshop"

# --- Guard: never work against a stale or diverged tree -----------------------
# A stale checkout silently reintroduces old code and produces confusing build
# errors (e.g. pre-0.8 `signal name = value;` reported as a framework problem).
git fetch --quiet origin
branch=$(git rev-parse --abbrev-ref HEAD)
if git rev-parse --verify --quiet "origin/$branch" >/dev/null; then
  behind=$(git rev-list --count "HEAD..origin/$branch")
  ahead=$(git rev-list --count "origin/$branch..HEAD")
  if [ "$behind" -gt 0 ] && [ "$ahead" -gt 0 ]; then
    echo "✋ $branch has diverged from origin/$branch ($ahead ahead, $behind behind)."
    echo "   Sort that out before shipping: git log --oneline origin/$branch..HEAD"
    exit 1
  fi
  if [ "$behind" -gt 0 ]; then
    echo "Local $branch is $behind commit(s) behind origin. Fast-forwarding..."
    git merge --ff-only "origin/$branch" || {
      echo "✋ Could not fast-forward — you have local work on a stale base."
      echo "   Do NOT merge blindly; check: git diff origin/$branch --stat"
      exit 1
    }
    echo "⚠️  The tree moved underneath this run. Re-check the work you just did"
    echo "   against the updated files before trusting it."
  fi
fi

# --- Verify -------------------------------------------------------------------
case "$what" in
  slipway)
    cargo build  --manifest-path slipway/Cargo.toml
    cargo test   --manifest-path slipway/Cargo.toml
    cargo clippy --manifest-path slipway/Cargo.toml -- -D warnings
    msg="${msg:-slipway: update capstone}"
    url="$repo_url/tree/main/slipway"
    ;;
  docs)
    if command -v mdbook >/dev/null; then
      mdbook build docs
      if grep -rl '{{#include' docs/book/html >/dev/null 2>&1; then
        echo "✋ Unresolved {{#include}} in the built book — fix the paths first."
        rm -rf docs/book
        exit 1
      fi
      rm -rf docs/book
    else
      echo "note: mdbook not installed locally; relying on the Docs workflow"
    fi
    msg="${msg:-docs: update book}"
    url="https://penrynjohnp.github.io/Topcoat-Workshop/"
    ;;
  *)
    n=$(printf "%02d" "$((10#$what))")
    scripts/check-lab.sh "$n"
    msg="${msg:-lab-$n: build out}"
    url="https://penrynjohnp.github.io/Topcoat-Workshop/02-labs/lab-$n.html"
    ;;
esac

# --- Commit and push ----------------------------------------------------------
git add -A
git commit -m "$msg" || echo "nothing new to commit"
sha=$(git rev-parse HEAD)
git push

# --- Watch the workflows for THIS commit --------------------------------------
echo "Waiting for workflows on $sha..."
sleep 15
for wf in ci.yml docs.yml; do
  id=$(gh run list --workflow="$wf" --commit="$sha" -L 1 --json databaseId -q '.[0].databaseId' 2>/dev/null || true)
  if [ -n "$id" ]; then
    gh run watch --exit-status "$id"
  else
    echo "no $wf run for this commit"
  fi
done

echo "✅ $url"
