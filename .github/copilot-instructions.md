# Agent instructions

This repo is a learning workshop for Topcoat, not an application. Read `PLAN.md` before making structural changes.

## Layout
- `labs/lab-NN-slug/README.md` follows the template in PLAN.md §4 exactly. `starter/` must always compile (TODOs marked `// TODO(lab-NN):`). `solution/` must have at least one integration test.
- `docs/` is an mdBook. Code blocks in docs are `{{#include}}`d from `labs/*/solution` — never paste code into docs by hand.
- `Cargo.toml` at the root is a workspace over every `starter/` and `solution/`. Use `[workspace.dependencies]` for `topcoat`, `topcoat-cli`, `toasty`, `tokio`; never pin a version inside a lab crate.
- Every lab has a book page `docs/src/02-labs/lab-NN.md` containing only the title and
  `{{#include ../../../labs/lab-NN-slug/README.md:3:}}`. Creating or renaming a lab
  updates this page in the same commit.

## Documentation pages
These rules apply to every page under `docs/src/` other than the lab pages above.
- The book is mdBook 0.5 with `mdbook-mermaid` only. Callouts use GitHub alert syntax — `> [!NOTE]`, `> [!TIP]`, `> [!WARNING]` — never `admonish` blocks.
- Code is never pasted. It comes from `labs/*/solution` or `slipway/` via `{{#include}}` with `// ANCHOR: name` / `// ANCHOR_END: name` markers. Add anchors to source files where needed, then re-run that crate's tests to prove the file still compiles.
- Include paths are relative to the page: from `docs/src/01-concepts/` the labs are at `../../../labs/...`.
- Diagrams are Mermaid fenced blocks (```` ```mermaid ````). No images.
- Every page ends with a **See also** line linking the relevant lab pages (`../02-labs/lab-NN.md`) and any related concept or reference page.
- Voice: second person, present tense, short paragraphs, one idea per sentence. No filler introductions.
- Every claim about how Topcoat behaves is checked against the v0.8.0 guides under `crates/*/docs/` in tokio-rs/topcoat (at the tag, never `main`). Where a page explains internals (`$(...)` cross-compilation, shards, `live!`), say which guide each section draws on.
- After writing or editing any page: run `mdbook build docs`, confirm the output has no ` ERROR ` lines and that `grep -rl '{{#include' docs/book/html` returns nothing, then `rm -rf docs/book`.

## Topcoat facts to respect
- Server-rendered. Never suggest browser storage, a client bundle, or a separate API layer for page data.
- Reactivity: `signal` + `$(...)` for client-only state; `#[shard]` for server re-render; `#[procedure]` for server calls; `live!`/`suspense` for streaming. Prefer these over htmx unless the lab is about htmx.
- Auth and request-scoped concerns are `async fn f(cx: &Cx)` functions, not middleware.
- Versions are pinned in `COMPATIBILITY.md`. The API surface changes between minor versions — check https://docs.rs/topcoat for the pinned version before writing code. When reading the guides under `crates/*/docs/` in tokio-rs/topcoat, use the release tag (e.g. `https://raw.githubusercontent.com/tokio-rs/topcoat/v0.8.0/...`), **never `main`** — `main` is ahead of the published crate and its examples will not compile against the pin.
- Assets: `topcoat asset bundle --release` writes `target/release/assets/`; binary and bundle must come from the same build.
- `topcoat-icon`'s build script stages the Iconify cache non-atomically; parallel builds can read an empty file. CI stages one crate first (`cargo build -p lab11-solution`); do the same locally after `cargo clean`.
- The CLI at 0.8.0 has no top-level `--version` and `topcoat fmt` has no `--check`; CI formats then `git diff --exit-code`.

## Conventions
- `cargo fmt` and `topcoat fmt` on every Rust file; `cargo clippy -D warnings` clean.
- Commit messages: `lab-NN: ...`, `docs: ...`, `slipway: ...`, `infra: ...`, `ci: ...`, `repo: ...`.
- Deployment is Bicep-only, managed identity only, no secrets in templates.
- `AGENTS.md` and `.github/copilot-instructions.md` must be identical; CI checks this. Edit `AGENTS.md`, then copy.
