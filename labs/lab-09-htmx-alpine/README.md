# Lab 09 — htmx and Alpine as the pragmatic path

**Time:** 45 min · **Module:** 3 · **Prerequisites:** Lab 08 complete (or copy `labs/lab-08-shards-procedures-streaming/solution`)

## What you'll learn
- Read htmx request headers on the server with `topcoat::htmx` accessors like `hx_request`
- Steer a swap from the server with `HxPushUrl`, `HxRetarget`, `HxReswap`, and `HxResponseTrigger`
- Choose between the native runtime, htmx, and Alpine AJAX for a given interaction, and say why — morph versus swap, where state lives, and what each guide does *not* promise

## Concepts (read first, 5 min)
Read the pinned [`topcoat::htmx` guide](https://github.com/tokio-rs/topcoat/blob/v0.8.0/crates/topcoat/docs/htmx.md) and the [Alpine AJAX guide](https://github.com/tokio-rs/topcoat/blob/v0.8.0/crates/topcoat/docs/alpine-ajax.md) beside it. Lab 08 built a live vessel search out of a signal and a `#[shard]`. The page creates the state with `signal(cx, String::new)` and hands the live handle to the shard as `Signal<String>` with `query: $(query)`. The shard's `.get()` is a tracked read, so a change re-runs *only that shard*, and the returned fragment is morphed into place rather than replacing the region — focus and the partly-typed query survive. Topcoat coalesces changes made in one tick and aborts stale in-flight requests so the latest result wins, but it does not debounce the pauses between keystrokes. Topcoat owns the endpoint, the request, and the update; the only thing you wrote was Rust.

htmx inverts that. The *client* decides what to request, when to request it, and where to put the answer, using `hx-*` attributes on the element, and the returned fragment is **swapped** into the target. Your server's job shrinks to returning an HTML fragment — and, when it wants to, overriding the client's decisions with `HX-*` response headers.

Neither is "the modern way." They are different distributions of the same responsibility, and the table further down lays the split out row by row. This lab builds the *same* search both ways, side by side in the same app, sharing the same Rust — so the only thing that differs is the transport.

`topcoat::htmx` is behind the non-default `htmx` feature. Turn it on per-crate, not in `[workspace.dependencies]`, so earlier labs keep building exactly as they were.

## Steps

### Step 1 — Turn on the htmx feature
The workspace pins the version once; this lab adds a feature on top of it. Never pin a version inside a lab crate.

```toml
{{#include solution/Cargo.toml:9:9}}
```

The first build downloads `topcoat-htmx`.
> [!NOTE]
> **Checkpoint:** `cargo build -p lab09-solution` succeeds and `cargo tree -p lab09-solution | grep topcoat-htmx` prints `topcoat-htmx v0.8.0`.

### Step 2 — Extract the markup both transports will share
This is the whole point of the lab, so do it before writing any htmx. The vessel filtering and the results list move out of the shard body into `shared.rs`, where a plain function holds the domain logic and a `#[component]` holds the markup:

```rust
{{#include solution/src/shared.rs:shared-search}}
```

`search_vessels` returns `Result<_, QueryTooLong>` rather than rendering an error itself. That matters: the *domain* decides the query is invalid, but each *transport* decides where the message goes — inline for the shard, retargeted into an alert region for htmx.

The Lab 08 shard now shrinks to a length check and a call. It still takes the page's signal as `Signal<String>` and reads it with `.get()`, so the native side keeps its tracked re-run and its morph; only the markup moved:

```rust
{{#include solution/src/app/_marketing/vessels.rs:vessel-results-shard}}
```

The `<li>` ids Lab 08 added for morphing move into `shared.rs` with the rest of the markup. Both transports get them, which costs htmx nothing and keeps the two fragments identical:

```rust
{{#include solution/src/shared.rs:vessel-result-ids}}
```

> [!NOTE]
> **Checkpoint:** `cargo test -p lab09-solution --test server_runtime` still passes. Lab 08's shard test still asserts the same fragment, which proves the refactor did not alter a byte of the rendered markup.

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

`hx-trigger` and `hx-indicator` are htmx's own attributes, not part of `topcoat::htmx`. The pinned guide documents exactly three things — loading the script, the request-header accessors, and the response responders — and says nothing about trigger modifiers or indicator elements. So treat their behaviour as htmx's to define: read [htmx's attribute reference](https://htmx.org/reference/) for the version you pinned in Step 3 before relying on what `changed`, `delay:250ms`, or `hx-indicator` do here. The division the guide *does* draw is the useful one: Topcoat owns the headers on either side of the request, and leaves what the client does around them to htmx.
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
The suite asserts that neither page borrowed the other's mechanism — the native page drives its search through a shard marker and ships no `hx-get`; the htmx page does the reverse — and, the important one, that the two transports emit *identical* markup:

```rust
{{#include solution/tests/htmx.rs:coexistence-test}}
```

```rust
{{#include solution/tests/htmx.rs:fragment-equality-test}}
```

The equality test strips the runtime's `<!--::topcoat::...-->` comments before comparing. The shard's reply carries a `dep(...)` marker recording the tracked read of `query`; that is how the runtime knows what to re-run, and htmx has no equivalent because it has nothing to track. The *markup* either side of it is byte-for-byte the same.

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

Eight questions separate them. Every htmx cell comes from the pinned [`topcoat::htmx` guide](https://github.com/tokio-rs/topcoat/blob/v0.8.0/crates/topcoat/docs/htmx.md), every Alpine cell from the [Alpine AJAX guide](https://github.com/tokio-rs/topcoat/blob/v0.8.0/crates/topcoat/docs/alpine-ajax.md), and every native cell from the runtime guide Lab 08 is built on. Where a guide is silent, the cell says so rather than guessing.

| | Native `#[shard]` | htmx | Alpine AJAX |
|---|---|---|---|
| **Where state lives** | in a page-owned signal: `signal(cx, String::new)`, handed to the shard as `Signal<String>` | in the DOM — `hx-*` attributes on the element — plus the URL the server pushes with `HxPushUrl` | in the DOM — `x-target` on a `<form>` or `<a>`, naming target `id`s |
| **What triggers a request** | a tracked read changes: the shard's `.get()` on its `Signal<T>` argument | an event on the element carrying `hx-get` / `hx-post` | submitting the `<form>` or following the `<a>` that carries `x-target` |
| **How the response is applied** | **morphed** into the shard's region | **swapped** into the target; the server may override with `HxReswap`, `HxRetarget`, `HxReselect` | **merged** into every named target, plus any `x-sync` region whose `id` appears in the response |
| **Focus and typing survive** | yes — that is what the morph buys you | not addressed by the guide; a swap replaces the target, so anything inside it is rebuilt | not addressed by the guide |
| **Coalescing and aborting** | same-tick changes coalesced, stale in-flight requests aborted, latest result wins | not addressed by the guide; whatever batching or cancelling happens is htmx's to define, in markup on the client | not addressed by the guide |
| **Debounce** | none — the one thing Lab 08 leaves you to solve | not addressed by the guide; pausing between keystrokes is configured on the client, with htmx's own attributes | not addressed by the guide |
| **JavaScript required** | the Topcoat runtime script; you write none | the htmx script, from a CDN or vendored with `asset!`; you write none | *two* scripts in order — the alpine-ajax plugin, then Alpine.js core, both `defer` — you write none |
| **Testability** | highest: the shard is a Rust function and its arguments are Rust types `rustc` checks | the endpoint is a Rust `#[route]` with typed header accessors and responders, but targets and URLs are strings the compiler never sees | same as htmx for the request side; there is **no response-header convention at all**, so there is nothing server-side to assert about the swap |

Read the table honestly in both directions. htmx moves the trigger decision into markup: a pause between keystrokes or a loading state is an attribute on the element rather than code you write, though the pinned guide describes neither, so htmx's own reference is where you confirm what those attributes do. The shard wins the state and testability rows: `vessel_results(query: $(query))` is checked by `rustc`, while `hx-target="#vessel-results-htmx"` is a string that silently does nothing if you rename the div. Neither is a default; the interaction decides.

Three things fell out of the table but still matter when you choose. **Overriding the swap from the server** is htmx-only: `HxRetarget`, `HxReswap`, `HxReselect`, and `HxRedirect` let the server overrule the client's decision, as Step 5 does for a validation failure. **History and the URL** are htmx's too — `HxPushUrl` and `HxReplaceUrl` — and a shard swap leaves the address bar alone. **Updating several regions at once** is one shard per region natively, and several `id`s in one `x-target` (or an `x-sync` region) in Alpine; htmx has a client-side out-of-band mechanism of its own, which the pinned guide does not cover — what the guide gives you server-side is `HxRetarget` and `HxReselect` for steering a single swap.

The accessors available on each side, all from `topcoat::htmx`:

- **Request:** `hx_request`, `hx_boosted`, `hx_history_restore_request` return `bool`; `hx_current_url`, `hx_prompt`, `hx_target`, `hx_trigger`, `hx_trigger_name` return `Option<&str>`.
- **Response:** `HxPushUrl`, `HxReplaceUrl`, `HxRedirect`, `HxLocation`, `HxRefresh`, `HxReswap`, `HxRetarget`, `HxReselect`, `HxResponseTrigger`. Each implements `IntoResponseParts`, so it goes *before* the body in the response tuple.

Alpine AJAX is the odd one out, and that is the point of including it: it has **no response-header convention**. Everything is decided in markup — `x-target` for the destination, `x-merge` for how the fragment is merged, `x-sync` for a region that refreshes whenever a response contains its `id`, and status modifiers like `x-target.422` or `x-target.error` for choosing a different target list on a failure. A server that wants to retarget or redirect a swap has nowhere to say so; all it can do is choose the status code and which fragments it renders. Topcoat's `alpine-ajax` feature reflects that asymmetry: it offers only `ajax_request`, `ajax_targets`, and `ajax_target`.

## Stretch goals
- Add a "N vessels" counter that listens for the `vessels:searched` event fired by `HxResponseTrigger::after_swap` and reads `event.detail`. No new endpoint required.
- Give the native side the pause-between-keystrokes htmx configures in markup, then decide whether it was worth the code using the raw! timer approach from Lab 08's stretch goals . Coalescing and stale-request aborting are already handled; you are only buying the pause.
- Build a third copy of the search with the `alpine-ajax` feature and `ajax_target(cx)`. Notice what you *cannot* express: try to move the validation message into the alert region from the server.
- Look up htmx's out-of-band swap in htmx's own reference and use it to update the nav's vessel count in the same response as the results fragment, then ask whether a second shard would have been simpler. Note where you are: the pinned `topcoat::htmx` guide does not cover that mechanism.
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
- **The equality test fails after an edit.** One transport stopped calling `shared::vessel_matches`. That test exists precisely to catch the two copies drifting apart. If the only difference is a `<!--::topcoat::...-->` comment, the markup is fine — that is runtime bookkeeping, and `without_runtime_markers` is meant to remove it.
- **Typing in the htmx box loses focus; typing in the native box does not.** Expected. htmx swaps the target, rebuilding whatever is inside it; the native side morphs, so focus and the partly-typed query survive. If the native side starts losing focus too, check that the input is *outside* the shard and that the result `<li>`s still carry stable ids.

## What's next
Lab 10 replaces every in-memory `Vec` behind these searches with real Toasty queries against SQLite — the same components, backed by data that survives a restart.
