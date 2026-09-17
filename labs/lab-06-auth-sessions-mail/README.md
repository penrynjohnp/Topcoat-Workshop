# Lab 06 — Cookies, sessions, and functions—not middleware

**Time:** 75 min · **Module:** 2 · **Prerequisites:** Lab 05 complete (or copy `labs/lab-05-cx-memoize/solution`)

## What you'll learn
- Read and write secure cookies from `Cx`
- Start, refresh, rotate, and stop sessions with application-owned storage
- Protect reusable components with `require_auth(cx)` and send a file-backed magic link

## Concepts (read first, 5 min)
Read [Locality of behaviour vs middleware](https://github.com/penrynjohnp/Topcoat-Workshop/blob/main/docs/src/01-concepts/functions-not-middleware.md) and the pinned Topcoat session and cookie guides. Topcoat supplies the token and cookie mechanics, but your application owns the session record: store the token hash beside the user and expiry, never the raw token. The session API deliberately exposes `start`, `token_hash`, `refresh`, `rotate`, and `stop` so login, sliding expiry, privilege changes, and logout remain ordinary request-scoped functions.

Authentication is a rendering concern as well as a routing concern. A page or component calls `require_auth(cx).await?` at the point where it needs a user, so the same protected work-orders component remains protected when embedded in a public dashboard. This lab uses an in-memory map so the lifecycle is visible; Lab 10 moves the same records to Toasty/SQLite. The development mail transport writes a real `.eml` file under `mail/`, making the magic-link flow inspectable without an SMTP account.

## Steps
### Step 1 — Configure cookies, sessions, and mail
Start from the Lab 05 solution. Install cookie support, the pinned session API, and the file mail transport on the module router:

```rust
{{#include solution/src/app.rs:app-router}}
```

The default session cookie is opaque, secure, HTTP-only, and SameSite=Lax. For application cookies, Topcoat also provides signed and encrypted jar adapters; keep keys in application state and never hard-code production secrets.
> [!NOTE]
> **Checkpoint:** `cargo check -p lab06-solution` succeeds and the router still renders `/`, `/berths`, and `/api/health`.

### Step 2 — Own the session record
Add an app-scoped in-memory store keyed by `TokenHash`. `session::start` returns the hash and expiry to persist; `token_hash` reads the presented token; `refresh` updates the expiry; `rotate` replaces the token; and `stop` returns the old hash for deletion:

```rust
{{#include solution/src/auth.rs:session-store}}
```

The map is deliberately application-owned. Topcoat does not decide whether the authenticated subject is a user, organization, or service account, and it never needs the raw cookie token on the server.
> [!NOTE]
> **Checkpoint:** `cargo test -p lab06-solution --test auth` can establish a session and render `/admin`.

### Step 3 — Add `require_auth` as a function
Resolve the current user from the request token and guard only the component or page that needs it:

```rust
{{#include solution/src/auth.rs:require-auth}}
```

The `work_orders` component calls the guard itself. The public dashboard embeds that component,
proving the protection travels with the markup rather than depending on a route-level middleware
chain. Before returning the redirect error, `require_auth` stores the attempted GET or HEAD path in
a private, signed cookie that expires after ten minutes.

Topcoat 0.8.1 serializes queued cookie changes even when a handler returns an error such as this
redirect ([tokio-rs/topcoat#408](https://github.com/tokio-rs/topcoat/pull/408)). In 0.8.0 the router
dropped the `Set-Cookie` header on error responses, so the login flow could not remember where the
request started.

> [!NOTE]
> `error::redirect` returns 307, preserving the original method and body, while `error::see_other`
> returns 303 and follows with GET. In 0.8.1 `SeeOther` implements `Error`, so a view-returning guard
> can send it through `Err`. The lab keeps 307 in this pass, but a browser auth guard should prefer
> 303 so a guard that fires during POST does not repost the form body to `/login`.

> [!NOTE]
> **Checkpoint:** an unauthenticated GET of `/admin` or `/dashboard` returns a redirect to `/login`
> and a short-lived `return_to` cookie in the same error response.

### Step 4 — Send and consume a magic link
Request a link for an email address. The handler creates a one-time token, sends a `mail!` message
through `FileTransport`, and exposes the link for local inspection. Verification consumes the token,
starts a fresh session, and redirects to the remembered protected path or `/admin` as a fallback:

```rust
{{#include solution/src/app/_marketing/login/request.rs:magic-link}}
```

Open the generated `.eml` file in `mail/` and copy its `/login/verify?token=...` URL. The token is
single-use and expires; a production application would persist it in the database and rate-limit
requests. Verification consumes and clears the `return_to` cookie, then uses `see_other` so the
browser performs a GET of the original protected page. Without a usable cookie it falls back to
`/admin`.

> [!NOTE]
> Treat the cookie as untrusted when you read it. Reject external URLs, `..`, and every backslash:
> several browsers normalize `/\evil.com` into a protocol-relative destination, so checking only
> for a leading `//` misses the bypass.

> [!NOTE]
> **Checkpoint:** requesting a link creates `mail/*.eml`, and visiting its verification URL sets a
> session cookie, clears `return_to`, and returns 303 to the protected path that started the flow.

### Step 5 — Exercise logout, sliding expiry, and rotation
The authenticated admin page refreshes the session expiry as it resolves the user. Logout calls `session::stop` and deletes the corresponding record; the rotate route replaces the token after a privilege change:

```rust
{{#include solution/src/app/_marketing/admin.rs:admin-page}}
```

State-changing actions use `POST`; a session cookie is only written while the handler still owns
the response headers. Sliding refresh and an authentication failure cannot occur together in this
lab: refresh runs only after `require_auth` has resolved a valid session. The return-to feature is
the real error-response cookie case because it intentionally queues a cookie immediately before the
auth guard returns its redirect error.

Keep the file transport and in-memory store for this lab, then replace them with production
implementations in later modules.
> [!NOTE]
> **Checkpoint:** after logout, the old cookie no longer authorizes `/admin`; the auth integration
> test proves it.

### Step 6 — Prove the unauthenticated redirect and login lifecycle
Use the in-process router test style from Lab 05:

```rust
{{#include solution/tests/auth.rs:auth-integration-test}}
```

The test hits the real router, checks that a protected GET returns both the redirect and the
`return_to` cookie, reads the generated mail, follows the magic link, and verifies the 303 lands on
the original protected path. It then sends the returned session cookie to that path and verifies
logout invalidates the session.
> [!NOTE]
> **Checkpoint:** `cargo test -p lab06-solution --test auth` passes, including the complete
> `/dashboard` → `/login` → `/dashboard` round trip.

## Stretch goals
- Replace the in-memory session map with a Toasty model while keeping the same helper signatures.
- Add a request rate limit for magic-link requests and make tokens cryptographically random.
- Add a “sign out everywhere” action that deletes every session record for one user.

## Troubleshooting
- **`/admin` returns 500** → sessions or cookies were not registered on the router → add `.cookies().sessions(...)` before `.build()`.
- **The session cookie is present but the user is anonymous** → the cookie token hash has no live application record → persist the `Session` returned by `session::start` and check expiry.
- **The mail directory is empty** → the mail feature/configuration is missing → enable Topcoat's `mail` feature and register `MailConfig` with `FileTransport`.
- **A protected component works on one page but not another** → the guard was placed in the route instead of the component → call `require_auth(cx)` at the component's first line.
- **A cookie write panics during streaming** → headers already started → set or remove cookies before returning a streaming view.

## What's next
Lab 07 moves from request-scoped authentication into client-side signals and `$(...)` expressions without adding a browser bundle.
