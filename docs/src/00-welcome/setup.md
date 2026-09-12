# Setup

Set up the pinned Rust and Topcoat toolchain before Lab 01. You can work directly on Linux or macOS, use Windows through WSL, or open the repository in a devcontainer or GitHub Codespace.

The workshop currently targets Topcoat and Topcoat CLI 0.8.0, Toasty 0.10.0, and the versions recorded in [`COMPATIBILITY.md`](https://github.com/penrynjohnp/Topcoat-Workshop/blob/main/COMPATIBILITY.md). Topcoat is pre-1.0, so keep the repository checkout and its pins together.

## The setup script

From the repository root, run:

```bash
./scripts/setup.sh
```

The script checks for `rustup`, reports the active toolchain, installs or refreshes `topcoat-cli` 0.8.0 with `--locked`, notes whether the `sqlite3` command is available, and builds Lab 01's solution as a smoke test:

```bash
{{#include ../../../scripts/setup.sh}}
```

A missing `sqlite3` command is only a note at this stage. Lab 10 uses SQLite, so install it before that lab if you are not using the devcontainer.

> [!WARNING]
> Topcoat CLI 0.8.0 has no top-level `topcoat --version` command. Check the installed version with `cargo install --list`, or use `topcoat fmt --version` for the formatter binary. `topcoat fmt` also has no `--check` flag; the repository formats and then checks for a Git diff.

## Linux

Install Rust with [rustup](https://rustup.rs), then clone the workshop and run the setup script:

```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
git clone https://github.com/penrynjohnp/Topcoat-Workshop.git
cd Topcoat-Workshop
./scripts/setup.sh
```

The exact system package names vary by distribution. You need a C compiler and linker for native dependencies, and Lab 10 needs the `sqlite3` command-line client. On Debian or Ubuntu, install it with:

```bash
sudo apt-get install -y sqlite3
```

You also need a working Docker installation for the container steps in Lab 12. Docker is not required for Labs 01–11.

## macOS

Install Xcode Command Line Tools, Rust, and Git:

```bash
xcode-select --install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
git clone https://github.com/penrynjohnp/Topcoat-Workshop.git
cd Topcoat-Workshop
./scripts/setup.sh
```

Install SQLite before Lab 10 if it is not already available. Homebrew is one option:

```bash
brew install sqlite3
```

Lab 12 also needs Docker Desktop running. The setup script does not install Docker, Azure CLI, or `azd`; install those separately when you reach the deployment lab.

## Windows through WSL

Use a Linux distribution under [WSL 2](https://learn.microsoft.com/windows/wsl/install) and run the workshop inside the WSL filesystem, such as under `~/src`, rather than under `/mnt/c` when possible. This gives Cargo and file watchers normal Linux filesystem behaviour.

Inside WSL, follow the Linux instructions:

```bash
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev sqlite3 curl git
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
git clone https://github.com/penrynjohnp/Topcoat-Workshop.git
cd Topcoat-Workshop
./scripts/setup.sh
```

For Lab 12, either install Docker Desktop on Windows with WSL 2 integration enabled, or install Docker Engine inside WSL. Confirm that this works from the same WSL shell where you run `azd`:

```bash
docker version
```

Run the Azure CLI and `azd` in the same environment as the deployment commands. Do not mix a Windows `azd` with a WSL project path unless you understand which filesystem and Docker daemon each command is using.

## Devcontainer

The repository includes a `.devcontainer/devcontainer.json`. Open the repository in VS Code with the Dev Containers extension, choose **Reopen in Container**, and wait for `postCreateCommand` to finish.

The container provides Rust, Node.js LTS, Azure CLI, `azd`, and Docker-in-Docker. Its post-create command installs Chromium for Marp slide export, `sqlite3`, Topcoat CLI 0.8.0, mdBook 0.5.4, mdbook-mermaid 0.17.1, and mdbook-linkcheck2 0.13.0:

```json
{{#include ../../../.devcontainer/devcontainer.json}}
```

The container forwards ports 3000 and 8080. Run `./scripts/setup.sh` once after the container is ready; this verifies the active Rust toolchain and builds Lab 01 even though the post-create command has already installed the CLI.

## GitHub Codespaces

Select **Code → Codespaces → Create codespace on main** from the repository, or create a Codespace from your fork. Codespaces detects the devcontainer configuration and runs the same post-create setup as the local devcontainer.

After the Codespace opens, verify the tools:

```bash
rustup show active-toolchain
cargo --version
node --version
chromium --version
topcoat fmt --version
mdbook --version
az version
azd version
docker version
```

Then run:

```bash
./scripts/setup.sh
```

Codespaces is suitable for Labs 01–11 and for building the Lab 12 image. Azure deployment still requires an Azure login, subscription access, and a Codespace with enough resources for Docker builds. The Codespace's forwarded ports are for local development; an Azure Container Apps deployment gets its own public URL.

## Azure prerequisites for Lab 12

Lab 12 provisions Azure Container Apps, a container registry, Key Vault, managed identities, and an Entra-authenticated PostgreSQL Flexible Server through Bicep and `azd`. Complete Labs 01–11, or start from the Lab 11 solution, before deploying.

You need:

- An Azure subscription where you can create the resource group and the resources in the Lab 12 Bicep modules.
- Permission to create role assignments. The deployment grants the app identity ACR pull, Key Vault read, and PostgreSQL administrator access, and grants the separate bootstrap identity permission to create the cookie secret. A subscription or resource-group policy can block these assignments even when ordinary resource creation is allowed.
- An Azure region that supports the selected Container Apps and PostgreSQL resources, with sufficient quota. `azd` asks for the location when you create the environment.
- Azure CLI (`az`), Azure Developer CLI (`azd`), and Docker with a running daemon. The devcontainer supplies all three.
- An authenticated Azure session:

```bash
az login
az account show
azd auth login
```

Select the intended subscription before provisioning if your account has more than one:

```bash
az account set --subscription <subscription-id>
```

From the capstone directory, create or select an `azd` environment, then deploy:

```bash
cd slipway
azd env new lab12-workshop --location <region>
azd up
```

`azd up` provisions the Bicep resources, builds the Docker image, pushes it to ACR, and deploys a Container Apps revision. The app's managed identity and the bootstrap identity are created by Bicep; do not add passwords or secret values to the templates.

Before running `azd up`, make sure Docker can build locally and that you understand the production configuration. The image binds `HOST=0.0.0.0`; the release binary and `topcoat asset bundle --release` output must come from the same build. The Azure path uses managed identity for PostgreSQL and Key Vault. Toasty's 0.10.0 PostgreSQL token has no refresh callback, so the Lab 12 deployment is an experimental workshop path; read the [SQLite-to-PostgreSQL how-to](../03-reference/howto/postgres.md) before relying on it.

To verify and clean up:

```bash
curl https://<your-container-app-host>/api/health
azd monitor --logs
azd down
```

If you need to reuse the same globally-unique Key Vault name after teardown, use `azd down --purge`; a soft-deleted vault can retain its name during the retention period. See [How-to: deploy to Azure Container Apps](../03-reference/howto/deploy-azure.md) for the identity and deployment details.

## First check

A successful setup ends with the script's `✅ ready` message. Start with the Lab 01 README:

```bash
cd labs/lab-01-hello-topcoat
cat README.md
```

**See also:** [Welcome](index.md), [Lab 01](../02-labs/lab-01.md), [How-to: pin and upgrade Topcoat](../03-reference/howto/upgrade.md), and [How-to: deploy to Azure Container Apps](../03-reference/howto/deploy-azure.md).
