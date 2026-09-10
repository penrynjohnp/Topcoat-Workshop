# How-to: SQLite to PostgreSQL

Lab 10 gives Slipway a real database: Toasty models, generated queries, DB-backed sessions, all against a local SQLite file. This page is what changes to run the same models against PostgreSQL — the swap Lab 12 makes for production.

## The connection is a config decision, not a code rewrite

Toasty models, `#[belongs_to]`/`#[has_many]` relations, and every generated query stay identical between SQLite and PostgreSQL — see [What Toasty does](../../01-concepts/toasty.md). What changes is which URL `toasty::Db::builder().connect(...)` receives and how that URL gets its credential. `AppConfig` in Lab 12 expresses that as a typed enum instead of a single connection string, precisely because the two paths need different fields:

```rust
{{#include ../../../../labs/lab-12-build-containerise-deploy/solution/src/config.rs:6:26}}
```

A local, non-Azure PostgreSQL server needs nothing more exotic than a standard `postgresql://user:password@host/db` URL passed to the same `toasty::Db::builder().connect(&url)` call [Lab 10's `connect`](../../02-labs/lab-10.md) already uses for SQLite:

```rust
{{#include ../../../../labs/lab-10-toasty-sqlite/solution/src/database.rs:database-setup}}
```

Only the URL scheme and its embedded credential differ; Toasty's builder API does not otherwise change per backend. That said, this is a statement about Toasty and Lab 10's `connect` function in the abstract, not a supported path in the Lab 12 `slipway` binary: `AppConfig` (above) has exactly two variants, `Sqlite` and `ManagedPostgres` — there is no third variant for a plain password-authenticated PostgreSQL URL, so off-Azure production deployments (Step 12 of the lab) use the `Sqlite` variant against a persistent volume, not a password-authenticated PostgreSQL server.

## Azure PostgreSQL with a managed-identity token instead of a password

On Azure, Lab 12 replaces that password with a short-lived Entra access token, fetched from the container's user-assigned managed identity and percent-encoded into the same connection-URL shape:

```rust
{{#include ../../../../labs/lab-12-build-containerise-deploy/solution/src/production.rs:managed-postgres-token}}
```

`sslmode=verify-full` is not optional here: an access token traveling as a URL-encoded password must go over a verified TLS connection, or it is exposed exactly like a plaintext password would be. The matching `postgres.bicep` module disables password authentication on the server entirely, so there is no fallback path that would silently accept one:

```bicep
{{#include ../../../../labs/lab-12-build-containerise-deploy/solution/infra/modules/postgres.bicep:35:39}}
```

## Read this before you rely on it past a workshop

> [!WARNING]
> Toasty 0.10.0's PostgreSQL driver takes one access token at connect time and stores it as a static credential on that connection — there is no renewable-token callback. Entra access tokens for `ossrdbms-aad.database.windows.net` are short-lived (typically under 24 hours). A connection opened while the token was valid generally keeps working after expiry, but a **new** pool connection opened after that point can fail to authenticate, and nothing in the driver retries with a fresh token. The only recovery today is restarting the process (in Container Apps: `az containerapp revision restart`) so it fetches a new token on its next connect.
>
> Treat this pattern as **experimental**: appropriate for a workshop, a demo, or a short-lived environment where a restart is cheap, not yet as a long-running production connection-pooling story. Watch [Toasty's PostgreSQL driver](https://github.com/tokio-rs/toasty) for a refresh-callback API before adopting this beyond that.
>
> Toasty 0.10.0's `push_schema()` is also not idempotent. Slipway probes for the `berths` relation and calls `push_schema()` only for a fresh database, so ordinary Container Apps restarts do not try to recreate every table. This is a bootstrap guard, not a migration system; use versioned migrations before evolving a deployed schema.

## Registering the administrator

Microsoft Entra-only PostgreSQL Flexible Server needs at least one Entra administrator before any token-based connection can succeed. Lab 12 registers the app's own managed identity in that role, rather than provisioning a second identity just for database administration:

```bicep
{{#include ../../../../labs/lab-12-build-containerise-deploy/solution/infra/modules/postgres.bicep:43:51}}
```

That is a bootstrap shortcut, not least privilege: the identity that runs queries and the identity that administers the server are the same one. A production deployment would register a human or break-glass identity as the administrator and grant the application identity a narrower, non-admin Entra role instead — see [How-to: deploy to Azure Container Apps](deploy-azure.md) for the rest of that identity's responsibilities (ACR pull, Key Vault secret read).
