# Lab 07 — Signals and `$(...)` expressions

**Time:** 60 min · **Module:** 3 · **Prerequisites:** Lab 06 complete (or copy `labs/lab-06-auth-sessions-mail/solution`)

## What you'll learn
- Declare browser state with `signal` and read it inside a `$(...)` runtime expression
- Drive the DOM with `@click`/`@input` handlers and `:hidden`/`:class`/`:value` bind attributes
- Recognise what the shared Rust/JavaScript vocabulary does not support, and read the compile error it produces

## Concepts (read first, 5 min)
Read the pinned Topcoat runtime and `expr!` guides. A `$(...)` block is a **runtime expression**: type-checked Rust that Topcoat compiles twice, once to server code that produces the initial HTML and once to JavaScript that ships with the page. A `signal` statement inside a `view!` body declares state that lives in the browser; its initial value is an ordinary Rust expression evaluated during the server render. When a signal changes, every expression that read it re-runs in the browser. There is no WebAssembly, no client build step, and no round-trip.

Because an expression must mean the same thing in both languages, only a small vocabulary is supported: `f64` (all numbers, so integer literals are rejected), `bool`, `String`/`&str`, `Option`, `Result`, tuples, and `Signal`, each with a subset of its Rust API. `match`, struct expressions, multi-segment paths, and the `&&`/`||` operators are not in it — you spell combinations with `if`/`else` and `let` bindings instead. Anything the vocabulary cannot say is either restructured or escaped to hand-written JavaScript with `raw!`. This lab is deliberately client-only: no `#[shard]`, no `#[procedure]` — those arrive in Lab 08.

## Steps
### Step 1 — Load the runtime script
Interactive pages need the runtime's browser script, and that script is a Topcoat asset — so the router has to have an asset bundle loaded. Rendering an asset that is not in the loaded bundle panics, and integration tests run from a binary with no bundle beside it, so the router takes the bundle as an `Option`:

```rust
{{#include solution/src/app.rs:app-router}}
```

`main.rs` passes `Some(AssetBundle::load()?)`; tests pass `None`. The layout reads the resulting flag and renders the script tag only when it can actually be served:

```rust
{{#include solution/src/app/_marketing.rs:runtime-script}}
```

`topcoat dev` rebuilds and re-bundles assets after every successful build, so nothing extra is needed while developing.
> [!NOTE]
> **Checkpoint:** `cargo test -p lab07-solution --test pages` still passes, and `topcoat dev` serves `/berths` with a `<script type="module" src="/_topcoat/assets/...">` in the head.

### Step 2 — Declare signals and filter berths in the browser
Add three signals to the berths page: the search text and one `bool` per status chip. The input syncs both ways (`:value` reads the signal, `@input` writes it), and each chip flips its signal with `toggle`:

```rust
{{#include solution/src/app/_marketing/berths.rs:berth-filter}}
```

Two details matter. `name` and `occupied` are **captured** from the surrounding Rust scope, so each row's expression closes over a snapshot taken during the render — later server-side changes never reach it. And `&&`/`||` are not in the vocabulary, which is why the `:hidden` expression is a block with `let` bindings and an `if`/`else`.
> [!NOTE]
> **Checkpoint:** run `topcoat dev`, open `/berths`, type `A` — only A1 and A2 stay. Click "Occupied" and A1 disappears too. The Network tab stays silent throughout.

### Step 3 — Collapse the work-order panel
The protected work-orders component from Lab 06 becomes a disclosure. One `bool` signal drives four things at once: the button label, its `class`, its `aria-expanded` state, and the panel's `hidden` attribute:

```rust
{{#include solution/src/app/_marketing.rs:work-order-panel}}
```

`require_auth(cx).await?` still runs on the server before any of this renders — reactivity does not weaken the guard, it only decides what the browser does with markup it was already allowed to receive. Never hide privileged data behind `:hidden`; the HTML is still in the page.
> [!NOTE]
> **Checkpoint:** log in via the magic link, open `/dashboard`, and toggle the panel. `aria-expanded` flips in the Elements panel with no request.

### Step 4 — Break it on purpose
Replace the button's label expression with the `match` you would write in ordinary Rust:

