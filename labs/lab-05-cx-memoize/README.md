# Lab 05 — Cx, app context, and memoization

**Time:** 60 min · **Module:** 2 · **Prerequisites:** Lab 04 complete (or copy `labs/lab-04-routing/solution`)

## What you'll learn
- Read request-scoped data from `Cx`
- Register long-lived app state with `app_context(cx)`
- Deduplicate repeated data loads with `#[memoize]`

## Concepts (read first, 5 min)
Read [Routing and the request](https://github.com/penrynjohnp/Topcoat-Workshop/blob/main/docs/src/01-concepts/routing-and-the-request.md) and the Topcoat app-context docs (`docs/app_context.md` in the crate). This lab builds on the same Slipway app from Lab 04. The key distinction is that `Cx` is request-scoped and short-lived, while app context is created once at router startup and shared across every request. A page can ask for the current path, the matched slug, and an app-scoped database or config object without threading those values through every component manually.

Memoization is the next step: when multiple components need the same data during one request, `#[memoize]` ensures the underlying async fetch runs once and the result is reused. This keeps request helpers composable and avoids hidden repeated work. In Topcoat, the cache is intentionally tied to a single request, so no value leaks across users or across unrelated requests.

## Steps
### Step 1 — Add long-lived app state
Start from the Lab 04 solution. Give the app a single `AppState` value that tracks a load counter and is shared for the lifetime of the router:

```rust
{{#include ../../../labs/lab-05-cx-memoize/solution/src/shared.rs:app-state}}
```

Register it once on the router so every handler sees the same state:

```rust
{{#include ../../../labs/lab-05-cx-memoize/solution/src/app.rs:app-context-router}}
```

The key idea is that app context is keyed by type, so `app_context::<AppState>(cx)` is the stable lookup point for shared values such as config, pools, or instrumentation.
> ✅ **Checkpoint:** the app still renders `/`, `/berths`, and `/berths/a1` successfully after the router starts with `AppState::new()`.

### Step 2 — Move berth loading behind `Cx`
Create a memoized request helper that reads the shared app state and increments a counter when a berth is loaded:

```rust
{{#include ../../../labs/lab-05-cx-memoize/solution/src/shared.rs:memoized-loader}}
```

This is where the request-scoped `Cx` becomes important: the function can read both the current request and the long-lived app state in one place. The first call in a request computes the value; later calls with the same args reuse the memoized result without re-running the load.
> ✅ **Checkpoint:** `cargo test -p lab05-solution --test pages` shows the count staying at `1` even though multiple parts of the page ask for the same berth.

### Step 3 — Render the same berth in three components
Split the detail page into three sibling components, each asking for the same slug:

```rust
{{#include ../../../labs/lab-05-cx-memoize/solution/src/app/_marketing/berths/id.rs:memoized-components}}
```

Once the page renders, every component reads the same `Berth` record from a single memoized fetch. Notice how the logic stays local to the code that needs it; no middleware or manual cache is required.
> ✅ **Checkpoint:** the page still renders the same A1 berth details, but the load count remains `1` because the memoized function is reused across all three components.

### Step 4 — Prove the memoization behavior in an integration test
Use the same in-process routing test style as Lab 04, but hold onto a shared `AppState` instance for the router. Then perform one request and assert the counter after render:

```rust
{{#include ../../../labs/lab-05-cx-memoize/solution/tests/pages.rs:memoize-integration-test}}
```

The test is the proof, not the behavior of a mock. It hits the real router, renders the real page, and confirms that the exact same `load_berth` work was deduplicated inside one request.
> ✅ **Checkpoint:** `cargo test -p lab05-solution --test pages` passes and the assertion reads `1`.

## Stretch goals
- Move the counter into a second app-context value and log the request path in the same loader.
- Add a second slug (`/berths/a2`) to the same page and confirm the memoization key is the slug + request context.
- Replace the plain `AtomicUsize` with a tiny `Db`-like app state and sketch how a real database pool would be accessed from the same pattern.

## Troubleshooting
- **`app_context` panics** → the value was not registered on the router with `.app_context(...)`. The type must match exactly.
- **The load counter is greater than 1** → the `#[memoize]` call is being re-entered with a different key or the same function is invoked outside the same request.
- **The page still compiles but rerenders too much** → all copies of the same berth request need to share the same `cx` and the same slug value.
- **The test cannot see the count** → hold a clone of `AppState` outside the router and read it after the request completes.

## What's next
Lab 06 moves from request-scoped helpers into cookies, signed sessions, and protecting pages with `require_auth(cx)` rather than router middleware.
