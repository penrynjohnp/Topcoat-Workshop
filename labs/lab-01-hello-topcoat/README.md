# Lab 01 — Hello, Topcoat

**Time:** 30 min · **Module:** 1 · **Prerequisites:** `./scripts/setup.sh` has run cleanly

## What you'll learn
- Create a Topcoat project from a plain `cargo new` binary
- Run it with `cargo run`, then with the `topcoat dev` server and hot reload
- Write your first `#[page]` and `#[component]`

## Concepts (read first, 5 min)
Topcoat renders everything on the server. A **page** is an `async fn` bound to a URL; a **component** is an `async fn` that returns `Result<impl View>` — usually `Ok(view! { ... })`. `Router::builder().discover()` finds every `#[page]` in the binary so you don't register routes by hand. `topcoat::dev::script()` injects a tiny script that reloads the browser when `topcoat dev` finishes a rebuild.

## Steps

### Step 1 — Run the starter
```bash
cd labs/lab-01-hello-topcoat/starter
cargo run
```
> [!NOTE]
> **Checkpoint:** http://127.0.0.1:3000 shows **Hello, World!**

### Step 2 — Switch to the dev server
Stop `cargo run` and instead:
```bash
topcoat dev
```
It builds, bundles assets, starts the server, and watches your source tree.
> [!NOTE]
> **Checkpoint:** the terminal shows the server listening on 127.0.0.1:3000.

### Step 3 — Add hot reload
In `src/main.rs`, replace the first `TODO(lab-01)` with:
```rust
topcoat::dev::script()
```
Save. Change the `<h1>` text and save again.
> [!NOTE]
> **Checkpoint:** the browser reloads by itself with the new text — no manual refresh.

### Step 4 — Your first component
Below `home`, add:
```rust
#[component]
async fn hello(name: &str) -> Result<impl View> {
    Ok(view! {
        <h1>"Hello, " (name) "!"</h1>
    })
}
```
Then in `home`, replace the `<h1>` with `hello(name: "Topcoat")`. Components are invoked like functions, with named arguments, directly inside `view!`; `(name)` interpolates a Rust expression. Pages and components return `Result<impl View>` — the `Ok(...)` around `view!` is what lets a component bail out with an error (a 404, a redirect) instead of markup, which Lab 06 relies on.
> [!NOTE]
> **Checkpoint:** page shows **Hello, Topcoat!** Compare with `../solution/src/lib.rs`.

### Step 5 — Bind address
```bash
HOST=0.0.0.0 PORT=8080 topcoat dev
```
> [!NOTE]
> **Checkpoint:** http://localhost:8080 works. Remember this — the `127.0.0.1` default is the classic "deployed but unreachable" mistake in Lab 12.

## Stretch goals
- Add a second `#[page("/about")]` and a link between the two pages.
- Run `topcoat fmt` after deliberately mangling the indentation inside `view!`.

## Troubleshooting
- `topcoat: command not found` → `cargo install topcoat-cli --version 0.7.0 --locked` and check `~/.cargo/bin` is on `PATH`.
- Compile error mentioning a missing import → compare the `use topcoat::{...}` block against the solution; the facade crate re-exports through feature-gated modules.

## What's next
Lab 02 takes the `view!` macro further: loops, conditionals, and conditional attributes, building Slipway's first real page.
