# Lab 12 — Build, containerise, and deploy with azd

**Time:** 120 min · **Module:** 6 · **Prerequisites:** Lab 11 complete (or copy `labs/lab-11-assets-tailwind-ui/solution`); `az`, `azd`, and Docker installed (all three are in the devcontainer)

## What you'll learn
- Package a release binary and its matching asset bundle into a minimal, non-root multi-stage container that binds `0.0.0.0` and fails fast on missing production configuration
- Provision Azure Container Apps, ACR, Key Vault, and an Entra-only PostgreSQL Flexible Server from Bicep modules split across two least-privilege managed identities — one the running app uses, one that only ever bootstraps a secret — instead of any password or connection-string secret
- Run the full `azd up` → `azd monitor` → `azd down` loop, including a Bicep-driven secret bootstrap that runs before the app exists, a read-only postprovision verification hook, and a GitHub Actions pipeline authenticated by OIDC

## Concepts (read first, 5 min)
Read [How-to: deploy to Azure Container Apps](https://github.com/penrynjohnp/Topcoat-Workshop/blob/main/docs/src/03-reference/howto/deploy-azure.md) and [How-to: SQLite to PostgreSQL](https://github.com/penrynjohnp/Topcoat-Workshop/blob/main/docs/src/03-reference/howto/postgres.md) alongside the [ACA Bicep quickstart](https://containerapps.azure.com/docs/containerapps/quickstart/setup-bicep). Going to production changes three things Lab 01–11 could ignore. First, the binary and its `topcoat asset bundle` output must come from the *same* release build — a debug bundle beside a release binary, or vice versa, panics at render time — so the Dockerfile builds and bundles in one stage before copying both into a distroless runtime image. Second, `AppConfig` in `src/config.rs` reads its settings from the environment once at startup and refuses to start in production without a database host and a stable cookie key, rather than silently falling back to development defaults the way `topcoat dev` does. Third, `tracing` switches from human-readable to JSON so Log Analytics can parse it.

`azd` is the second half: it wraps Bicep provisioning, a Docker build/push, and a Container Apps deploy behind three commands, and it reads `azure.yaml` to know your service is `language: docker` rather than a Rust-aware language plugin (there isn't one). No resource in `infra/` is reached with an ACR admin user, a database password, or a connection string with a secret in it — but it takes **two** user-assigned managed identities to get there, not one. The app's own identity (ACR pull, Key Vault *read*, PostgreSQL Entra administrator) is what the running container uses at every request. A second, narrowly-scoped bootstrap identity exists purely so a Bicep deployment script can create the cookie-signing secret in Key Vault *before* the app resource is even submitted — Container Apps validates `keyVaultUrl` secret references at deploy time and fails the whole deployment if the target secret doesn't exist yet, so that secret cannot wait for a postprovision hook that only runs after the app is created. The one exception the workshop is explicit about: Toasty 0.10's PostgreSQL driver takes a single access token at connect time and has no renewable-token callback, so authenticating it with a managed identity is an experimental pattern here, not an established one — read the warning in Step 4 before you rely on it past a workshop.

## Steps

### Step 1 — Build a release image with matching assets
The Dockerfile is two stages: a builder that compiles the release profile and bundles assets from that exact build, and a distroless runtime that copies only the binary and the `assets/` directory it produced, running as a non-root user:

```dockerfile
{{#include solution/Dockerfile:release-container}}
```

`HOST=0.0.0.0` is the line that matters most. Every prior lab's default — and `topcoat dev`'s default — is `127.0.0.1`, which is invisible from outside the container. Binding `0.0.0.0` and matching the `PORT` your platform expects is the difference between a working deployment and a container that "starts fine" and then 502s. The two `--mount=type=cache` mounts persist the Cargo registry and `target/` across builds without baking them into any image layer, so a code-only change rebuilds in seconds.

Build and run it locally exactly as `azd` will:
```bash
DOCKER_BUILDKIT=1 docker build -f labs/lab-12-build-containerise-deploy/solution/Dockerfile -t slipway:release .
docker run --rm -p 8080:8080 -e SLIPWAY_ENV=development slipway:release
```
> [!NOTE]
> **Checkpoint:** `curl http://localhost:8080/api/health` returns `{"status":"ok"}`, and the container log line is single-line JSON containing `"message":"starting Slipway"`.

### Step 2 — Typed, fail-fast environment configuration
`AppConfig` replaces scattered `std::env::var` calls with one typed struct built once at startup. `DatabaseConfig` is an enum, not a URL string, because SQLite and managed PostgreSQL need different fields entirely:

```rust
{{#include solution/src/config.rs:6:26}}
```

`from_pairs` (used by `from_env` and by tests) refuses to start in production without *some* database configured — a managed-PostgreSQL host, or an explicitly-set `SLIPWAY_DATABASE_URL` for an off-Azure SQLite volume — and without a stable, previously-generated cookie key. It never silently falls back to the in-memory key or the relative `./slipway.db` file the way development mode does:

```rust
{{#include solution/src/config.rs:65:94}}
```

> [!NOTE]
> **Checkpoint:** `cargo test -p slipway --test config` passes all six tests, including the one asserting the exact `"SLIPWAY_DATABASE_HOST is required in production unless SLIPWAY_DATABASE_URL is explicitly set"` error text, and the one proving `SLIPWAY_ENV=production` with an explicit `SLIPWAY_DATABASE_URL` and no `SLIPWAY_DATABASE_HOST` still starts.

### Step 3 — Wire structured tracing and typed config into `main`
`main.rs` now calls `production::init_tracing()` before anything else, builds `AppConfig::from_env()`, and dispatches to SQLite or managed PostgreSQL based on which variant `from_env` returned:

```rust
{{#include solution/src/main.rs}}
```

```rust
{{#include solution/src/production.rs:structured-tracing}}
```

`tracing_subscriber`'s `EnvFilter` still reads `RUST_LOG`, but `.json()` changes the *shape* of every line instead of just its verbosity — that's what lets Log Analytics parse fields like `host`, `port`, and `database_kind` instead of grepping free text.

> [!NOTE]
> **Checkpoint:** run `docker run --rm -p 8080:8080 -e SLIPWAY_ENV=development -e RUST_LOG=info slipway:release` again and confirm the startup line is valid JSON with a `"database_kind":"sqlite"` field (use `docker logs <container> | python3 -m json.tool` on the first line to prove it parses).

### Step 4 — Authenticate to PostgreSQL with a managed-identity token
`connect_managed_postgres` fetches an Entra access token scoped to `ossrdbms-aad.database.windows.net` from the container's user-assigned identity and percent-encodes it as the connection URL's password:

```rust
{{#include solution/src/production.rs:managed-postgres-token}}
```

> [!WARNING]
> Toasty 0.10.0's PostgreSQL driver stores this token once, as a static credential on the connection, and has no renewable-token callback. Access tokens for `ossrdbms-aad.database.windows.net` are short-lived. A connection your app already holds open when the token expires typically keeps working, but a **new** pool connection opened after expiry can fail to authenticate — and the only fix today is restarting the container app revision (`az containerapp revision restart`) so it fetches a fresh token on the next connect. Treat this managed-identity path as **experimental**, appropriate for a workshop or a short-lived demo, not as a production connection-pooling story until Toasty grows a refresh callback. Because `push_schema()` is not idempotent either, the connector first probes for the `berths` relation and only bootstraps a genuinely fresh database; this keeps routine restarts safe but is not a substitute for versioned migrations.

> [!NOTE]
> **Checkpoint:** `cargo test -p slipway --lib` passes `production::tests::credentials_are_percent_encoded_and_tls_is_verified`, proving the token is percent-encoded (so a `/` or `+` in it can't break the URL) and that `sslmode=verify-full` is always set.

### Step 5 — Define the azd project
`azure.yaml` tells `azd` there is one service, `web`, built from this Dockerfile with `language: docker` (there is no Rust-aware `azd` language plugin), and it declares a `postprovision` hook:

```yaml
{{#include solution/azure.yaml}}
```

From the repository root, `slipway` is a symlink to this lab's `solution/` directory — `azd` commands in this lab run from `slipway/`, and `project: ../../..` in `azure.yaml` resolves back through that symlink to the workspace root, which is what makes `docker build`'s `context: .` include the whole Cargo workspace the release build needs.

> [!NOTE]
> **Checkpoint:** `cd slipway && azd env list` runs without error (add an environment with `azd env new lab12-workshop --location <region>` if none exists yet).

### Step 6 — Two managed identities, least privilege by design
`identity.bicep` is a generic single-identity factory — it takes a name and creates one user-assigned identity. `main.bicep` calls it **twice**: once for the app (`identity`) and once for a narrowly-scoped bootstrap identity used only by a deployment script:

```bicep
{{#include solution/infra/main.bicep:56:77}}
```

```bicep
{{#include solution/infra/modules/identity.bicep}}
```

The app's identity is what the running container authenticates with — ACR pull (below), the PostgreSQL Entra administrator registration (Step 8), and a Key Vault *read* role (next step). The bootstrap identity is never attached to the container app at all and never gets `AcrPull` or database access; its only job, next step, is writing one secret once. `registry.bicep` disables the ACR admin user entirely and grants only `AcrPull` to the app's identity:

```bicep
{{#include solution/infra/modules/registry.bicep}}
```

> [!NOTE]
> **Checkpoint:** after `azd provision` (Step 11 covers running it), `az role assignment list --assignee <app-identity-principal-id> --all --query "[].roleDefinitionName"` lists `AcrPull`, and the same query against the *bootstrap* identity's principal ID does **not** — only `Key Vault Secrets Officer` (next step) shows up there.

### Step 7 — A cookie secret that exists before the app does
Azure Container Apps validates every `keyVaultUrl` secret reference **at deploy time** and fails the entire deployment if the target secret doesn't exist yet. That rules out creating the cookie signing key in a postprovision hook — hooks run *after* the app resource is already submitted, which is too late. Instead, `infra/modules/secret-bootstrap.bicep` is an idempotent `Microsoft.Resources/deploymentScripts` (`kind: 'AzureCLI'`) resource that creates the secret if it's absent, running under the bootstrap identity from Step 6:

```bicep
{{#include solution/infra/modules/secret-bootstrap.bicep}}
```

`main.bicep` wires the ordering explicitly: the `app` module's `dependsOn` names `secretBootstrap`, so Bicep will not even start deploying the container app until the script has finished:

```bicep
{{#include solution/infra/main.bicep:91:105}}
```

```bicep
{{#include solution/infra/main.bicep:150:155}}
```

`keyvault.bicep` grants the bootstrap identity `Key Vault Secrets Officer` (write) and the app's identity `Key Vault Secrets User` (read-only) — neither the app nor the `azd` deployer ever gets write access to the vault:

```bicep
{{#include solution/infra/modules/keyvault.bicep}}
```

`app.bicep` then references that secret by URI, resolved through the app's own (read-only) identity — never a literal value in the template:

```bicep
{{#include solution/infra/modules/app.bicep:57:63}}
```

> [!NOTE]
> **Checkpoint:** after `azd provision`, `az deployment group show --resource-group <rg> --name secret-bootstrap --query properties.provisioningState -o tsv` prints `Succeeded`. The app module succeeding after that proves ACA resolved the pre-existing secret reference; the deployer intentionally has no data-plane permission to read the secret. `az role assignment list --assignee <bootstrap-identity-principal-id> --all --query "[].roleDefinitionName"` lists only `Key Vault Secrets Officer`.

### Step 8 — Entra-only PostgreSQL, and why there's no Azure Files
`postgres.bicep` disables password authentication outright and registers the *app's* identity as the server's Entra administrator:

```bicep
{{#include solution/infra/modules/postgres.bicep}}
```

Registering the app's own identity as the PostgreSQL Entra administrator is a workshop bootstrap shortcut, not least-privilege production practice — a real deployment would provision a narrower non-admin Entra role for the app and reserve the administrator role for a human or a break-glass identity. It keeps this lab to one extra Bicep module instead of two; say so if you carry this pattern further.

The earlier plan for this lab considered mounting SQLite on an Azure Files volume so Lab 12 stayed a single service. That was deliberately dropped: Container Apps' Azure Files integration authenticates the storage mount with a **storage account key**, not a managed identity, so keeping SQLite-on-Files would have meant one secret-free service (the app) sitting next to one storage credential the platform requires you to hold — a compromise this workshop's "managed identity everywhere, no secrets in templates" rule doesn't accept. PostgreSQL with Entra-only auth removes that exception instead of tolerating it.

> [!NOTE]
> **Checkpoint:** `az postgres flexible-server show --name <server-name> --resource-group <rg> --query authConfig` reports `"activeDirectoryAuth": "Enabled"` and `"passwordAuth": "Disabled"`.

### Step 9 — Health probes and scale-to-zero
The container app reuses Lab 04's `/api/health` route for both probes, and scales to zero replicas when idle:

```bicep
{{#include solution/infra/modules/app.bicep:86:113}}
```

A readiness probe with no traffic-serving replica means the *first* request after a cold start waits for a replica to start and pass that probe before it's routed — expect a multi-second delay the first time you load the URL after it's been idle. `environment.bicep` wires the Consumption workload profile to Log Analytics so every `tracing::info!` line — including the one the health route emits — lands in the workspace `azd monitor` reads from:

```bicep
{{#include solution/infra/modules/environment.bicep}}
```

```rust
{{#include solution/src/app/_marketing/api/health.rs:health-route}}
```

> [!NOTE]
> **Checkpoint:** `az containerapp show --name <app-name> --resource-group <rg> --query "properties.template.scale.minReplicas"` returns `0`, and `--query "properties.template.containers[0].probes"` lists both a `Liveness` and a `Readiness` probe on `/api/health`.

### Step 10 — A read-only postprovision check
By the time `azd provision` finishes, `secret-bootstrap.bicep` (Step 7) has already guaranteed the cookie key exists — there is nothing left for a hook to create. `scripts/azd-postprovision.sh` instead only *verifies* the control-plane wiring: that the deployed container app is actually using its own user-assigned identity, and that its Key Vault secret reference resolves through that same identity to the expected vault and secret name. It never reads the secret's value and never mutates anything:

```sh
{{#include solution/scripts/azd-postprovision.sh}}
```

> [!NOTE]
> **Checkpoint:** run `./scripts/azd-postprovision.sh` from `slipway/` twice in a row against a provisioned environment. Both runs print the identical `Verified: container app '<name>' uses its own user-assigned identity and its Key Vault secret reference resolves through that identity.` — a read-only check has nothing to change between runs.

### Step 11 — `azd up`, `azd monitor`, `azd down`
With the identity, secrets, and probes in place, run the full loop:
```bash
cd slipway
azd auth login
azd up
```
`azd up` provisions the Bicep, builds and pushes the image to ACR, deploys the revision, and prints the public URL. Iterate with `azd deploy` alone (skips provisioning) after a code change; watch logs with `azd monitor --logs`; tear everything down with `azd down --purge` so the Key Vault name is released rather than left soft-deleted.

The GitHub Actions workflow runs the same two commands (`azd provision`, `azd deploy`) under a federated OIDC credential, so CI never stores a client secret:

```yaml
{{#include ../../.github/workflows/azure-dev.yml}}
```

> [!NOTE]
> **Checkpoint:** `curl "$(azd env get-value WEB_URI)/api/health"` returns `{"status":"ok"}`, and `azd monitor --logs` streams a JSON line for that request within a few seconds.

### Step 12 — A shorter path: plain Docker on Fly.io or a VM
Not every audience wants Bicep and `azd`. The same image from Step 1 runs anywhere that accepts a container, with the same environment-variable contract — this is the whole reason `AppConfig` reads from the environment instead of a config file baked into the image. There is no managed identity off Azure, and this binary has exactly two database paths: managed-identity PostgreSQL (Step 4, Azure-only — it authenticates with an Entra token, not a password, so it cannot be pointed at a plain password-based PostgreSQL server) and SQLite. Off Azure in production mode, that means an explicit `SLIPWAY_DATABASE_URL` pointing at a SQLite file on a volume that survives restarts, plus a stable, pre-generated `SLIPWAY_COOKIE_KEY` — an ephemeral container filesystem or a freshly-generated key on every restart would silently lose data and log sessions out.

**Fly.io:** `fly launch --dockerfile labs/lab-12-build-containerise-deploy/solution/Dockerfile --no-deploy`, attach a persistent volume (`fly volumes create slipway_data --size 1`) mounted at `/data`, then set `fly secrets set SLIPWAY_ENV=production SLIPWAY_DATABASE_URL=sqlite:/data/slipway.db SLIPWAY_COOKIE_KEY=$(head -c 64 /dev/urandom | base64 | tr -d '\n')` before `fly deploy` — the same base64-of-64-random-bytes shape `secret-bootstrap.bicep` (Step 7) generates for Azure.

**A plain VM:** `docker build` the same image, `docker save | ssh vm docker load` (or push to any registry the VM can reach), then bind-mount a host directory for the database file and pass a fixed cookie key: `docker run -d --restart=always -p 8080:8080 -v /srv/slipway-data:/data -e SLIPWAY_ENV=production -e SLIPWAY_DATABASE_URL=sqlite:/data/slipway.db -e SLIPWAY_COOKIE_KEY=<base64-of-64-bytes> slipway:release` — behind whatever reverse proxy terminates TLS. Generate the key once (`head -c 64 /dev/urandom | base64 | tr -d '\n'`), store it outside the image, and reuse it across restarts and redeploys; a new key on every restart invalidates every existing session cookie.

> [!NOTE]
> **Checkpoint:** either path serves `/api/health` over HTTPS at a public URL under `SLIPWAY_ENV=production`, without `azd`, `az`, or any Bicep file involved — and data written through the app is still present after restarting the container/machine, proving the volume (not the container's writable layer) is where `slipway.db` lives.

## Stretch goals
- Swap the Consumption Container Apps environment for a VNet-integrated one with a private endpoint to PostgreSQL, and drop the `0.0.0.0` firewall rule from `postgres.bicep`.
- Give the app a narrower, non-administrator Entra role on PostgreSQL instead of the bootstrap administrator registration from Step 8, and confirm `connect_managed_postgres` still works.
- Add a second container app revision with 20% traffic for a blue/green demo, using Container Apps' revision traffic-splitting.
- Add a nightly scheduled GitHub Actions job that runs `azd down --purge` against a disposable workshop tenant so nobody leaves an environment running between sessions.

## Troubleshooting
- **Deployed, but every request times out or 502s.** `HOST` is still `127.0.0.1`. Check the Dockerfile `ENV` line and any `-e HOST=` override; ACA's ingress cannot reach a socket bound to loopback.
- **`Error: ConfigError("SLIPWAY_DATABASE_HOST is required in production unless SLIPWAY_DATABASE_URL is explicitly set")` on startup.** `SLIPWAY_ENV=production` (the Dockerfile's default) with no database configured. On Azure, set `SLIPWAY_DATABASE_HOST`/`_NAME`/`_USER` and `AZURE_CLIENT_ID` (Step 4). Off Azure, set an explicit `SLIPWAY_DATABASE_URL` pointing at a SQLite file on a persistent volume (Step 12) — or run locally with `-e SLIPWAY_ENV=development` for the ephemeral SQLite fallback.
- **`azd deploy` fails to pull from ACR right after `azd provision`.** The `AcrPull` role assignment can take up to a minute to propagate. Retry.
- **New database connections start failing hours after a successful deploy, but existing ones still work.** This is the Step 4 warning: the managed-identity access token Toasty is holding has expired and it has no callback to refresh it. Restart the revision (`az containerapp revision restart`) to force a fresh token on the next connect; this is a known limitation of Toasty 0.10.0, not a misconfiguration.
- **`azd provision` fails deploying the `app` module with a missing Key Vault secret reference.** `secret-bootstrap.bicep`'s own RBAC-propagation retry loop (Step 7) can still lose a race on a very fresh subscription — it gives up after six 15-second attempts. Re-run `azd provision`; the script is idempotent and the `app` module's `dependsOn` guarantees it never runs before the secret exists.
- **`azd down` succeeds but a re-`azd up` fails with a Key Vault name conflict.** Use `azd down --purge` — without it, the vault is soft-deleted and its name stays reserved for its retention period.
- **`topcoat build` is unknown.** As in Lab 11, CLI 0.8.0 has no such command; the Dockerfile uses `cargo build --release` followed by `topcoat asset bundle --release`, matching profiles.

## What's next
Slipway is now a deployed, production-configured app with no secrets in source control. The optional Lab 13 goes one direction further — mounting an Axum router for a JSON API under `/api/v1` through Topcoat's tower bridge, inside this same container.
