# Lab 13 — Tower bridge and Axum

**Time:** 45 min · **Module:** 6 · **Prerequisites:** Lab 12 complete (or copy `labs/lab-12-build-containerise-deploy/solution`)

## What you'll learn
- Mount an Axum router under `/api/v1` with `TowerRoute`
- Apply `tower-http` compression and tracing with `TowerLayer`
- Keep Topcoat as the primary app shell while exposing a JSON API alongside it

## Concepts (read first, 5 min)
The tower bridge is the “use both” story for a Topcoat app. Topcoat still owns the server-rendered shell and page model, but the Tower ecosystem is a mature HTTP middleware and service stack: timeouts, tracing, compression, policy layers, and existing Axum routers all slot into the same request pipeline. The point is not to replace Topcoat with Axum; the point is to keep the Topcoat app as the UI layer while attaching a small standard API beneath it when you need JSON, middleware, or a service that already exists in Tower/Axum. This is exactly the incremental migration story the [Topcoat announcement](https://tokio.rs/blog/2026-07-22-announcing-topcoat) calls out: mount a known-good service, keep the rest of the app in Topcoat, and add conversion only where it buys you real value.

The `TowerRoute` and `TowerLayer` bridge work at the router boundary, not at the component boundary. A mounted Axum router receives the original request URI under its subtree, and the middleware can wrap only the paths you need. That keeps the app easy to reason about in tests and easy to explain to learners: the Topcoat shell remains the “what the browser sees,” while the mounted router handles `/api/v1/*` traffic.

## Steps
### Step 1 — Mount an Axum router under `/api/v1`
Create a small Axum router with `/health`, then mount it under `/api/v1` with `TowerRoute`.
The adapter forwards the original URI. Add `StripPrefixLayer::new("/api/v1")` so this relative
Axum route receives `/health`, while Topcoat still matches the external `/api/v1/health` path.
The capstone instead defines an already-prefixed Axum route and does not strip it.

```rust
{{#include solution/src/lib.rs:tower-bridge-router}}
```

`TowerRoute::any` is shorthand for `TowerRoute::new(Methods::Any, ...)`.
This example keeps the general constructor to make method selection visible.
Read the tagged [Tower guide](https://raw.githubusercontent.com/tokio-rs/topcoat/v0.9.0/crates/topcoat-router/docs/tower.md).

> [!NOTE]
> **Checkpoint:** the app builds, and `curl http://127.0.0.1:3000/api/v1/health` returns `{"status":"ok"}`.

### Step 2 — Keep the Topcoat page intact
The Topcoat router still owns the public HTML pages. Add a single `#[page("/")]` root page to confirm the app shell still renders normally and that the mounted API is just a sibling route, not a replacement. This keeps the “Topcoat first, Tower second” story explicit for learners.

```rust
{{#include solution/src/lib.rs:tower-bridge-home}}
```

> [!NOTE]
> **Checkpoint:** `GET /` still renders the Topcoat HTML page; `GET /api/v1/health` returns JSON.

### Step 3 — Test the API route
Request the actual Topcoat `build_router()` rather than testing a separately nested Axum router.
Check the mounted JSON, method handling, missing API paths, and the ordinary Topcoat page:

```rust
{{#include solution/tests/tower_bridge.rs:tower-bridge-tests}}
```

> [!NOTE]
> **Checkpoint:** `cargo test -p lab13-solution --test tower_bridge` passes and confirms the `/api/v1/health` path works through the Tower bridge.

## Stretch goals
- Add a second `/api/v1/vessels` endpoint that returns a tiny JSON list from a static `Vec`.
- Wrap the API with a timeout or request-id layer and confirm the middleware shows up in the response headers or logs.
- Explore whether the Topcoat app can embed a `TowerService` fallback for the remaining unmatched routes in a later project.

## Troubleshooting
- **The mounted `/health` returns 404.** `TowerRoute` forwards `/api/v1/health`; add `StripPrefixLayer` for a relative Axum route, or define the full prefixed route without stripping it.
- **`TowerRoute` does not match `/api/v1` exactly.** Register the catch-all form `"/api/v1/{*rest}"` and, if you also serve the bare prefix, add a second route for `"/api/v1"`.
- **JSON response is empty or the `Content-Type` is wrong.** Return `axum::Json(...)` for the endpoint and make sure the route is nested inside the mounted Axum router.
- **Middleware does not wrap the API path.** Use `.at("/api/v1")` on each `TowerLayer` call; otherwise the layer wraps every route in the router.
- **Compilation fails on the tower bridge.** Enable the `tower` feature on `topcoat` and keep the `axum` and `tower-http` versions consistent with the workspace pins.

## What's next
Lab 13 is the bridge pattern: Topcoat remains the application shell and page renderer, while Axum/Tower fills the gap when a JSON API or mature HTTP middleware stack is the better fit.
