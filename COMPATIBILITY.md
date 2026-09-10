# Compatibility

Topcoat is pre-1.0 and ships breaking changes. Every lab pins the versions below; `drift.yml` rebuilds weekly against latest and opens an issue when something breaks.

| Tested | topcoat | topcoat-cli | toasty | Rust | Labs passing | Notes |
|---|---|---|---|---|---|---|
| 2026-09-10 | 0.7.0 | 0.7.0 | 0.10.0 | stable (1.98.0) | 01-09 | Lab 01 code from `getting_started.md` at tag `v0.7.0`. Note: `main` already differs (bare `Result` vs `Result<impl View>`); always read docs at the tag. CLI 0.7.0: no top-level `--version`, and `topcoat fmt` has no `--check` (CI formats then `git diff --exit-code`). Attribute render order is not guaranteed once an element takes a spread `Attributes`, so lab tests assert on the attribute set. Runtime expressions have no `&&`/`||` and no `match`; Lab 07 spells combinations with `if`/`else`. The runtime script is an asset, so Lab 07's router takes an `Option<AssetBundle>` and tests pass `None`. Lab 08 explicitly calls `discover_shards()` and `discover_procedures()`; shard arguments are JSON tuples (for one `String`, `["Lady"]`), and streamed swaps arrive in the original response as template/script envelopes. `topcoat::htmx` sits behind the non-default `htmx` feature, so Lab 09 enables it per-crate (`topcoat = { workspace = true, features = ["htmx"] }`) rather than in `[workspace.dependencies]`; this pulls in `topcoat-htmx` 0.7.0. htmx itself is pinned to `htmx.org@2.0.10` from jsDelivr in the Lab 09 layout (Lab 11 vendors it with `asset!`). `view!` rejects bare boolean attributes, so the script tag needs `defer="defer"`. htmx response responders implement `IntoResponseParts` and must precede the body in the response tuple; a `view!` fragment becomes a body with `.single().await?`. |

## Upgrading

1. Bump versions in the workspace `Cargo.toml` (`[workspace.dependencies]`) and `scripts/setup.sh`.
2. `cargo build --workspace && cargo test --workspace`.
3. Fix breakages lab by lab; each fix PR adds a row here.

## Docs toolchain

The mdBook toolchain is pinned as a set in `.github/workflows/docs.yml` and `.devcontainer/devcontainer.json`. Bump them together.

| Tested | mdbook | mdbook-mermaid | mdbook-linkcheck2 | Notes |
|---|---|---|---|---|
| 2026-09-09 | 0.5.4 | 0.17.1 | 0.13.0 | Callouts are mdBook 0.5's built-in GitHub-style alerts (`> [!NOTE]`), so `mdbook-admonish` is no longer a dependency — it was the only thing holding the book on 0.4. `mdbook-linkcheck` 0.7.7 version-checks for mdbook 0.4.x and refuses to run under 0.5, so we use the maintained `mdbook-linkcheck2` fork; its config table is `[output.linkcheck2]` and it has no `taiki-e/install-action` manifest, so CI `cargo install`s it. HTML output stays at `docs/book/html` because linkcheck2 is a second output backend. |
| 2026-09-09 | 0.4.52 | 0.15.0 | — (linkcheck 0.7.7) | Superseded. mdbook 0.5 changed the preprocessor JSON (null `[book]` fields, `items` replaces `sections`); admonish 1.20.0 could not parse it, and mermaid ≥ 0.16 requires mdbook 0.5, so the whole set was held back. |
