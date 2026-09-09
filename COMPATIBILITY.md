# Compatibility

Topcoat is pre-1.0 and ships breaking changes. Every lab pins the versions below; `drift.yml` rebuilds weekly against latest and opens an issue when something breaks.

| Tested | topcoat | topcoat-cli | toasty | Rust | Labs passing | Notes |
|---|---|---|---|---|---|---|
| 2026-09-09 | 0.7.0 | 0.7.0 | 0.10.0 | stable (1.98.0) | 01-04 | Lab 01 code from `getting_started.md` at tag `v0.7.0`. Note: `main` already differs (bare `Result` vs `Result<impl View>`); always read docs at the tag. CLI 0.7.0: no top-level `--version`, and `topcoat fmt` has no `--check` (CI formats then `git diff --exit-code`). Attribute render order is not guaranteed once an element takes a spread `Attributes`, so lab tests assert on the attribute set. |

## Upgrading

1. Bump versions in the workspace `Cargo.toml` (`[workspace.dependencies]`) and `scripts/setup.sh`.
2. `cargo build --workspace && cargo test --workspace`.
3. Fix breakages lab by lab; each fix PR adds a row here.

## Docs toolchain

The mdBook toolchain is pinned as a set in `.github/workflows/docs.yml` and `.devcontainer/devcontainer.json`. Bump all three together.

| Tested | mdbook | mdbook-admonish | mdbook-mermaid | mdbook-linkcheck | Notes |
|---|---|---|---|---|---|
| 2026-09-09 | 0.4.52 | 1.20.0 | 0.15.0 | 0.7.7 | mdbook 0.5 changed the preprocessor JSON (null `[book]` fields, `items` replaces `sections`); admonish 1.20.0 cannot parse it. mermaid ≥ 0.16 requires mdbook 0.5. Move to 0.5 once admonish ships support; then unpin mermaid too. |
