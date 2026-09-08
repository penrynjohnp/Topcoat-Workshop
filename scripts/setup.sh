#!/usr/bin/env bash
# Install and verify the toolchain pinned in COMPATIBILITY.md
set -euo pipefail
TOPCOAT_CLI_VERSION="0.7.0"
command -v rustup >/dev/null || { echo "rustup not found — install from https://rustup.rs then re-run"; exit 1; }
rustup show active-toolchain
if ! command -v topcoat >/dev/null || ! topcoat --version | grep -q "$TOPCOAT_CLI_VERSION"; then
  cargo install topcoat-cli --version "$TOPCOAT_CLI_VERSION" --locked
fi
command -v sqlite3 >/dev/null || echo "note: sqlite3 CLI not found (needed from Lab 10)"
echo; echo "rustc:   $(rustc --version)"; echo "topcoat: $(topcoat --version)"
echo "Building lab 01 to verify..."
cargo build -p lab01-solution
echo "✅ ready — start with labs/lab-01-hello-topcoat/README.md"
