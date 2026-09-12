# Lab 10 — Toasty models and queries

**Time:** 90 min · **Module:** 4 · **Prerequisites:** Lab 09 complete (or copy `labs/lab-09-htmx-alpine/solution`)

## What you'll learn
- Model related data with Toasty and initialize a SQLite database
- Query and update persistent records directly inside server-rendered components and procedures
- Validate a POSTed form by hand and re-render preserved values with inline errors

## Concepts (read first, 5 min)
Read [What Toasty does](https://github.com/penrynjohnp/Topcoat-Workshop/blob/main/docs/src/01-concepts/toasty.md) and the pinned [Toasty 0.10.0 API documentation](https://docs.rs/toasty/0.10.0/toasty/). Toasty turns Rust model declarations into typed queries. A `Berth` owns related vessels and work orders, while each child stores the indexed foreign key. Components still render on the server: they clone the request's database handle, execute a query, and return HTML. No JSON API or browser-side data store appears between the page and SQLite.

This lab uses `push_schema()` only to bootstrap a fresh workshop database. Toasty 0.10.0 does not make that call idempotent for an existing SQLite file, so startup checks whether the file already exists before connecting. That is convenient for a prototype, not a migration strategy. Schema changes in a real service need ordered migrations and operational review.

## Steps
### Step 1 — Add Toasty and define relations
The workspace pins Toasty once, and both Lab 10 crates opt into that workspace dependency. Define the three domain models plus persisted session and magic-link records:

```rust
{{#include solution/src/models.rs:toasty-models}}
```

```rust
{{#include solution/src/models.rs:persisted-auth-models}}
```

`#[has_many]` creates relation accessors such as `berth.work_orders()`. `#[belongs_to]` connects those accessors to the child's indexed `berth_slug`; `Deferred<T>` lets a query decide when to load related rows.

> [!NOTE]
> **Checkpoint:** `cargo check -p lab10-starter` compiles all five models against Toasty 0.10.0.

### Step 2 — Open, push, and seed SQLite
Register every model discovered in this crate, push the schema only for a new database, and seed children through their parent relations:

```rust
{{#include solution/src/database.rs:database-setup}}
```

```rust
{{#include solution/src/database.rs:database-seed}}
```

The default URL is `sqlite:./slipway.db`; override it with `SLIPWAY_DATABASE_URL`. Visitors and Yard are real, unmanaged berth rows, so every vessel has a valid relation while `/berths` continues to list only managed berths.

> [!WARNING]
> `push_schema()` is a workshop bootstrap, not a migration runner. If you change a model during this lab, stop the server and delete `slipway.db` before restarting.

> [!NOTE]
> **Checkpoint:** run `cargo run -p lab10-solution`, stop it, run it again, and confirm the second start succeeds without reseeding duplicate rows.

### Step 3 — Query inside server components
Put the cloneable `toasty::Db` in `AppState`. Each request clones the handle because Toasty operations take `&mut Db`, then executes a typed query:

```rust
{{#include solution/src/shared.rs:toasty-queries}}
```

The same queried vessel collection feeds both the native shard and htmx route, preserving Lab 09's byte-for-byte fragment contract. The berth detail query eagerly includes vessels, so the three memoized components share one loaded model graph.

```rust
{{#include solution/tests/persistence.rs:relation-query-test}}
```

> [!NOTE]
> **Checkpoint:** `cargo test -p lab10-solution --test pages --test htmx --test persistence seeded_relations_are_queryable` passes.

### Step 4 — Persist sessions and mutations
The app now stores session token hashes and expiry timestamps in SQLite instead of process memory:

```rust
{{#include solution/src/auth.rs:database-session-store}}
```

Magic links are persistent too, and their tokens are cryptographically random:

```rust
{{#include solution/src/auth.rs:database-magic-links}}
```

The existing `#[procedure]` now loads and updates a `WorkOrder`. Topcoat 0.8 runtime numbers use `f64`, so the server validates that surrogate before converting it to Toasty's `u64` key:

```rust
{{#include solution/src/app/_marketing.rs:complete-work-order-procedure}}
```

Keep the cookie key stable when testing a restart; the database can preserve a session record, but a newly generated encryption key cannot decrypt an old cookie.

> [!NOTE]
> **Checkpoint:** `cargo test -p lab10-solution --test persistence data_and_sessions_survive_reopening_a_file_database` proves both domain data and an authenticated session survive reopening the SQLite file.

### Step 5 — Build and validate the create form
GET and POST pages share one server-rendered form. The POST trims the title, applies the length limit, confirms the submitted berth is a managed database row, and creates through `berth.work_orders()` only when both fields are valid:

```rust
{{#include solution/src/app/_marketing/dashboard/work_orders/new.rs:work-order-form}}
```

```rust
{{#include solution/src/app/_marketing/dashboard/work_orders/new.rs:manual-form-validation}}
```

Topcoat validation helpers are still on the roadmap, so this lab deliberately keeps validation explicit. Topcoat 0.8.0 now decodes an empty form or query value as `None` for an `Option<T>`. These two required fields deliberately remain `String`, so an empty submission reaches the explicit `is_empty()` and managed-berth checks below. Invalid input returns HTTP 200, preserves submitted values, and renders both messages; valid input returns 303 with `Location: /dashboard`.

```rust
{{#include solution/tests/persistence.rs:form-validation-test}}
```

> [!NOTE]
> **Checkpoint:** log in, open `/dashboard/work-orders/new`, submit an empty or overlong title, and confirm the page keeps your input. Then run `cargo test -p lab10-solution --test persistence`.

### Step 6 — Verify restart persistence
The final integration test creates a record, authenticates, drops the first app state, reconnects to the same SQLite file, and checks both the row and protected page:

```rust
{{#include solution/tests/persistence.rs:restart-persistence-test}}
```

> [!NOTE]
> **Checkpoint:** `bash scripts/check-lab.sh 10` builds both crates and runs the solution suite.

## Stretch goals
- Replace the fresh-file `push_schema()` bootstrap with ordered migrations; include a migration that adds a nullable work-order description.
- Add an edit-work-order form using `toasty::update!`, with the same preserved-value validation contract.
- Add pagination to vessel search and compare where cursor state belongs for the shard and htmx transports.
- Read a stable cookie master key from deployment configuration, then rotate it with an overlap period.

## Troubleshooting
- **Startup says a table already exists.** `push_schema()` ran against an existing file. Delete `slipway.db` while learning, or implement migrations before changing a real schema.
- **The database looks empty after changing `SLIPWAY_DATABASE_URL`.** You opened a different SQLite file. Print or inspect the exact URL used by the process.
- **A berth relation is empty even though the child row exists.** The query did not call `.include(...)`, or the child has the wrong `berth_slug`.
- **POST returns 415 or fails to extract `Form`.** Send `Content-Type: application/x-www-form-urlencoded`.
- **A session row exists but the old browser cookie no longer works.** The cookie encryption key changed across the restart. Persist that key in deployment configuration; never hard-code it or commit it.
- **The procedure rejects a numeric ID.** Topcoat 0.8 transports runtime numbers as `f64`; accept the surrogate, validate that it is a non-negative integer, then convert it to the database key.

## What's next
Lab 11 keeps this persistent server-rendered app and adds bundled assets, fonts, icons, Tailwind, and owned Topcoat UI components.
