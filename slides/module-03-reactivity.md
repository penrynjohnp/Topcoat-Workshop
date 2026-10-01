---
marp: true
theme: default
paginate: true
header: 'Topcoat Workshop · Module 3'
footer: 'Reactivity'
---

# Module 3 — Reactivity
## Signals, shards, procedures, streaming and server push

Ask one question first: does the browser already have the answer?

---

# Choose by ownership

| Mechanism | Update owner | Best for |
|---|---|---|
| `signal` + `$(...)` | Browser | toggles and filters over rendered data |
| `#[shard]` | Server | new markup from server data |
| `#[procedure]` | Server | an action or value without a markup swap |
| `live!` / `suspense` | HTTP or a connected WebSocket | progressive delivery and server updates |
| htmx / Alpine | Browser markup | pragmatic enhancement and declarative triggers |
| Datastar | Server stream | server-pushed patches over time |

They can coexist on one page.

---

# Signals stay client-only

The server evaluates `$(...)` once for the initial render.

The browser runtime re-runs the supported expression when its signals change.

Use signals for:

- a collapsible panel;
- a local filter over rows already rendered;
- a class or hidden binding;
- input state that does not need fresh server data.

No network request is needed.

---

# The dual-expression boundary

```mermaid
flowchart LR
  Rust[shared Rust expression] --> Server[initial server evaluation]
  Rust --> JS[cross-compiled browser evaluation]
  Server --> HTML[initial HTML]
  JS --> DOM[local DOM update]
```

The expression vocabulary is deliberately smaller than all of Rust. The compiler tells you when you leave it.

Topcoat 0.9.0 supports all Rust integer types, vectors, arrays and slices.
Unsuffixed integers are `usize`; database IDs remain `u64` without an `f64` conversion.
Composed expressions retain signal dependencies. Captured collections remain render-time snapshots.

---

# Shards replace one region

1. The first render runs the shard inline.
2. A signal changes an argument.
3. The runtime sends the current arguments to the shard endpoint.
4. The server validates, authorizes, and queries.
5. Returned HTML replaces only the shard region.

Read the emitted endpoint URL rather than constructing an internal prefix.
Use HTML IDs for moved DOM nodes and `#[key(item.id)]` for repeated stateful component identity.

---

# Procedures call the server

Use a procedure when the browser needs a server action or value:

- mark a work order complete;
- save a draft;
- compute a value unavailable in the expression subset.

A procedure is a public HTTP endpoint. Validate arguments and repeat authorization inside it. Page and layout guards do not run first.

---

# Finite live regions stream the first response

`live!` is not a later interaction request.

- the page starts streaming;
- a slow region emits progress or content;
- default `Stream` suspense supplies a fallback only when the child is not ready;
- `Wait` makes initial child content available without JavaScript;
- `error_boundary` handles a late failure;
- authentication and headers happen before the first emission.

Use this when the page is slow, not when a signal changes later.

---

# Connected regions receive server push

1. Subscribe before reading the current snapshot.
2. Emit it and return when `connected(cx)` is false.
3. The browser opens the runtime WebSocket.
4. The connected render emits fresh state and waits for notifications.
5. Reconnect reloads state; disconnect drops subscriptions.

Check the stored session before each private emission.
The lab's notification channel is process-local, not durable multi-replica delivery.

---

# htmx, Alpine, and Datastar

- **htmx:** markup decides when to ask; debouncing and indicators are declarative.
- **Alpine AJAX:** choose it when the page already uses Alpine; the client owns the target.
- **Datastar:** the server sends event patches over a stream.

Topcoat's native runtime gives type-checked relationships. Integrations give pragmatic browser ergonomics.

**Checkpoint:** Lab 09 runs native and htmx search side by side.

**Next:** Toasty makes the server's data durable.
