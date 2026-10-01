# Contributing

- Follow `AGENTS.md` for conventions; it's written for humans too.
- New lab: `scripts/new-lab.sh NN slug`, then fill in the README template from `PLAN.md` §4.
- Every PR must keep `cargo build --workspace` and `cargo test --workspace` green.
- Fixing a drift issue? Add a row to `COMPATIBILITY.md`.
- The current target is Topcoat and CLI 0.9.0, Toasty 0.10.0 and Rust 1.98.0. Use release-tagged guides, not `main`.
- Upgrade all workspace/CLI installation pins together, including both release Dockerfiles. Keep starters compilable without completing their TODOs.
- Test the touched slice before propagating it. Runtime changes need actual endpoint and transport checks, not only serialized-markup assertions.
- Refresh owned UI deliberately with `topcoat ui add --overwrite`; preserve custom source and theme tokens.
- After book edits, run `mdbook build docs`, check for unresolved includes, and remove generated output. Add a compatibility row only for checks actually run.
