# `Cx` API summary

`Cx` is Topcoat's handle for the current request. Every page, route, layout, layer, component, shard,
procedure, and memoized function takes `cx: &Cx` as its first parameter, even when it does not use it.
Pass the same `cx` explicitly to ordinary helper functions.

The helpers in this page are the ones used across Labs 05–08. They follow Topcoat v0.12.1's
[context guide](https://raw.githubusercontent.com/tokio-rs/topcoat/v0.12.1/docs/context.md)
and [router guide](https://raw.githubusercontent.com/tokio-rs/topcoat/v0.12.1/docs/router.md).

> [!NOTE]
> `Cx` is request-scoped. Values added to the router with `.with(value)` outlive requests, but the
> `&Cx` used to reach them belongs only to the current request.

## Signals

Signal creation is request-scoped. In a page, component, layout, or shard, call
`signal(cx, || initial_value)` and pass a signal to a child as `&Signal<T>` when the child needs it:

The initial value is rendered on the server, then the browser owns the signal. A plain Rust `.get()`
or `.read()` is a tracked server read; use `.get_untracked()` or `.read_untracked()` when that read
must not make the body depend on the signal. Validate every signal value read on the server as user
input. See [How `$(...)` reaches the browser](../01-concepts/dual-expressions.md) for the dual
expression model.

*Guide basis: the v0.12.1 runtime guide's [Signals](https://raw.githubusercontent.com/tokio-rs/topcoat/v0.12.1/docs/runtime.md#signals)
and [Reading signals on the server](https://raw.githubusercontent.com/tokio-rs/topcoat/v0.12.1/docs/runtime.md#reading-signals-on-the-server) sections.*

## Request

Use request helpers to read the URI and values produced by route matching.

| API | Result | Used for |
|---|---|---|
| `request::uri(cx)` | `&Uri` | Read the current path or raw query string. |
| `uri(cx).path()` | `&str` | Highlight navigation for the current route. |
| `uri(cx).query()` | `Option<&str>` | Read the raw query string before parsing application values. |
| `id(cx)` | `&str`, or the declared type as a `Result` | Read a dynamic segment through the accessor that `module_param!(id)` or `path_param!(id: u64 else bad_request)` generates. |

Lab 05 reads the `id` path parameter through its generated accessor and returns a not-found error before composing the detail page:

```rust
{{#include ../../../labs/lab-05-cx-memoize/solution/src/app/_marketing/berths/id.rs:request-values}}
```

`uri(cx).query()` gives you the raw query text. Labs 06–08 use it for deliberately small examples
such as `?email=` and `?token=`. For real input, parse all pairs, percent-decode values, and validate
them before use.

Request helpers are ordinary functions. `Cx` itself is not a bag that you mutate and pass down.
Create focused helpers that accept `&Cx` when several components need the same interpretation of a
request value.

## Cookies

Enable cookie handling on the router with `.layer(CookieLayer::new())`. Then call `cookies(cx)` to obtain a cookie
view for the current request and response.

| API | Purpose |
|---|---|
| `cookies(cx)` | Access cookies associated with this request. |
| `.private(&key)` | Encrypt and authenticate cookie values with the supplied key. |
| `.signed(&key)` | Authenticate cookie values with the supplied key. |
| `.default_secure(bool)` | Set the default `Secure` attribute for cookies written through the view. |
| `.default_http_only(bool)` | Set the default `HttpOnly` attribute. |
| `.default_same_site(...)` | Set the default `SameSite` policy. |
| `.default_path(...)` | Set the default cookie path. |
| `.add(cookie)` | Add a cookie to the response. |

Lab 06 centralizes those defaults in one request function:

```rust
{{#include ../../../labs/lab-06-auth-sessions-mail/solution/src/auth.rs:cookie-access}}
```

Callers use the configured view instead of repeating security attributes:

```rust
{{#include ../../../labs/lab-06-auth-sessions-mail/solution/src/auth.rs:cookie-write}}
```

The lab disables `Secure` only because its integration tests use plain HTTP. Production HTTPS
cookies should remain secure. Keep signing and encryption keys in context or another server-side
secret source; never expose them to the browser.

## Session

Sessions need the `CookieLayer`. Set their lifetime with `.with(SessionLifetime(duration))`; without it, sessions last 30 days. The session helpers use `Cx` to read or update the
session cookie while your application owns the record associated with the token hash.

| API | Result | Meaning |
|---|---|---|
| `session::token_hash(cx).await` | Current token hash, if present | Look up application session data without retaining the raw token. |
| `session::start(cx).await` | New session metadata | Start a session and arrange for its cookie to be written. |
| `session::stop(cx).await` | Revoked token hash, if present | Clear the browser session and identify the server record to remove. |
| `session::refresh(cx).await` | Refreshed session metadata, if present | Extend or update an existing session. |
| `session::rotate(cx).await` | Revoked hash plus replacement session, if present | Replace the token and migrate application state to the new record. |

Lab 06 keeps the browser cookie and its in-memory session store synchronized:

```rust
{{#include ../../../labs/lab-06-auth-sessions-mail/solution/src/auth.rs:session-lifecycle}}
```

`session::token_hash(cx)` is fallible and async. Lab 06's memoized `current_user` treats a missing or
invalid session as signed out, then looks up the email in the shared state read with `context(cx)`. `require_auth` turns that
optional identity into a redirect for private behaviour.

Labs 07–08 carry the same server-side session pattern forward. Signals and `$(...)` do not move a
session into JavaScript. A shard or procedure receives a fresh request `Cx` and must perform its own
session and authorization checks.

A connected render can outlive its initial authorization result.
Read the current server-side session record before each private emission.
Do not reuse a memoized initial user for this check or mutate response cookies after streaming starts.

*Guide basis: the v0.12.1 [live connection lifecycle](https://raw.githubusercontent.com/tokio-rs/topcoat/v0.12.1/docs/view/live.md#long-lived-connections).*

## Identity

Repeated stateful components use `#[key(item.id)]` on their template loop.
Their signals and live regions keep the same identity when the item moves.
An unkeyed loop is ambiguous when a component consumes that identity.
For ordinary Rust helpers, derive a distinct context with `cx.keyed(item.id)`.

*Guide basis: the v0.12.1 [`view!` identity guide](https://raw.githubusercontent.com/tokio-rs/topcoat/v0.12.1/docs/view/view.md#keys).*

## Shared context

Since Topcoat 0.12 there is one context. Values added to the router with `.with(value)` are shared by
all requests handled by that router: a database pool, configuration, an HTTP client, or the Lab 05
state used to count loads.

| API | Result | Notes |
|---|---|---|
| `.with(value)` | Router builder | Add one value under its concrete Rust type. |
| `context::<T>(cx)` | `&T` | Borrow the closest value of that type; panics when the context has none. |
| `try_context::<T>(cx)` | `Option<&T>` | Use only when the value is intentionally optional. |
| `#[memoize]` on an async `cx` function | Request-local cached result | Deduplicate identical work during one request. |
| `#[memoize(as_ref)]` | Borrowed cached result | Store an owned result while returning references to callers. |

Lab 05 registers its state once:

```rust
{{#include ../../../labs/lab-05-cx-memoize/solution/src/app.rs:app-context-router}}
```

The loader borrows it through `Cx` and memoizes the lookup:

```rust
{{#include ../../../labs/lab-05-cx-memoize/solution/src/shared.rs:memoized-loader}}
```

Context values are keyed by their Rust type. Adding a second value of the same type no longer panics:
the closest value wins, and a request value wins over a router value. Requesting a type the context
does not hold with `context` is a configuration error. Use newtypes when you need two values with the
same underlying type.

Memoization has a different lifetime. It belongs to the current request, so one visitor never
receives another visitor's cached page data. A memoized call is reused only while the context values
it read still match. Callers pass `cx` explicitly. Components may call the same memoized helper
without threading the value through every intermediate component.

*Guide basis: the v0.12.1 [context guide](https://raw.githubusercontent.com/tokio-rs/topcoat/v0.12.1/docs/context.md)
and [memoize guide](https://raw.githubusercontent.com/tokio-rs/topcoat/v0.12.1/docs/context/memoize.md).*

## Errors and responses

Return `topcoat::Result<T>` when a request function or handler may stop normal rendering. The `?`
operator propagates router errors through components, pages, routes, shards, and procedures.

| API or pattern | HTTP behaviour in the labs | Use for |
|---|---|---|
| `error::not_found()` | `404 Not Found` | A route parameter or lookup has no resource. |
| `error::redirect(location)` | `307 Temporary Redirect` | Send an unauthenticated GET to login while preserving redirect semantics. |
| `error::see_other(location)` | `303 See Other` | Redirect after a state-changing request so the next request is a GET. |
| `Err(error.into())` | The converted router response | Return early from a helper or component. |
| `operation.await?` | Propagates the first error | Compose request functions without manually building responses. |

Lab 06's guard returns the identity on success and a login redirect otherwise:

```rust
{{#include ../../../labs/lab-06-auth-sessions-mail/solution/src/auth.rs:require-auth}}
```

Keep guards local to the behaviour they protect. Lab 08 repeats `require_auth(cx).await?` inside a
procedure because the procedure endpoint runs independently of the page and layouts that exposed
it:

```rust
{{#include ../../../labs/lab-08-shards-procedures-streaming/solution/src/app/_marketing.rs:complete-work-order-procedure}}
```

Do not treat a redirect or status as authorization by itself. Validate caller-controlled path,
query, shard, and procedure arguments, then check the current identity before reading or changing
private data.

Topcoat 0.12.1's `Error` is cheaply cloneable and does not wrap `anyhow::Error`.
Use `Error::from_anyhow` with the optional `anyhow` feature only when that conversion is needed.
Owned or mutable downcasts require unique ownership; use `downcast_cloned` when sharing is expected.

*Guide basis: the v0.12.1 [error source](https://raw.githubusercontent.com/tokio-rs/topcoat/v0.12.1/crates/topcoat-core/src/error.rs).*

## Quick choice

- Need the current path or a route value? Use a **request helper**.
- Need browser state carried by HTTP? Use **cookies**.
- Need a revocable login lifecycle? Use **session helpers** plus your server-side store.
- Need a long-lived service or configuration value? Use **shared context** added with `.with(value)`.
- Need to stop rendering or redirect? Return a **router error** through `topcoat::Result`.

Prefer a small `async fn f(cx: &Cx)` when more than one component needs the same request-scoped
meaning. That keeps the dependency visible where it is used and lets `#[memoize]` remove repeated
expensive work.

**See also:** [Lab 05 — Cx, context, and memoization](../02-labs/lab-05.md),
[Lab 06 — Cookies, sessions, and mail](../02-labs/lab-06.md),
[Lab 07 — Signals and expressions](../02-labs/lab-07.md),
[Lab 08 — Shards, procedures, and streaming](../02-labs/lab-08.md),
[Request lifecycle](../01-concepts/request-lifecycle.md), and
[Locality of behaviour vs middleware](../01-concepts/functions-not-middleware.md).
