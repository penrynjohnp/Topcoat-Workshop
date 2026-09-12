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
Start by creating a tiny Axum router with a JSON health endpoint, then mount it with `TowerRoute` from the Topcoat router. The key is the catch-all route: it forwards the request subtree to the nested service without rewriting the path. This is the “route under a prefix” pattern Topcoat expects when you want an existing Axum app to live beside the page routes.

```rust
use axum::{Json, Router as AxumRouter, routing::get};
use serde::Serialize;
use topcoat::router::{Methods, Router, RouterBuilderDiscoverExt, tower::{TowerLayer, TowerRoute}};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

fn api_router() -> AxumRouter {
    AxumRouter::new().route(
        "/health",
        get(|| async { Json(HealthResponse { status: "ok" }) }),
    )
}

fn build_router() -> Router {
    Router::builder()
        .route(TowerRoute::new(Methods::Any, "/api/v1/{*rest}", api_router()))
        .layer(TowerLayer::new(CompressionLayer::new()).at("/api/v1"))
        .layer(TowerLayer::new(TraceLayer::new_for_http()).at("/api/v1"))
        .build()
}
```

Topcoat 0.8.0 also adds `TowerRoute::any` as the catch-all shorthand: `Router::builder().route(TowerRoute::any("/api/v1/{*rest}", api_router()))`. The main example keeps `TowerRoute::new(Methods::Any, ...)` so you can still see how method selection fits the general constructor.

> [!NOTE]
> **Checkpoint:** the app builds, and `curl http://127.0.0.1:3000/api/v1/health` returns `{"status":"ok"}`.

### Step 2 — Keep the Topcoat page intact
The Topcoat router still owns the public HTML pages. Add a single `#[page("/")]` root page to confirm the app shell still renders normally and that the mounted API is just a sibling route, not a replacement. This keeps the “Topcoat first, Tower second” story explicit for learners.

```rust
use topcoat::{Result, router::page, view::{View, view}};

#[page("/")]
async fn home() -> Result<impl View> {
    Ok(view! {
        <!DOCTYPE html>
        <html>
            <body>
                <h1>"Slipway + Axum"</h1>
                <p>"API: /api/v1/health"</p>
            </body>
        </html>
    })
}
```

> [!NOTE]
> **Checkpoint:** `GET /` still renders the Topcoat HTML page; `GET /api/v1/health` returns JSON.

### Step 3 — Test the API route
Use a tower-compatible request against the Topcoat router and assert the mounted JSON. The test is more valuable than a browser smoke test here because it proves the route is mounted under the right path and returns the expected payload, not just that the server starts.

```rust
#[tokio::test]
async fn api_v1_health_route_works() {
    use axum::body::Body;
    use http::{Request, StatusCode};
    use tower::ServiceExt;

    let router = build_router();
    let req = Request::builder()
        .uri("/api/v1/health")
        .body(Body::empty())
        .unwrap();

    let response = tower::ServiceExt::oneshot(router, req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
```

> [!NOTE]
> **Checkpoint:** `cargo test -p lab13-solution --test tower_bridge` passes and confirms the `/api/v1/health` path works through the Tower bridge.

## Stretch goals
- Add a second `/api/v1/vessels` endpoint that returns a tiny JSON list from a static `Vec`.
- Wrap the API with a timeout or request-id layer and confirm the middleware shows up in the response headers or logs.
- Explore whether the Topcoat app can embed a `TowerService` fallback for the remaining unmatched routes in a later project.

## Troubleshooting
- **`TowerRoute` does not match `/api/v1` exactly.** Register the catch-all form `"/api/v1/{*rest}"` and, if you also serve the bare prefix, add a second route for `"/api/v1"`.
- **JSON response is empty or the `Content-Type` is wrong.** Return `axum::Json(...)` for the endpoint and make sure the route is nested inside the mounted Axum router.
- **Middleware does not wrap the API path.** Use `.at("/api/v1")` on each `TowerLayer` call; otherwise the layer wraps every route in the router.
- **Compilation fails on the tower bridge.** Enable the `tower` feature on `topcoat` and keep the `axum` and `tower-http` versions consistent with the workspace pins.

## What's next
Lab 13 is the bridge pattern: Topcoat remains the application shell and page renderer, while Axum/Tower fills the gap when a JSON API or mature HTTP middleware stack is the better fit.