```rust
$(match open.get() {
    true => "Hide work orders",
    false => "Show work orders",
})
```

`cargo check -p lab07-solution` rejects it, pointing at the whole expression:

```text
error: unsupported expression
  --> labs/lab-07-signals-expressions/solution/src/app/_marketing.rs:64:19
   |
64 |                   $(match open.get() {
   |  ___________________^
65 | |                     true => "Hide work orders",
66 | |                     false => "Show work orders",
67 | |                 })
   | |_________________^
```

The rejection is the point: the expression has to compile to JavaScript as well as to Rust, and `match` has no equivalent in the shared vocabulary. The commented-out version stays in the solution so you can reproduce the error at any time:

```rust
{{#include solution/src/app/_marketing.rs:unsupported-expression}}
```

Integer literals fail the same way, and that surprises people more: every number in an expression is an `f64`, so `$(count.get() + 1)` does not compile while `$(count.get() + 1.0)` does.
> [!NOTE]
> **Checkpoint:** you have seen `error: unsupported expression` yourself, and the `if`/`else` spelling is back in place.

### Step 5 — Know your escape hatches
Three exist, in increasing order of desperation. Restructure the expression (as `:hidden` does with `if`/`else`); drop to a raw JavaScript string for a handler (`@click="alert('hi')"`); or use `raw!("${n}.toUpperCase()", n.to_uppercase())`, which takes the JavaScript and the equivalent Rust so the server can still render it. Without that second argument the expression can no longer be evaluated on the server, and keeping the two sides equivalent is your responsibility either way.

The name filter in Step 2 is case-sensitive precisely because `to_lowercase` is not in the vocabulary — the stretch goals ask you to fix that.
> [!NOTE]
> **Checkpoint:** you can say which of the three you would reach for first, and why `raw!` is last.

### Step 6 — Prove it with a test
The reactive markup is server-rendered HTML, so an in-process router test can assert on it: signals serialize as `::topcoat::signal(...)` comments, handlers as `data-topcoat-on:*` attributes, binds as `data-topcoat-bind:*`.

```rust
{{#include solution/tests/reactivity.rs:reactivity-integration-test}}
```

The router is built with `None` for the bundle, so no asset is rendered and the test needs nothing on disk. The last test is the machine-checkable form of this lab's checkpoint: no shard or procedure endpoint appears anywhere in the page.
> [!NOTE]
> **Checkpoint:** `cargo test -p lab07-solution --test reactivity` passes.

## Stretch goals
- Make the name filter case-insensitive. The vocabulary has no `to_lowercase`, so either lowercase the captured name at render time and the query with `raw!`, or keep a second signal holding the lowered query.
- Add a "clear filters" button that resets all three signals in one handler — a handler body can be a block.
- Show a live count of visible berths. The vocabulary has no `Vec`, so you will need one signal per row, or a rethink; this is a good preview of why Lab 08 reaches for a `#[shard]`.
- Persist the chip state across navigation without browser storage. Hint: a signal's initial value is ordinary Rust and can read the query string from `Cx`.

## Troubleshooting
- **`error: unsupported operator` on `&&` or `||`** — `bool` has `!`, comparisons, `then` and `then_some`, but no logical operators. Use `if`/`else` with `let` bindings.
- **`error: unsupported expression`** — `match`, struct literals, or multi-segment paths. Restructure, or escape with `raw!`.
- **Type errors around `1` or `2`** — integer literals are not accepted; write `1.0`.
- **The page renders but nothing reacts** — the runtime script is missing. Check the router was built with a bundle and that `topcoat dev` finished a build; a bundle from another profile does not describe your binary.
- **Panic: an asset is not present in the loaded bundle** — binary and bundle came from different builds. Rebuild both, or pass `None` if you are in a test.
- **`internal compiler error: encountered incremental compilation error`** after uncommenting the `match` — an incremental-cache artifact, not your code. Run `cargo clean -p lab07-solution` and try again.
- **A captured value never updates** — captures are snapshots taken during the render. Anything that must track server state belongs in Lab 08's shard.

## What's next
Lab 08 crosses the boundary the other way: `#[shard]` re-renders markup on the server as its arguments change, and `#[procedure]` calls server functions straight from a handler.
