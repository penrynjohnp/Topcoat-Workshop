# Slipway Capstone

Slipway is the finished Topcoat workshop application: the Lab 12 marina manager with Lab 13's Tower bridge. It keeps the server-rendered Topcoat UI, Toasty persistence, authentication, sessions, assets, production container, and Azure deployment, while mounting an Axum JSON API under `/api/v1` with compression and tracing middleware.

## Run locally

From the repository root:

```bash
cargo run -p slipway-capstone
```

The development configuration uses SQLite. Start the app with the asset bundle available when you want the complete styled UI:

```bash
topcoat dev -p slipway-capstone
```

Useful checks:

```bash
curl http://127.0.0.1:3000/api/health
curl http://127.0.0.1:3000/api/v1/health
cargo test -p slipway-capstone
```

## Deploy with Azure Developer CLI

Prerequisites: Azure CLI, Azure Developer CLI (`azd`), Docker, and an Azure subscription with permission to create a resource group, Container Apps resources, PostgreSQL Flexible Server, Key Vault, ACR, and managed identities.

Run `azd` from this directory. The Bicep templates provision the resource group, Container Apps environment, registry, PostgreSQL, Key Vault, monitoring, and managed identities. No passwords or registry admin credentials are required.

```bash
cd slipway
az login
azd auth login
azd env new slipway-dev
azd env set AZURE_LOCATION uksouth   # choose an available region if needed
azd up
```

`azd up` provisions the infrastructure, builds and pushes the release image, deploys the Container App, and runs the read-only post-provision wiring check. The public endpoint is available with:

```bash
azd show
azd env get-values
azd monitor --logs
```

The app binds to `0.0.0.0:8080` in the container. The deployment health probe uses `/api/health`; the Lab 13 API is available at `/api/v1/health`.

When finished, remove the workshop environment to stop Azure charges:

```bash
azd down
```

## Workspace notes

The capstone is a workspace member, but its package name is `slipway-capstone` because Lab 12's solution already owns the package name `slipway`. Its release binary is still named `slipway` inside the container.
