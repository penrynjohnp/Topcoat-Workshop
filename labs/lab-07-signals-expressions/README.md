# Lab 07 — Signals and `$(...)` expressions

**Time:** 60 min · **Module:** 3 · **Prerequisites:** Lab 06 complete (or copy `labs/lab-06-auth-sessions-mail/solution`)

## What you'll learn
- Create browser state with `signal(cx, || initial)` before entering `view!`
- Pass signal handles to child components as `&Signal<T>`
- Distinguish client-side `$(...)` reads from tracked and untracked server-side reads
- Drive the DOM with `@click`/`@input` handlers and `:hidden`/`:class`/`:value` bind attributes
- Recognise what the shared Rust/JavaScript vocabulary does not support, and read the compile error it produces

## Concepts (read first, 5 min)
 Read the pinned Topcoat v0.8.0 [runtime guide](https://raw.githubusercontent.com/tokio-rs/topcoat/v0.8.0/crates/topcoat/docs/runtime.md) and expr! [vocabulary](https://raw.githubusercontent.com/tokio-rs/topcoat/v0.8.0/crates/topcoat-runtime/macro/docs/expr.md). A `$(...)` block is a **runtime expression**: type-checked Rust that Topcoat compiles twice, once to server code that produces the initial HTML and once to JavaScript that ships with the page. Create a signal in the surrounding Rust body with `signal(cx, || initial)`, then capture it in as many runtime expressions as you need. The body needs `cx: &Cx` because Topcoat derives the signal's stable identity from the current page, layout, component, or shard render.

A signal is an ordinary, cheaply cloned handle. Pass it to a child component as `&Signal<T>`. A `.get()` inside `$(...)` is a browser-side reactive read and does not make the server body depend on the signal. A `.get()` in ordinary Rust is a tracked server-side read: changing that signal asks the server to run the page again and morph the returned HTML into place. Use `.get_untracked()` when the server needs the current value without subscribing the body to future changes.

Because a runtime expression must mean the same thing in Rust and JavaScript, only a small vocabulary is supported: `f64` (all numbers, so integer literals are rejected), `bool`, `String`/`&str`, `Option`, `Result`, tuples, and `Signal`, each with a subset of its Rust API. `match`, struct expressions, multi-segment paths, and the `&&`/`||` operators are not in it — spell combinations with `if`/`else` and `let` bindings instead. Anything the vocabulary cannot say is either restructured or escaped to hand-written JavaScript with `raw!`. The filter and disclosure in this lab remain client-only; one focused example intentionally adds a page re-run. `#[shard]` and `#[procedure]` arrive in Lab 08.

## Steps
### Step 1 — Load and route the runtime
Interactive pages need the runtime's browser script, and that script is a Topcoat asset. Tracked server reads also need `.runtime()` on the router so the browser has an endpoint that can re-run the page. Rendering an asset that is not in the loaded bundle panics, and integration tests run from a binary with no bundle beside it, so the router takes the bundle as an `Option`:

```rust
{{#include solution/src/app.rs:app-router}}
```

Import `RouterBuilderRuntimeExt`, call `.runtime()`, and keep the asset bundle conditional. `main.rs` passes `Some(AssetBundle::load()?)`; tests pass `None`. The layout reads the resulting flag and renders the script tag only when it can actually be served:

```rust
{{#include solution/src/app/_marketing.rs:runtime-script}}
```

`topcoat dev` rebuilds and re-bundles assets after every successful build, so nothing extra is needed while developing.
> [!NOTE]
> **Checkpoint:** `cargo test -p lab07-solution --test pages` still passes, and `topcoat dev` serves `/berths` with a `<script type="module" src="/_topcoat/assets/...">` in the head.

### Step 2 — Create signals and filter berths in the browser
Give the berths page a `cx: &Cx` parameter. Before `view!`, create the search text and one `bool` per status chip with the ordinary Rust `signal` function. Each initializer is a closure, such as `|| String::from("")` or `|| true`:

```rust
{{#include solution/src/app/_marketing/berths.rs:berth-filter}}
```

The input syncs both ways: `:value` reads the signal, while `@input` writes it. Each chip flips its signal with `toggle`. These `.get()` calls are inside `$(...)`, so they run reactively in the browser and do not subscribe the page to a server re-run.

Two details matter. `name` and `occupied` are **captured** from the surrounding Rust scope, so each row's expression closes over a snapshot taken during the render. And `&&`/`||` are not in the vocabulary, which is why the `:hidden` expression is a block with `let` bindings and an `if`/`else`.
> [!NOTE]
> **Checkpoint:** run `topcoat dev`, open `/berths`, type `A` — only A1 and A2 stay. Click "Occupied" and A1 disappears too. The Network tab stays silent throughout.

### Step 3 — Read signals on the server
Add a focused demonstration to the home page. The parent component creates two signals, then passes both handles to a child as `&Signal<String>`. The child reads one with `.get()` and the other with `.get_untracked()` in ordinary Rust, outside any `$(...)` expression:

```rust
{{#include solution/src/app/_marketing.rs:server-read-signals}}
```

The tracked read emits a dependency marker. Changing that signal in the browser requests a fresh page render, and Topcoat morphs the returned HTML into the existing page. Changing only the untracked signal does not initiate a request. If another tracked change later causes a re-run, the server can still receive the untracked signal's current value; “untracked” means “do not subscribe,” not “ignore the value.”

> [!WARNING]
> Every signal value read on the server is user input controlled by the browser. Validate it before using it for authorization, database access, prices, identifiers, or any other trusted decision.

> [!NOTE]
> **Checkpoint:** open `/` with the Network tab visible. Typing in “Untracked value” causes no request. Typing in “Tracked value” requests a page re-run and updates the server-rendered readout.

### Step 4 — Collapse the work-order panel
The protected work-orders component from Lab 06 becomes a disclosure. Create its `open` signal with `signal(cx, || false)` before `view!`. That one `bool` drives four things at once: the button label, its `class`, its `aria-expanded` state, and the panel's `hidden` attribute:

```rust
{{#include solution/src/app/_marketing.rs:work-order-panel}}
```

`require_auth(cx).await?` still runs on the server before any of this renders — reactivity does not weaken the guard, it only decides what the browser does with markup it was already allowed to receive. Never hide privileged data behind `:hidden`; the HTML is still in the page.
> [!NOTE]
> **Checkpoint:** log in via the magic link, open `/dashboard`, and toggle the panel. `aria-expanded` flips in the Elements panel with no request.

### Step 5 — Break it on purpose
Replace the button's label expression with the `match` you would write in ordinary Rust:

```rust
$(match open.get() {
    true => "Hide work orders",
    false => "Show work orders",
})
```

`cargo check -p lab07-solution` rejects it, pointing at the whole `match` expression:

```text
error: unsupported expression
```

The exact line number depends on the steps you have completed.

The rejection is the point: the expression has to compile to JavaScript as well as to Rust, and `match` has no equivalent in the shared vocabulary. The commented-out version stays in the solution so you can reproduce the error at any time:

```rust
{{#include solution/src/app/_marketing.rs:unsupported-expression}}
```

Integer literals fail the same way, and that surprises people more: every number in an expression is an `f64`, so `$(count.get() + 1)` does not compile while `$(count.get() + 1.0)` does.
> [!NOTE]
> **Checkpoint:** you have seen `error: unsupported expression` yourself, and the `if`/`else` spelling is back in place.

### Step 6 — Know your escape hatches
Three exist, in increasing order of desperation. Restructure the expression (as `:hidden` does with `if`/`else`); drop to a raw JavaScript string for a handler (`@click="alert('hi')"`); or use `raw!("${n}.toUpperCase()", n.to_uppercase())`, which takes the JavaScript and the equivalent Rust so the server can still render it. Without that second argument the expression can no longer be evaluated on the server, and keeping the two sides equivalent is your responsibility either way.

The name filter in Step 2 is case-sensitive precisely because `to_lowercase` is not in the vocabulary — the stretch goals ask you to fix that.
> [!NOTE]
> **Checkpoint:** you can say which of the three you would reach for first, and why `raw!` is last.

### Step 7 — Prove it with tests
The reactive markup is server-rendered HTML, so an in-process router test can assert on it: signals serialize as `::topcoat::signal(...)` comments, handlers as `data-topcoat-on:*` attributes, binds as `data-topcoat-bind:*`.

```rust
{{#include solution/tests/reactivity.rs:reactivity-integration-test}}
```

The router is built with `None` for the bundle, so no asset is rendered and the test needs nothing on disk. The tracked-read test asserts that two signals produce exactly one `::topcoat::dep(...)` marker. The filter test also confirms that no shard or procedure endpoint is involved.
> [!NOTE]
> **Checkpoint:** `cargo test -p lab07-solution --test reactivity` passes.

## Stretch goals
- Validate and normalize the tracked text before using it in the server-rendered readout. Treat the value as hostile input even though it has the expected Rust type.
- Make the name filter case-insensitive. The vocabulary has no `to_lowercase`, so either lowercase the captured name at render time and the query with `raw!`, or keep a second signal holding the lowered query.
- Add a "clear filters" button that resets all three signals in one handler — a handler body can be a block.
- Show a live count of visible berths. The vocabulary has no `Vec`, so you will need one signal per row, a tracked server read, or a rethink; this previews why Lab 08 reaches for a `#[shard]`.
- Persist the chip state across navigation without browser storage. Hint: a signal's initializer is ordinary Rust and can read the query string from `Cx`.

## Troubleshooting
- **`expected view node` at `signal name = value;`** — that was the pre-0.8 syntax. Import `signal`, accept `cx: &Cx`, and create the signal before `view!` with `let name = signal(cx, || value);`.
- **A tracked signal changes but no request appears** — add `.runtime()` to the router and confirm the runtime script and matching asset bundle are loaded.
- **`error: unsupported operator` on `&&` or `||`** — `bool` has `!`, comparisons, `then` and `then_some`, but no logical operators. Use `if`/`else` with `let` bindings.
- **`error: unsupported expression`** — `match`, struct literals, or multi-segment paths. Restructure, or escape with `raw!`.
- **Type errors around `1` or `2`** — integer literals are not accepted; write `1.0`.
- **The page renders but nothing reacts** — the runtime script is missing. Check the router was built with a bundle and that `topcoat dev` finished a build; a bundle from another profile does not describe your binary.
- **Panic: an asset is not present in the loaded bundle** — binary and bundle came from different builds. Rebuild both, or pass `None` if you are in a test.
- **`internal compiler error: encountered incremental compilation error`** after uncommenting the `match` — an incremental-cache artifact, not your code. Run `cargo clean -p lab07-solution` and try again.
- **An untracked server read changes only after another signal triggers a request** — expected. Untracked reads do not subscribe the page, but the next re-run still receives the browser's current signal values.
- **A captured value never updates** — ordinary captured Rust values are snapshots taken during the render. Use a signal for browser state or a tracked server read/shard for server-rendered state.

## What's next
Lab 08 crosses the boundary the other way: `#[shard]` re-renders markup on the server as its arguments change, and `#[procedure]` calls server functions straight from a handler.
