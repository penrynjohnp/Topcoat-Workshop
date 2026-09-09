# Agent instructions

This repo is a learning workshop for Topcoat, not an application. Read `PLAN.md` before making structural changes.

## Layout
- `labs/lab-NN-slug/README.md` follows the template in PLAN.md §4 exactly. `starter/` must always compile (TODOs marked `// TODO(lab-NN):`). `solution/` must have at least one integration test.
- `docs/` is an mdBook. Code blocks in docs are `{{#include}}`d from `labs/*/solution` — never paste code into docs by hand.
- `Cargo.toml` at the root is a workspace over every `starter/` and `solution/`. Use `[workspace.dependencies]` for `topcoat`, `topcoat-cli`, `toasty`, `tokio`; never pin a version inside a lab crate.
- Every lab has a book page `docs/src/02-labs/lab-NN.md` containing only the title and
  `{{#include ../../../labs/lab-NN-slug/README.md:3:}}`. Creating or renaming a lab
  updates this page in the same commit.

## Topcoat facts to respect
- Server-rendered. Never suggest browser storage, a client bundle, or a separate API layer for page data.
- Reactivity: `signal` + `$(...)` for client-only state; `#[shard]` for server re-render; `#[procedure]` for server calls; `live!`/`suspense` for streaming. Prefer these over htmx unless the lab is about htmx.
- Auth and request-scoped concerns are `async fn f(cx: &Cx)` functions, not middleware.
- Versions are pinned in `COMPATIBILITY.md`. The API surface changes between minor versions — check https://docs.rs/topcoat for the pinned version before writing code. When reading the guides under `crates/*/docs/` in tokio-rs/topcoat, use the release tag (e.g. `https://raw.githubusercontent.com/tokio-rs/topcoat/v0.7.0/...`), **never `main`** — `main` is ahead of the published crate and its examples will not compile against the pin.
- Assets: `topcoat asset bundle --release` writes `target/release/assets/`; binary and bundle must come from the same build.

## Conventions
- `cargo fmt` and `topcoat fmt` on every Rust file; `cargo clippy -D warnings` clean.
- Commit messages: `lab-NN: ...`, `docs: ...`, `infra: ...`, `ci: ...`.
- Deployment is Bicep-only, managed identity only, no secrets in templates.
