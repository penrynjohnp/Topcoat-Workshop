# Lab 11 — Asset pipeline, fonts, icons, Tailwind, and Topcoat UI

**Time:** 75 min · **Module:** 5 · **Prerequisites:** Lab 10 complete (or copy `labs/lab-10-toasty-sqlite/solution`)

## What you'll learn
- Bundle local and remote files behind content-hashed `asset!` URLs
- Generate self-hosted fonts, compile-checked icons, and Tailwind CSS without Node
- Vendor Topcoat UI components into your source tree and restyle every use from one owned file

## Concepts (read first, 5 min)
Read [Assets, styling, and owned UI](https://github.com/penrynjohnp/Topcoat-Workshop/blob/main/docs/src/01-concepts/assets-styling-ui.md) and the pinned [Topcoat 0.7.0 asset guide](https://docs.rs/topcoat/0.7.0/topcoat/asset/index.html). An `asset!` handle is not file contents. It embeds a declaration in the compiled binary. The bundler scans that exact binary, copies or downloads the declared files, gives each output a content-derived filename, and writes a manifest that resolves handles to URLs. The binary and bundle are therefore one release unit.

Tailwind, Fontsource, and Iconify join that same pipeline. A Cargo build script runs Topcoat's standalone Tailwind integration and stages icon metadata; no Node, PostCSS, Vite, or client application appears. Topcoat UI follows a different ownership model from a component dependency: the CLI copies ordinary Rust source into this crate. You can inspect and change it, and Tailwind scans those local class strings with the rest of the app.

## Steps
### Step 1 — Enable build-time integrations
Enable `font-fontsource`, `icon-iconify`, `tailwind`, and `ui` alongside the inherited `htmx` feature. Tailwind and Iconify also need Topcoat in `[build-dependencies]`. The build script stages Lucide and renders CSS from the UI theme:

```rust
{{#include solution/build.rs:asset-build-script}}
```

Topcoat 0.7.0 downloads its pinned standalone Tailwind CLI on the first build and caches it under `target/topcoat/cache`. This is a Rust/Cargo build step, not an npm step.

> [!NOTE]
> **Checkpoint:** `cargo check -p lab11-solution` succeeds, and no `package.json` or `node_modules` directory exists.

### Step 2 — Declare local and remote assets
The local marina mark and the inherited htmx script both become assets. The remote declaration pins htmx 2.0.10 and verifies its SHA-256 digest before bundling:

```rust
{{#include solution/src/assets.rs:asset-declarations}}
```

A URL's hash protects the download; the hash in the output filename protects browser caching. They solve different problems. `rename` gives the output a readable stem, but the content hash still makes every changed file a new immutable URL.

> [!NOTE]
> **Checkpoint:** `topcoat asset bundle -p lab11-solution` writes `slipway-mark-<16 hex>.svg`, `htmx-<16 hex>.js`, generated CSS, font files, and `manifest.toml` under `target/debug/assets/`.

### Step 3 — Self-host Geist and stage Lucide
Fontsource checks the family, weights, style, and subset at compile time. `host: Asset` downloads the selected files while bundling so the browser fetches them from Slipway's origin:

```rust
{{#include solution/src/assets.rs:fontsource-font}}
```

Iconify works one phase earlier. `build.rs` stages the Lucide set, then `include!` turns named icons into checked `IconData` constants. Icons render as inline SVG, so they need no asset request and inherit text color:

```rust
{{#include solution/src/assets.rs:iconify-icons}}
```

> [!NOTE]
> **Checkpoint:** misspell one included icon, run `cargo check -p lab11-solution`, and read the compile-time suggestion. Restore the correct name before continuing.

### Step 4 — Vendor Topcoat UI
From the repository root, run `topcoat ui init --package lab11-solution --components-dir src/components --theme neutral`, then `topcoat ui add --package lab11-solution card button input label`. The first command creates `components.toml` and `styles.css`; the second copies four Rust modules under `src/components/` and records their registry hashes.

The generated install state is committed with the app:

```toml
{{#include solution/components.toml}}
```

These modules are not wrappers around a hidden runtime package. They are your components now. Re-running `topcoat ui add card --overwrite` replaces local changes, so inspect the diff before accepting an upstream version.

> [!NOTE]
> **Checkpoint:** `topcoat ui list --package lab11-solution --installed` lists `button`, `card`, `input`, and `label`, and `git status --short` shows their source files.

### Step 5 — Own the theme and card
The installed theme exposes semantic tokens. Change the neutral values to Slipway's cyan, teal, and deep-water palette rather than scattering raw colors through every component:

```css
{{#include solution/styles.css:marina-theme}}
```

Then edit the vendored card's local class constant. This workshop adds a translucent surface and a three-color waterline across the top:

```rust
{{#include solution/src/components/card.rs:owned-card-style}}
```

Every berth card now calls this owned component. The integration test counts both the semantic marker and the customized class on all four rendered berths, proving the edit propagated rather than being copied page by page.

> [!NOTE]
> **Checkpoint:** `cargo test -p lab11-solution --test pages vendored_card_style_reaches_every_berth` passes.

### Step 6 — Load the release unit
The root layout loads the font stylesheet, generated Tailwind asset, Topcoat runtime, checksummed htmx asset, and local mark only when an asset bundle is registered:

```rust
{{#include solution/src/app/_marketing.rs:asset-head}}
```

Tests that focus on routing can still build a router without assets. The real binary loads `AssetBundle::load()` beside itself, and rendering an absent asset intentionally panics: that catches a binary/bundle mismatch instead of silently serving stale CSS.

> [!NOTE]
> **Checkpoint:** run `topcoat dev -p lab11-solution`, open view-source, and verify the CSS, htmx, logo, font files, and font stylesheet use `/_topcoat/` URLs rather than the old htmx CDN URL.

### Step 7 — Assert the hash contract
The asset test embeds a local declaration in its test binary, bundles that binary into a temporary directory, resolves the handle through the generated catalog, and checks the 16-character hexadecimal content hash:

```rust
{{#include solution/tests/assets.rs:hashed-asset-test}}
```

This tests the pipeline rather than guessing at a filename. A second page test ensures the htmx CDN URL disappeared and the vendored card customization reaches every berth.

> [!NOTE]
> **Checkpoint:** `cargo test -p lab11-solution --test assets --test pages` passes.

### Step 8 — Bundle the release profile
Topcoat CLI 0.7.0 has no `topcoat build` command. Build with Cargo and bundle with the same profile: `cargo build --release -p lab11-solution`, then `topcoat asset bundle --release -p lab11-solution`. The normal output is `target/release/assets/` beside `target/release/lab11-solution`.

Do not copy a debug asset directory beside a release binary. Tailwind's generated path includes Cargo's profile-specific `OUT_DIR`, so even identical CSS can have a different asset identity.

> [!NOTE]
> **Checkpoint:** `find target/release/assets -maxdepth 1 -type f -printf '%f\n' | sort` shows `manifest.toml` and content-hashed CSS, JavaScript, SVG, and font files.

## Stretch goals
- Add the vendored `badge` component and replace the berth status spans without changing their wording or data attributes.
- Commit a pinned Lucide JSON cache and configure `cache_dir` so clean offline builds can stage icons.
- Host the release bundle on a static origin with `AssetConfig::hosted_at` and verify only the URL prefix changes.
- Add dark-mode tokens and choose the `.dark` class from a server-side user preference rather than browser storage.

## Troubleshooting
- **`tailwindcss` cannot download.** The standalone CLI is fetched on the first build. Retry with network access or point `BuildConfig` at a preinstalled executable; npm is not required.
- **`iconify::include!` says the set was not staged.** Add the set to `build.rs`, keep `icon-iconify` in build dependencies, and rebuild.
- **`fontsource_font!` cannot find `topcoat_asset`.** Self-hosting expands through that crate; keep the workspace-pinned `topcoat-asset` dependency enabled for Lab 11.
- **Rendering panics with “failed to resolve asset.”** The loaded manifest came from a different binary, checkout, or profile. Re-run the bundler for the binary you will execute.
- **Tailwind omits a class.** Keep class names as literal strings in Rust or `styles.css`; dynamically assembled fragments are invisible to its scanner.
- **A local card edit vanished.** `topcoat ui add card --overwrite` replaced the owned file. Restore the diff, then reapply upstream changes deliberately.
- **`topcoat build` is unknown.** That command does not exist in CLI 0.7.0. Use Cargo plus `topcoat asset bundle` with matching profile flags.

## What's next
Lab 12 ships the release binary and its matching `target/release/assets/` directory together in a production container before deploying with `azd`.
