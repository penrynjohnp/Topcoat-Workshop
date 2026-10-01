# Lab 08 — Shards, procedures, and streaming

**Time:** 90 min · **Module:** 3 · **Prerequisites:** Lab 07 complete (or copy `labs/lab-07-signals-expressions/solution`)

## What you'll learn
- Re-render one server-owned region with a signal-parameter shard and stable identities
- Call an authenticated server function from an async handler with `#[procedure]`
- Stream progress, recover from errors, receive connected updates, and compare `Stream` with `Wait`

## Concepts (read first, 5 min)
Read the pinned [Topcoat runtime guide](https://docs.rs/topcoat/0.9.0/topcoat/runtime/index.html) and [`live!` guide](https://docs.rs/topcoat/0.9.0/topcoat/view/macro.live.html). Lab 07 introduced browser expressions and a whole-page tracked re-run. A shard narrows that server work to one region: the browser sends the current signal to a shard endpoint, the server recomputes just that fragment, and the result is morphed into place. A procedure crosses back for data or an action: an async event handler awaits an ordinary Rust server function and uses its result locally.

Both are public HTTP endpoints. Arguments are spoofable, page and layout guards do not protect them, and every endpoint must validate input and repeat authorization where needed. Shard input events may produce request storms: Topcoat 0.9.0 coalesces changes in one tick, aborts stale in-flight requests, and lets the latest result win, but it does not debounce pauses between keystrokes. Streaming solves a different problem—time to first content. `suspense` and `error_boundary` are convenience components built from the general `live!`/`emit!` replacement-region primitives.

## Steps
### Step 1 — Register server runtime endpoints
`.discover()` registers annotated application handlers, including shards and procedures.
`.runtime()` adds `RuntimeLayer` for page reruns and WebSocket connections.
Register application layers before `.runtime()` so they see reruns rewritten to GET:

```rust
{{#include solution/src/app.rs:app-router}}
```

The runtime script and asset-bundle arrangement from Lab 07 remains unchanged.
> [!NOTE]
> **Checkpoint:** `cargo check -p lab08-solution` succeeds and `/vessels` still renders.

### Step 2 — Search vessels with a shard
The page owns the query signal so the input stays outside the shard. In Topcoat 0.9.0, the caller creates the signal with `signal(cx, String::new)`, then hands the live signal handle to the shard as `Signal<String>` with `query: $(query)`. The handle itself is stable. The shard's `.get()` is a tracked server read, so changing the signal re-runs only that shard. The shard validates the value, filters server-owned vessel data, and returns only the result region:

```rust
{{#include solution/src/app/_marketing/vessels.rs:vessel-results-shard}}
```

```rust
{{#include solution/src/app/_marketing/vessels.rs:vessel-search-page}}
```

The initial page contains all vessels without an extra fetch. Later input sends a POST to the
complete URL emitted in the shard marker. Do not reconstruct a prefix from an endpoint ID.
The response is a fragment, not a document. The browser morphs it into the existing region.
> [!NOTE]
> **Checkpoint:** run `topcoat dev`, open `/vessels`, focus the search input, and type `Lady` slowly. Each shard response leaves focus in the input, and the partly typed query survives while the results morph to Lady Jane. Inspect the shard POST and its partial HTML response.

### Step 3 — Give reorderable results stable IDs
Morphing matches existing elements to returned elements. Position alone is ambiguous when a list can reorder, so give every vessel row a stable HTML `id` derived from the vessel identity:

```rust
{{#include solution/src/app/_marketing/vessels.rs:vessel-result-ids}}
```

The current search only filters, but the same rows can later be sorted by name, berth, or status. Their IDs let the morph move the existing nodes to new positions instead of rewriting the rows between them.
> [!NOTE]
> **Checkpoint:** inspect a result row and confirm Lady Jane renders as `id="vessel-lady-jane"`.

### Step 4 — Test the partial response directly
Generated endpoint paths are implementation details. The test reads the complete shard URL and
invocation identity from `/vessels`, then sends the browser's runtime request envelope:

```rust
{{#include solution/tests/server_runtime.rs:shard-partial-response-test}}
```

A `Signal<String>` argument is encoded as a signal surrogate inside the runtime envelope: `{ "args": [{ "t": "Signal", "id": "…", "v": "Lady" }], "signals": {} }`. The signals map carries the current values of any signals the shard body itself reads; this shard reads only its argument, so it is empty. The request also carries the shard identity header extracted from the initial page. The assertions prove that the endpoint returns the vessel-results `<section>` and stable row ID, but no doctype, `<html>`, `<body>`, nav, layout, or `<main>` wrapper. The test fails if the response expands beyond the fragment.
> [!NOTE]
> **Checkpoint:** `cargo test -p lab08-solution --test server_runtime vessel_shard_returns_only_the_filtered_fragment` passes.

### Step 5 — Complete work orders with a procedure
A procedure call runs only in an async browser position. The endpoint repeats `require_auth(cx)`, rejects unknown IDs, updates application-owned state, and returns a vocabulary type (`bool`):

```rust
{{#include solution/src/app/_marketing.rs:complete-work-order-procedure}}
```

Each row starts its signal from server state. Its async `@click` awaits the procedure, then updates class, disabled state, label, and text without replacing the page:

```rust
{{#include solution/src/app/_marketing.rs:work-order-procedure-ui}}
```

The `#[key(order.id)]` loop gives each row's signal stable context identity.
The HTML `id` identifies its DOM node; it does not replace the context key.
Procedure metadata contains a complete `path`, not an endpoint ID to prefix.

Do not rely on the dashboard's guard: a caller can POST directly to the procedure. Also note that procedure `Err` details are not observable by the expression; return a `Result<T, String>` as the successful data type when the UI must display failures.
> [!NOTE]
> **Checkpoint:** log in, open `/dashboard`, mark a work order complete, and observe one procedure POST plus the row updating in place.

### Step 6 — Stream progress with `live!` and `emit!`
A live region must emit once. Its first emission joins the initial document; later emissions replace the same region in the still-open response:

```rust
{{#include solution/src/app/_marketing/dashboard/history.rs:live-progress}}
```

This example emits three progress states. Use direct macros for progress, retry, or multiple replacements; use the convenience components for the common one-fallback/one-result shape.
> [!NOTE]
> **Checkpoint:** open `/dashboard/history` and inspect the document request's streamed `<template data-topcoat-swap>` chunks—there is no second fetch.

### Step 7 — Combine suspense and an error boundary
The history page authenticates before streaming begins. Default `Stream` suspense shows the
fallback while its slow child loads, but skips it if content is ready immediately.
The page's display selector uses `Wait` to make initial history available without JavaScript.
`error_boundary` replaces failed content even after the response starts:

```rust
{{#include solution/src/app/_marketing/dashboard/history.rs:streamed-history}}
```

Once streaming starts, status codes and cookies cannot be changed. Read sessions and set headers first; use an in-page boundary for recoverable late failures.
> [!NOTE]
> **Checkpoint:** `/dashboard/history` streams the history; `/dashboard/history?fail=true` keeps the page shell and replaces the history region with “History is unavailable.” `cargo test -p lab08-solution --test server_runtime` verifies both paths.

### Step 8 — Keep status current with a connected region
Add an `updates: tokio::sync::broadcast::Sender<()>` field to `AppState` and initialize it with
`tokio::sync::broadcast::channel(16).0`. Keep Tokio's `sync` feature enabled in the workspace.
Send a notification only after a work-order change succeeds:

```rust
{{#include solution/src/auth.rs:completion-notifications}}
```

Use an uncached lookup for each private emission rather than reusing the request's memoized user:

```rust
{{#include solution/src/auth.rs:fresh-live-session}}
```

Notify the same channel when a stored session is removed, so open regions react to logout.
Add a private status shard with an explicit URL and call it from the history page:

```rust
{{#include solution/src/app/_marketing/dashboard/history.rs:connected-work-order-status}}
```

Subscribe before reading current state. Emit a snapshot, then return its token when
`connected(cx)` is false. Only the connected render waits for later notifications.
The completion procedure sends a notification after changing application state.
A reconnect reloads state; a disconnect drops the subscription.
Check the stored session before each private emission, without reusing the memoized initial user.
`#[memoize]` caches for one request/render, and one connected live render keeps the same `Cx` across every loop iteration.
A memoized call inside the loop therefore returns its first result until a reconnection starts a fresh render.
Labs 08-09 use `current_user_uncached` for this reason; Labs 10-12 call `current_user` safely only because their DB-backed version is not memoized, not because the call site behaves differently.

The notification channel is process-local. It is not a durable event log or cross-replica fanout.

```rust
{{#include solution/tests/server_runtime.rs:connected-runtime-test}}
```

> [!NOTE]
> **Checkpoint:** keep history open in one authenticated tab and complete a work order in another. Status changes over the runtime WebSocket without polling. Reload `/dashboard/history?mode=wait` with JavaScript disabled and confirm the history is visible. The socket integration test covers updates, reconnects, logout, and subscription cleanup.

## Stretch goals
- Debounce vessel search with a small `raw!` timer, then compare requests with the direct version.
- Require two characters before invoking the shard and render a local prompt below that threshold.
- Return `Result<bool, String>` as procedure data and show an inline error signal for an unknown work-order ID.
- Add a fourth `emit!` state containing elapsed time, keeping all delays short in tests.
- Add a sort signal and reverse the vessel results, then confirm the stable row IDs let the morph move existing nodes.

## Troubleshooting
- **Shard route returns 404** → confirm `.runtime()` is on the router and the shard is reachable from compiled code so `.discover()` can register it.
- **Procedure route returns 404** → confirm `.runtime()` is on the router and the procedure is reachable from compiled code so `.discover()` can register it.
- **Shard POST returns 415/400** → send `Content-Type: application/json`, the `x-topcoat-identity` header, and the runtime envelope containing the `Signal` surrogate shown in the test.
- **The whole page re-runs while searching** → check for a tracked `.get()` in ordinary page Rust. Reads inside `$(...)` are client-reactive rather than whole-page dependencies.
- **A `Signal<String>` argument has a type error** → pass the handle as `$(query)`, not its string value as `$(query.get())`.
- **A signal resets after searching** → check its identity and the restored signal values. Key repeated stateful components with `#[key(item.id)]`.
- **Private data appears without the page guard** → shard/procedure endpoints bypass page/layout code. Call `require_auth(cx)` inside each private endpoint.
- **Every keystroke creates a request** → expected in this version; same-tick changes coalesce and stale requests abort, but debounce is application logic.
- **The DOM jumps while typing** → the browser is re-running the shard and morphing the region. Keep focus and partially typed text by keeping the signal in the page and giving reorderable items stable `id`s.
- **Cookie/header panic during streaming** → response headers were changed after the first emission. Authenticate and mutate cookies before returning the live view.
- **Only final streamed content appears in a test** → `to_bytes` collects the whole body; assert fallback text and `data-topcoat-swap` envelopes, or poll frames to observe timing.
- **The initial document never finishes** → emit current content and return when `connected(cx)` is false before waiting for notifications.
- **Another replica misses an update** → the example's notification channel is process-local; durable multi-replica delivery is outside this lab.

## What's next
Lab 09 rebuilds one server interaction with Topcoat's htmx helpers and compares native runtime, htmx, and Alpine trade-offs.
