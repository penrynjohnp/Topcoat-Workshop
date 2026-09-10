# How-to: deploy to Azure Container Apps

This is the reference version of [Lab 12](../../02-labs/lab-12.md): what to run if you already know Topcoat and Bicep and just want the shape of a production deployment, without the step-by-step teaching. It assumes the release image from [Lab 12 Step 1](../../02-labs/lab-12.md) already exists.

## Topology

```mermaid
{{#include ../../../../labs/lab-12-build-containerise-deploy/solution/infra/deployment-topology.mmd}}
```

`azd` provisions everything below Container Apps from Bicep, then builds the Dockerfile, pushes it to ACR, and deploys a revision. Nothing in that path is a person clicking in the portal, and nothing in it is a stored password: every arrow into the app is a managed-identity token, and the only arrow into GitHub Actions is OIDC.

## Two identities, least privilege by design

Two user-assigned managed identities do the work here, not one. `identity.bicep` is a generic single-identity factory; `main.bicep` calls it twice — once for the app, once for a bootstrap identity used only by a deployment script:

```bicep
{{#include ../../../../labs/lab-12-build-containerise-deploy/solution/infra/main.bicep:56:77}}
```

The app's identity is attached to the container app itself and is what the running container authenticates with everywhere — ACR pull, PostgreSQL, Key Vault *read*:

```bicep
{{#include ../../../../labs/lab-12-build-containerise-deploy/solution/infra/modules/app.bicep:31:40}}
```

The bootstrap identity is never attached to the container app and never gets `AcrPull` or database access. Its only job is described next: writing one Key Vault secret, once, before the app exists.

## A cookie secret that exists before the app does

Azure Container Apps validates every `keyVaultUrl` secret reference **at deploy time** and fails the whole deployment if the target secret doesn't exist yet. That rules out creating the cookie key in a postprovision hook, since hooks run *after* the app resource is already submitted — too late. `secret-bootstrap.bicep` is an idempotent `Microsoft.Resources/deploymentScripts` (`kind: 'AzureCLI'`) resource that creates the secret if it's absent, running under the bootstrap identity:

```bicep
{{#include ../../../../labs/lab-12-build-containerise-deploy/solution/infra/modules/secret-bootstrap.bicep}}
```

`main.bicep` wires the ordering explicitly with `dependsOn`, so the `app` module cannot start deploying until the script finishes:

```bicep
{{#include ../../../../labs/lab-12-build-containerise-deploy/solution/infra/main.bicep:91:105}}
```

```bicep
{{#include ../../../../labs/lab-12-build-containerise-deploy/solution/infra/main.bicep:150:155}}
```

`keyvault.bicep` grants the bootstrap identity `Key Vault Secrets Officer` (write) and the app's identity `Key Vault Secrets User` (read-only) — neither the app's identity nor the `azd` deployer ever gets write access to the vault:

```bicep
{{#include ../../../../labs/lab-12-build-containerise-deploy/solution/infra/modules/keyvault.bicep}}
```

`app.bicep` references that secret by URI, resolved through the app's own (read-only) identity at revision start — never a literal value in the template:

```bicep
{{#include ../../../../labs/lab-12-build-containerise-deploy/solution/infra/modules/app.bicep:57:63}}
```

## The container app: ingress, probes, scale

Liveness and readiness both call the same `/api/health` route Lab 04 built; `minReplicas: 0` means the app costs nothing while idle, at the cost of a cold start on the next request:

```bicep
{{#include ../../../../labs/lab-12-build-containerise-deploy/solution/infra/modules/app.bicep:86:113}}
```

## PostgreSQL with no password

`postgres.bicep` turns password authentication off entirely and registers the *app's* identity as the Entra administrator, so the running app authenticates with a token instead:

```bicep
{{#include ../../../../labs/lab-12-build-containerise-deploy/solution/infra/modules/postgres.bicep}}
```

Using the app's own identity as the *administrator* is a workshop shortcut — it keeps this deployment to one extra Bicep module instead of two — not a least-privilege pattern to copy verbatim. A production setup would register a narrower, non-admin Entra role for the app and keep the administrator role for a human or a break-glass identity. See [How-to: SQLite to PostgreSQL](postgres.md) for how the app authenticates with the token that registration makes possible, and its most important caveat.

Azure Files, not PostgreSQL, was the original plan for keeping Lab 12 to a single service — mount the Lab 10 SQLite file on a Files share instead of standing up a database server. It was dropped because Container Apps' Azure Files integration authenticates the mount with a **storage account key**, not a managed identity. That would have left one secret-free service sitting next to one storage credential the platform requires — the exact compromise this workshop's "managed identity everywhere" rule refuses to make.

## A read-only postprovision check

By the time `azd provision` finishes, the deployment script above has already guaranteed the cookie key exists — there is nothing left for a hook to create. `scripts/azd-postprovision.sh` only *verifies* the control-plane wiring: that the deployed container app is using its own user-assigned identity, and that its Key Vault secret reference resolves through that same identity to the expected vault and secret name. It never reads the secret's value and never mutates infrastructure:

```sh
{{#include ../../../../labs/lab-12-build-containerise-deploy/solution/scripts/azd-postprovision.sh}}
```

## CI/CD without a stored client secret

`azd pipeline config --provider github` (already run once for this workshop) wires a federated OIDC credential and writes the workflow below. GitHub Actions authenticates to Azure by proving its own workflow identity to Entra ID; there is no `AZURE_CLIENT_SECRET` anywhere.

```yaml
{{#include ../../../../.github/workflows/azure-dev.yml}}
```

## Without azd

None of the above is required to run the image. The same Dockerfile from Lab 12 Step 1 runs on Fly.io, a plain VM, or any other container host — see [Lab 12 Step 12](../../02-labs/lab-12.md) for both. The provided off-Azure path uses a persistent SQLite volume and a stable `SLIPWAY_COOKIE_KEY`; this lab's PostgreSQL connector is specifically managed-identity based and does not accept a password connection secret.
