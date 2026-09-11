# `Cx` API summary

`Cx` is Topcoat's handle for the current request. Pass `&Cx` to small request functions, or declare it
on a page, layout, component, route, shard, or procedure that needs request-scoped information.

The helpers in this page are the ones used across Labs 05–08. They follow Topcoat v0.7.0's [app
context](https://raw.githubusercontent.com/tokio-rs/topcoat/v0.7.0/crates/topcoat/docs/app_context.md)
and [functions, not
middlewares](https://raw.githubusercontent.com/tokio-rs/topcoat/v0.7.0/crates/topcoat/docs/functions_not_middlewares.md)
guides.

> [!NOTE]
> `Cx` is request-scoped. Values registered as app context outlive requests, but the `&Cx` used to
> reach them belongs only to the current request.

## Request

Use request helpers to read the URI and values produced by route matching.

| API | Result | Used for |
|---|---|---|
| `request::uri(cx)` | `&Uri` | Read the current path or raw query string. |
| `uri(cx).path()` | `&str` | Highlight navigation for the current route. |
| `uri(cx).query()` | `Option<&str>` | Read the raw query string before parsing application values. |
| `path_param::<Id>(cx)` | The declared parameter type | Read a dynamic module-routing segment declared with `path_param!(id)`. |

Lab 05 reads a typed path parameter and returns a not-found error before composing the detail page:

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

Enable cookie handling on the router with `.cookies()`. Then call `cookies(cx)` to obtain a cookie
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
cookies should remain secure. Keep signing and encryption keys in app context or another server-side
secret source; never expose them to the browser.

## Session

Enable sessions with `.sessions(SessionConfig)`. The session helpers use `Cx` to read or update the
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
invalid session as signed out, then looks up the email in app context. `require_auth` turns that
optional identity into a redirect for private behaviour.

Labs 07–08 carry the same server-side session pattern forward. Signals and `$(...)` do not move a
session into JavaScript. A shard or procedure receives a fresh request `Cx` and must perform its own
session and authorization checks.

## App context

App context holds values that are shared by all requests handled by a router: a database pool,
configuration, an HTTP client, or the Lab 05 state used to count loads.

| API | Result | Notes |
|---|---|---|
| `.app_context(value)` | Router builder | Register one value under its concrete Rust type. |
| `app_context::<T>(cx)` | `&T` | Borrow the registered value; panics when that exact type was not registered. |
| `try_app_context::<T>(cx)` | `Option<&T>` | Use only when the registration is intentionally optional. |
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

App context is keyed by `TypeId`. Registering two values of the same concrete type or requesting an
unregistered type is a configuration error. Use newtypes when you need two values with the same
underlying type.

Memoization has a different lifetime. It belongs to the current request, so one visitor never
receives another visitor's cached page data. Components may call the same memoized helper without
threading the value through every intermediate component.

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

## Quick choice

- Need the current path or a route value? Use a **request helper**.
- Need browser state carried by HTTP? Use **cookies**.
- Need a revocable login lifecycle? Use **session helpers** plus your server-side store.
- Need a long-lived service or configuration value? Use **app context**.
- Need to stop rendering or redirect? Return a **router error** through `topcoat::Result`.

Prefer a small `async fn f(cx: &Cx)` when more than one component needs the same request-scoped
meaning. That keeps the dependency visible where it is used and lets `#[memoize]` remove repeated
expensive work.

**See also:** [Lab 05 — Cx, app context, and memoization](../02-labs/lab-05.md),
[Lab 06 — Cookies, sessions, and mail](../02-labs/lab-06.md),
[Lab 07 — Signals and expressions](../02-labs/lab-07.md),
[Lab 08 — Shards, procedures, and streaming](../02-labs/lab-08.md),
[Request lifecycle](../01-concepts/request-lifecycle.md), and
[Locality of behaviour vs middleware](../01-concepts/functions-not-middleware.md).
