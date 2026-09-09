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

The `work_orders` component calls the guard itself. The public dashboard embeds that component, proving the protection travels with the markup rather than depending on a route-level middleware chain.
> [!NOTE]
> **Checkpoint:** an unauthenticated request to `/admin` and `/dashboard` returns a redirect to `/login`.

### Step 4 — Send and consume a magic link
Request a link for an email address. The handler creates a one-time token, sends a `mail!` message through `FileTransport`, and exposes the link for local inspection. Verification consumes the token, starts a fresh session, and redirects to `/admin`:

```rust
{{#include solution/src/app/_marketing/login/request.rs:magic-link}}
```

Open the generated `.eml` file in `mail/` and copy its `/login/verify?token=...` URL. The token is single-use and expires; a production application would persist it in the database and rate-limit requests.
> [!NOTE]
> **Checkpoint:** requesting a link creates `mail/*.eml`, and visiting its verification URL sets a session cookie.

### Step 5 — Exercise logout, sliding expiry, and rotation
The authenticated admin page refreshes the session expiry as it resolves the user. Logout calls `session::stop` and deletes the corresponding record; the rotate route replaces the token after a privilege change:

```rust
{{#include solution/src/app/_marketing/admin.rs:admin-page}}
```

State-changing actions use `POST`; a session cookie is only written while the handler still owns the response headers. Keep the file transport and in-memory store for this lab, then replace them with production implementations in later modules.
> [!NOTE]
> **Checkpoint:** after logout, the old cookie no longer authorizes `/admin`; the auth integration test proves it.

### Step 6 — Prove the unauthenticated redirect and login lifecycle
Use the in-process router test style from Lab 05:

```rust
{{#include solution/tests/auth.rs:auth-integration-test}}
```

The test hits the real router, checks the redirect location, reads the generated mail, follows the magic link, sends the returned cookie to `/admin`, and verifies logout invalidates it.
> [!NOTE]
> **Checkpoint:** `cargo test -p lab06-solution --test auth` passes.

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
