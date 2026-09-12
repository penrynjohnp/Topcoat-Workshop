# Lab 08 — Shards, procedures, and streaming

**Time:** 75 min · **Module:** 3 · **Prerequisites:** Lab 07 complete (or copy `labs/lab-07-signals-expressions/solution`)

## What you'll learn
- Re-render one server-owned HTML region with `#[shard]`
- Pass a browser signal into a shard as `Signal<String>` and call it with `$(query)`
- Call an authenticated server function from an async handler with `#[procedure]`
- Stream fallbacks, progress, content, and recoverable errors with `live!`, `suspense`, and `error_boundary`

## Concepts (read first, 5 min)
Read the pinned [Topcoat runtime guide](https://docs.rs/topcoat/0.8.0/topcoat/runtime/index.html) and [`live!` guide](https://docs.rs/topcoat/0.8.0/topcoat/view/macro.live.html). Lab 07 introduced browser expressions and a whole-page tracked re-run. A shard narrows that server work to one region: the browser sends the current signal to a shard endpoint, the server recomputes just that fragment, and the result is morphed into place. A procedure crosses back for data or an action: an async event handler awaits an ordinary Rust server function and uses its result locally.

Both are public HTTP endpoints. Arguments are spoofable, page and layout guards do not protect them, and every endpoint must validate input and repeat authorization where needed. Shard input events may produce request storms: Topcoat 0.8.0 coalesces changes in one tick, aborts stale in-flight requests, and lets the latest result win, but it does not debounce pauses between keystrokes. Streaming solves a different problem—time to first content. `suspense` and `error_boundary` are convenience components built from the general `live!`/`emit!` replacement-region primitives.

## Steps
### Step 1 — Register server runtime endpoints
The two router calls have different jobs. `.runtime()` mounts the runtime's own HTTP routes. `.discover()` registers the annotated application handlers, including pages, shards, procedures, layouts, and routes, before the builder adds cookies, sessions, mail, app context, and the optional asset bundle:

```rust
{{#include solution/src/app.rs:app-router}}
```

The runtime script and asset-bundle arrangement from Lab 07 remains unchanged.
> [!NOTE]
> **Checkpoint:** `cargo check -p lab08-solution` succeeds and `/vessels` still renders.

### Step 2 — Search vessels with a shard
The page owns the query signal so the input stays outside the shard. In Topcoat 0.8.0, the caller creates the signal with `signal(cx, String::new)`, then hands the live signal handle to the shard as `Signal<String>` with `query: $(query)`. The handle itself is stable. The shard's `.get()` is a tracked server read, so changing the signal re-runs only that shard. The shard validates the value, filters server-owned vessel data, and returns only the result region:

```rust
{{#include solution/src/app/_marketing/vessels.rs:vessel-results-shard}}
```

```rust
{{#include solution/src/app/_marketing/vessels.rs:vessel-search-page}}
```

Open DevTools: the initial page contains all vessels without an extra fetch. Each later input sends a POST under `/_topcoat/runtime/shards/<id>`; the response is a fragment, not a document. The browser morphs the new fragment into the existing DOM instead of replacing the region wholesale.
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
Generated endpoint IDs are implementation details, so the test renders `/vessels`, reads the shard path and identity from its marker, and sends the same runtime request envelope as the browser:

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
The history page authenticates before streaming begins. `suspense` emits “Loading history…” immediately, then replaces it with the slow child. `error_boundary` replaces that region with a fallback if the child fails after the response has started:

```rust
{{#include solution/src/app/_marketing/dashboard/history.rs:streamed-history}}
```

Once streaming starts, status codes and cookies cannot be changed. Read sessions and set headers first; use an in-page boundary for recoverable late failures.
> [!NOTE]
> **Checkpoint:** `/dashboard/history` streams the history; `/dashboard/history?fail=true` keeps the page shell and replaces the history region with “History is unavailable.” `cargo test -p lab08-solution --test server_runtime` verifies both paths.

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
- **The whole page re-runs while searching** → pass the handle as `$(query)` to a `Signal<String>` shard parameter. Passing `$(query.get())` makes the caller track the value instead.
- **A signal resets after searching** → it was declared inside the shard. Create it in the page and pass the signal handle as `$(query)`.
- **Private data appears without the page guard** → shard/procedure endpoints bypass page/layout code. Call `require_auth(cx)` inside each private endpoint.
- **Every keystroke creates a request** → expected in this version; same-tick changes coalesce and stale requests abort, but debounce is application logic.
- **The DOM jumps while typing** → the browser is re-running the shard and morphing the region. Keep focus and partially typed text by keeping the signal in the page and giving reorderable items stable `id`s.
- **Cookie/header panic during streaming** → response headers were changed after the first emission. Authenticate and mutate cookies before returning the live view.
- **Only final streamed content appears in a test** → `to_bytes` collects the whole body; assert fallback text and `data-topcoat-swap` envelopes, or poll frames to observe timing.

## What's next
Lab 09 rebuilds one server interaction with Topcoat's htmx helpers and compares native runtime, htmx, and Alpine trade-offs.
