# Lab 09 — htmx and Alpine as the pragmatic path

**Time:** 45 min · **Module:** 3 · **Prerequisites:** Lab 08 complete (or copy `labs/lab-08-shards-procedures-streaming/solution`)

## What you'll learn
- Read htmx request headers on the server with `topcoat::htmx` accessors like `hx_request`
- Steer a swap from the server with `HxPushUrl`, `HxRetarget`, `HxReswap`, and `HxResponseTrigger`
- Choose between the native runtime, htmx, and Alpine AJAX for a given interaction, and say why

## Concepts (read first, 5 min)
Read the pinned [`topcoat::htmx` guide](https://docs.rs/topcoat/0.7.0/topcoat/htmx/index.html). Lab 08 built a live vessel search out of a `signal` and a `#[shard]`: Topcoat owned the endpoint, the request, and the swap, and the only thing you wrote was Rust. htmx inverts that. The *client* decides what to request, when to request it, and where to put the answer, using `hx-*` attributes on the element. Your server's job shrinks to returning an HTML fragment — and, when it wants to, overriding the client's decisions with response headers.

Neither is "the modern way." They are different distributions of the same responsibility. The runtime gives you type-checked arguments, no client library, and no element ids in strings, but in Topcoat 0.7 it does not debounce and it has no built-in loading indicator. htmx gives you `delay:250ms` and `hx-indicator` for free, works without a reactive scope, degrades to a plain form, and costs you a `~14 kB` script plus a set of stringly-typed selectors the compiler cannot check. This lab builds the *same* search both ways, side by side in the same app, sharing the same Rust — so the only thing that differs is the transport.

`topcoat::htmx` is behind the non-default `htmx` feature. Turn it on per-crate, not in `[workspace.dependencies]`, so earlier labs keep building exactly as they were.

## Steps

### Step 1 — Turn on the htmx feature
The workspace pins the version once; this lab adds a feature on top of it. Never pin a version inside a lab crate.

```toml
{{#include solution/Cargo.toml:9:9}}
```

The first build downloads `topcoat-htmx`.
> [!NOTE]
> **Checkpoint:** `cargo build -p lab09-solution` succeeds and `cargo tree -p lab09-solution | grep topcoat-htmx` prints `topcoat-htmx v0.7.0`.

### Step 2 — Extract the markup both transports will share
This is the whole point of the lab, so do it before writing any htmx. The vessel filtering and the results list move out of the shard body into `shared.rs`, where a plain function holds the domain logic and a `#[component]` holds the markup:

```rust
{{#include solution/src/shared.rs:shared-search}}
```

`search_vessels` returns `Result<_, QueryTooLong>` rather than rendering an error itself. That matters: the *domain* decides the query is invalid, but each *transport* decides where the message goes — inline for the shard, retargeted into an alert region for htmx.

The Lab 08 shard now shrinks to a length check and a call:

```rust
{{#include solution/src/app/_marketing/vessels.rs:vessel-results-shard}}
```

> [!NOTE]
> **Checkpoint:** `cargo test -p lab09-solution --test server_runtime` still passes. Lab 08's shard test is unchanged, which proves the refactor did not alter a byte of the rendered fragment.

### Step 3 — Load htmx from a pinned URL
htmx reads attributes at load time, so it goes in the root layout beside the Topcoat runtime script:

```rust
{{#include solution/src/app/_marketing.rs:htmx-script}}
```

Pin the version. `hx-trigger` and swap semantics changed between htmx 1 and 2, and a floating `@2` tag would let a future minor release edit your app's behaviour without a commit. Lab 11 replaces this CDN URL with `asset!` so the script is content-hashed and served from your own origin.
> [!NOTE]
> **Checkpoint:** load any page and confirm the `<script>` tag appears in view-source, and that `htmx` is defined in the browser console.

### Step 4 — Build the htmx twin of the search page
Two things make this page more than a demo. First, it is wrapped in a real `<form method="get">`, so it works with JavaScript disabled. Second, it renders the first set of results **on the server** from `?q=`:

```rust
{{#include solution/src/app/_marketing/vessels/htmx.rs:htmx-search-page}}
```

Query parsing is typed and memoized rather than hand-split:

```rust
{{#include solution/src/app/_marketing/vessels/htmx.rs:htmx-query-params}}
```

`hx-trigger="input changed delay:250ms, search"` is the debounce that was a *stretch goal* in Lab 08 — declarative, and free. `changed` also suppresses requests when the value did not actually change (arrow keys, modifier keys). `hx-indicator` is the loading state the native runtime does not yet give you.
> [!NOTE]
> **Checkpoint:** `curl -s 'http://localhost:3000/vessels/htmx?q=Lady' | grep 'Lady Jane'` finds a match. The page searched with no JavaScript involved at all.

### Step 5 — Answer with a fragment, and steer the swap
The fragment endpoint is a `#[route(GET)]`, not a `#[page]`. Layouts wrap pages, so a route already returns bare markup — there is no shell to strip:

```rust
{{#include solution/src/app/_marketing/vessels/htmx/results.rs:htmx-fragment-route}}
```

Three separate lessons are packed in here:

1. **`hx_request(cx)` is a guard, not a security boundary.** Anyone can send `HX-Request: true` with `curl`. It is there so a *human* who pastes the fragment URL into the address bar gets a real page instead of naked markup. The actual protection is the length check — and if this data were private, it would need the same `require_auth(cx)` a shard endpoint needs. The transport changed; the threat model did not.
2. **`HxRetarget` + `HxReswap` let the server overrule the client.** The input says `hx-target="#vessel-results-htmx"`. For a validation failure the server says "no, put this in `#vessel-alert`, and use `innerHTML`" — without the client knowing that failures are even possible.
3. **`HxPushUrl` and the server-rendered first paint are a pair.** Pushing `?q=Lady` into the address bar is only honest because Step 4's page reads `?q=` and renders results itself. If it did not, every shared or reloaded link would show an empty list.

`HxResponseTrigger::after_swap` fires a DOM event once the swap settles, with the match count as JSON data — the hook for a "3 results" counter elsewhere on the page.

> [!NOTE]
> **Checkpoint:** run the two commands below. The first must show `HX-Push-Url` and `HX-Trigger-After-Swap`; the second must show a `307` to `/vessels/htmx`.
> ```
> curl -si -H 'HX-Request: true' 'http://localhost:3000/vessels/htmx/results?q=Lady' | head -20
> curl -si 'http://localhost:3000/vessels/htmx/results?q=Lady' | head -5
> ```

### Step 6 — Prove both implementations coexist
The suite asserts that neither page borrowed the other's mechanism, and — the important one — that the two transports emit *identical* HTML:

```rust
{{#include solution/tests/htmx.rs:coexistence-test}}
```

```rust
{{#include solution/tests/htmx.rs:fragment-equality-test}}
```

The remaining tests cover the guard, both response-header paths, and no-JS operation:

```rust
{{#include solution/tests/htmx.rs:hx-request-guard-test}}
```

```rust
{{#include solution/tests/htmx.rs:response-header-test}}
```

```rust
{{#include solution/tests/htmx.rs:no-js-test}}
```

> [!NOTE]
> **Checkpoint:** `cargo test -p lab09-solution` reports 22 passing tests, and `topcoat dev` serves a working search at both `/vessels` and `/vessels/htmx`.

## Choosing between them

| | Native `#[shard]` | htmx | Alpine AJAX |
|---|---|---|---|
| **Client payload** | none beyond the Topcoat runtime | `~14 kB` htmx | `~15 kB` Alpine + `~4 kB` alpine-ajax |
| **Where the request is decided** | Rust — a changed runtime-expression argument | markup — `hx-get` / `hx-trigger` | markup — a form or link with `x-target` |
| **Trigger control** | argument changes only | rich DSL: events, `delay`, `throttle`, `changed`, `from:`, `revealed` | the element's natural event |
| **Debouncing** | not built in (0.7 coalesces per tick and aborts stale requests) | `delay:250ms` | none built in; use Alpine's `.debounce` modifier |
| **Loading indicator** | write it yourself with a signal | `hx-indicator` + the `htmx-request` class | `x-target` adds `aria-busy` |
| **Works with JS off** | no — the region needs the runtime | yes, if you wrap it in a real form | yes, it enhances forms and links |
| **Where state lives** | server, in the shard's arguments | the DOM, plus the URL you push | the DOM, plus Alpine's `x-data` |
| **Type safety** | full: arguments are Rust types the compiler checks | none at the attribute boundary; ids and URLs are strings | none; targets are id strings |
| **Server can override the swap** | n/a — Topcoat owns the swap | yes: `HxRetarget`, `HxReswap`, `HxReselect`, `HxRedirect` | **no response-header convention at all** |
| **History / URL** | unchanged by a shard swap | `hx-push-url` or `HxPushUrl` | manual |
| **Multi-region update** | one shard per region | `hx-swap-oob` | `x-target` accepts several ids; `x-sync` regions update everywhere |
| **Reach for this when…** | the interaction is server data and you want the compiler to check it | you need debouncing, indicators, or progressive enhancement *today* | the page already uses Alpine for local UI state and you want AJAX in the same idiom |

Read the table honestly in both directions. htmx wins Step 4 outright: a 250 ms debounce and a loading indicator would each be hand-written work in the native version. The shard wins Step 2: `vessel_results(query: $(query.get()))` is checked by `rustc`, while `hx-target="#vessel-results-htmx"` is a string that silently does nothing if you rename the div. Neither is a default; the interaction decides.

The accessors available on each side, all from `topcoat::htmx`:

- **Request:** `hx_request`, `hx_boosted`, `hx_history_restore_request` return `bool`; `hx_current_url`, `hx_prompt`, `hx_target`, `hx_trigger`, `hx_trigger_name` return `Option<&str>`.
- **Response:** `HxPushUrl`, `HxReplaceUrl`, `HxRedirect`, `HxRefresh`, `HxReswap`, `HxRetarget`, `HxReselect`, `HxResponseTrigger`. Each implements `IntoResponseParts`, so it goes *before* the body in the response tuple.

Alpine AJAX is the odd one out, and that is the point of including it: it has **no response-header convention**. Everything is decided in markup — `x-target` for the destination, `x-target.away` to update a region elsewhere, `x-sync` for regions that should refresh on every request, `x-merge` for the swap style. A server that wants to redirect a swap has nowhere to say so. Topcoat's `alpine-ajax` feature reflects that asymmetry: it offers only `ajax_request`, `ajax_targets`, and `ajax_target`.

## Stretch goals
- Add a "N vessels" counter that listens for the `vessels:searched` event fired by `HxResponseTrigger::after_swap` and reads `event.detail`. No new endpoint required.
- Build a third copy of the search with the `alpine-ajax` feature and `ajax_target(cx)`. Notice what you *cannot* express: try to move the validation message into the alert region from the server.
- Use `hx-swap-oob` to update the nav's vessel count in the same response as the results fragment, then ask whether a second shard would have been simpler.
- Delete the `hx_request` guard and open `/vessels/htmx/results?q=Lady` in a browser. Decide whether the raw fragment is a bug, an information leak, or neither.
- Replace the hand-rolled `urlencode` helper with a crate, and add a test for a query containing `&` and a space.

## Troubleshooting
- **The htmx page renders but typing does nothing.** htmx is not loaded. Check for the `<script>` in view-source (Step 3) and for a CSP or offline environment blocking `cdn.jsdelivr.net`. Vendor it with `asset!` if your network blocks CDNs.
- **`cannot find module htmx` / `unresolved import topcoat::htmx`.** The `htmx` feature is not enabled. It is not in `default`; see Step 1. `topcoat.workspace = true` alone is not enough.
- **The fragment endpoint 404s.** `#[route(GET)]` derives its path from the module path, so `src/app/_marketing/vessels/htmx/results.rs` serves `/vessels/htmx/results`. Renaming the file changes the URL, and `hx-get` will not follow it.
- **The whole page appears inside the results div.** You pointed `hx-get` at the `#[page]` rather than the `#[route]`. Pages are wrapped by layouts; routes are not.
- **`HX-Push-Url` appears but the reloaded link shows nothing.** The page is not reading `?q=` and server-rendering. See Step 4 — a pushed URL is a promise the page has to keep.
- **Response headers are missing.** The responders must precede the body in the tuple: `(HxPushUrl(url), trigger, body)`. A responder placed last is treated as the body.
- **`expected `=`` on a `<script>` tag.** `view!` rejects bare boolean attributes; write `defer="defer"`, not `defer`.
- **The equality test fails after an edit.** One transport stopped calling `shared::vessel_matches`. That test exists precisely to catch the two copies drifting apart.

## What's next
Lab 10 replaces every in-memory `Vec` behind these searches with real Toasty queries against SQLite — the same components, backed by data that survives a restart.
