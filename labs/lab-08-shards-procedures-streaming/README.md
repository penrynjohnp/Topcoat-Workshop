# Lab 08 — Shards, procedures, and streaming

**Time:** 75 min · **Module:** 3 · **Prerequisites:** Lab 07 complete (or copy `labs/lab-07-signals-expressions/solution`)

## What you'll learn
- Re-render one server-owned HTML region with `#[shard]`
- Call an authenticated server function from an async handler with `#[procedure]`
- Stream fallbacks, progress, content, and recoverable errors with `live!`, `suspense`, and `error_boundary`

## Concepts (read first, 5 min)
Read the pinned [Topcoat runtime guide](https://docs.rs/topcoat/0.7.0/topcoat/runtime/index.html) and [`live!` guide](https://docs.rs/topcoat/0.7.0/topcoat/view/macro.live.html). Lab 07 kept every interaction in the browser. A shard crosses back to the server for markup: its initial render is inline, then changing a runtime-expression argument POSTs to the shard endpoint and replaces only that region. A procedure crosses back for data or an action: an async event handler awaits an ordinary Rust server function and uses its result locally.

Both are public HTTP endpoints. Arguments are spoofable, page and layout guards do not protect them, and every endpoint must validate input and repeat authorization where needed. Shard input events may produce request storms: Topcoat 0.7 coalesces changes in one tick, aborts stale in-flight requests, and lets the latest result win, but it does not debounce pauses between keystrokes. Streaming solves a different problem—time to first content. `suspense` and `error_boundary` are convenience components built from the general `live!`/`emit!` replacement-region primitives.

## Steps
### Step 1 — Register server runtime endpoints
`module_router!()` discovers pages, but Lab 08 explicitly discovers the linked shard and procedure routes before adding cookies, sessions, mail, app context, and the optional asset bundle:

```rust
{{#include solution/src/app.rs:app-router}}
```

The runtime script and asset-bundle arrangement from Lab 07 remains unchanged.
> [!NOTE]
> **Checkpoint:** `cargo check -p lab08-solution` succeeds and `/berths` still renders.

### Step 2 — Search vessels with a shard
The page owns the query signal so it survives shard replacements. The shard receives a `String`, validates its length, filters server-owned vessel data, and returns only the result region:

```rust
{{#include solution/src/app/_marketing/vessels.rs:vessel-results-shard}}
```

```rust
{{#include solution/src/app/_marketing/vessels.rs:vessel-search-page}}
```

Open DevTools: the initial page contains all vessels without an extra fetch. Each later input sends a POST under `/_topcoat/shards/`; the response is a fragment, not a document. State declared *inside* the shard would reset on every replacement, which is why `query` lives outside it.
> [!NOTE]
> **Checkpoint:** run `topcoat dev`, open `/vessels`, type `Lady`, and see only Lady Jane without a page navigation. Inspect the shard POST and its partial HTML response.

### Step 3 — Test the partial response directly
Generated endpoint IDs are implementation details, so the test renders `/vessels`, reads the shard path from the reactive-scope marker, and sends the same surrogate tuple the browser sends:

```rust
{{#include solution/tests/server_runtime.rs:shard-partial-response-test}}
```

A single `String` argument is encoded as the JSON tuple `["Lady"]`. The assertions prove that the endpoint returns matching vessel markup but no doctype, nav, layout, or `<main>` wrapper.
> [!NOTE]
> **Checkpoint:** `cargo test -p lab08-solution --test server_runtime vessel_shard_returns_only_the_filtered_fragment` passes.

### Step 4 — Complete work orders with a procedure
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

### Step 5 — Stream progress with `live!` and `emit!`
A live region must emit once. Its first emission joins the initial document; later emissions replace the same region in the still-open response:

```rust
{{#include solution/src/app/_marketing/dashboard/history.rs:live-progress}}
```

This example emits three progress states. Use direct macros for progress, retry, or multiple replacements; use the convenience components for the common one-fallback/one-result shape.
> [!NOTE]
> **Checkpoint:** open `/dashboard/history` and inspect the document request's streamed `<template data-topcoat-swap>` chunks—there is no second fetch.

### Step 6 — Combine suspense and an error boundary
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

## Troubleshooting
- **Shard route returns 404** → call `.discover_shards()` on the router and ensure the shard is linked into the binary.
- **Procedure route returns 404** → call `.discover_procedures()` and render/capture the procedure from reachable code.
- **Shard POST returns 415/400** → send `Content-Type: application/json` and a tuple-shaped body such as `["Lady"]`.
- **A signal resets after searching** → it was declared inside the shard. Move state outside and pass it as an argument.
- **Private data appears without the page guard** → shard/procedure endpoints bypass page/layout code. Call `require_auth(cx)` inside each private endpoint.
- **Every keystroke creates a request** → expected in this version; same-tick changes coalesce and stale requests abort, but debounce is application logic.
- **Cookie/header panic during streaming** → response headers were changed after the first emission. Authenticate and mutate cookies before returning the live view.
- **Only final streamed content appears in a test** → `to_bytes` collects the whole body; assert fallback text and `data-topcoat-swap` envelopes, or poll frames to observe timing.

## What's next
Lab 09 rebuilds one server interaction with Topcoat's htmx helpers and compares native runtime, htmx, and Alpine trade-offs.
