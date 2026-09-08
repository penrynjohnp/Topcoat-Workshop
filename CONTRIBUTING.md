# Contributing

- Follow `AGENTS.md` for conventions; it's written for humans too.
- New lab: `scripts/new-lab.sh NN slug`, then fill in the README template from `PLAN.md` §4.
- Every PR must keep `cargo build --workspace` and `cargo test --workspace` green.
- Fixing a drift issue? Add a row to `COMPATIBILITY.md`.
