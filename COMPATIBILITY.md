# Compatibility

Topcoat is pre-1.0 and ships breaking changes. Every lab pins the versions below; `drift.yml` rebuilds weekly against latest and opens an issue when something breaks.

| Tested | topcoat | topcoat-cli | toasty | Rust | Labs passing | Notes |
|---|---|---|---|---|---|---|
| 2026-09-08 | 0.7.0 | 0.7.0 | 0.10.0 | stable | 01 | Initial skeleton. Lab 01 code taken from upstream `getting_started.md` at this date. |

## Upgrading

1. Bump versions in the workspace `Cargo.toml` (`[workspace.dependencies]`) and `scripts/setup.sh`.
2. `cargo build --workspace && cargo test --workspace`.
3. Fix breakages lab by lab; each fix PR adds a row here.
