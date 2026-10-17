# CLI commands

This page records the `--help` surface of **Topcoat CLI 0.10.0**, the version pinned by this
workshop. The CLI has four command groups: `dev`, `fmt`, `asset`, and `ui`.
The command tables are checked against the installed 0.10.0 binary.

> [!WARNING]
> `topcoat --version` and `topcoat -V` print the CLI version.

> [!NOTE]
> There is no `topcoat build` command in 0.10.0. `topcoat build` fails with
> `error: unrecognized subcommand 'build'` and suggests `ui`, which is unrelated. Use Cargo to build
> the binary, then use `topcoat asset bundle` with the same package, binary, and profile selection.

## Top-level command

Usage is `topcoat <COMMAND>`.

| Subcommand | Purpose |
|---|---|
| `topcoat dev` | Start a development server. |
| `topcoat fmt` | Format Topcoat `view!`-family macro bodies. |
| `topcoat asset` | Inspect, bundle, or clean assets embedded in a built binary. |
| `topcoat ui` | Initialize and manage owned UI component source files. |
| `topcoat help` | Print top-level help, or dispatch help through a command group. |

| Flag | Meaning |
|---|---|
| `-h`, `--help` | Print top-level help. |
| `-V`, `--version` | Print the CLI version. |

## `topcoat dev`

Usage is `topcoat dev [OPTIONS]`.

| Flag | Value | Meaning |
|---|---|---|
| `--bin` | `<BIN>` | Build the named binary target. |
| `-p`, `--package` | `<PACKAGE>` | Build the named workspace package. |
| `-r`, `--release` | — | Build with Cargo's `release` profile. |
| `--profile` | `<NAME>` | Build with the named Cargo profile. |
| `-h`, `--help` | — | Print help. |

`dev` rebuilds the selected target and serves it for development. The workshop normally selects the
package explicitly from the workspace, for example with `-p lab11-solution`.
The 0.10.0 CLI honors Cargo's `default-run`, works again through `cargo topcoat`, preserves build
environment settings, and streams development reloads without losing current form state.

`--release` and `--profile` both select a profile. Use one profile consistently when the application
loads a generated asset bundle.

## `topcoat fmt`

Usage is `topcoat fmt [OPTIONS] [FILES]...`.

| Argument or flag | Value | Meaning |
|---|---|---|
| `[FILES]...` | Paths | Format the named Rust files. |
| `--stdin` | — | Read Rust source from standard input and write formatted source to standard output. |
| `--macros` | `<MACROS>` | Format only the comma-separated macro names supplied. The default is every supported macro. |
| `--check` | — | Check formatting without writing files or emitting formatted source. |
| `--rustfmt` | — | Run rustfmt before formatting macro bodies; requires rustfmt on `PATH` and an explicit edition in root `rustfmt.toml`. |
| `-h`, `--help` | — | Print help. |
| `-V`, `--version` | — | Print the formatter version. |

Run `cargo fmt` and `topcoat fmt --check` as separate checks. Rustfmt treats a macro body as opaque
tokens; `topcoat fmt` understands the markup inside supported Topcoat macros.

## `topcoat asset`

Usage is `topcoat asset <COMMAND>`. The group itself accepts only `-h` and `--help`.

| Subcommand | Purpose |
|---|---|
| `topcoat asset list` | List asset paths embedded in the binary produced by Cargo. |
| `topcoat asset bundle` | Write embedded assets to a bundle directory. |
| `topcoat asset clean` | Delete bundle output and the asset build cache. |
| `topcoat asset help` | Print asset help or help for an asset subcommand. |

### `topcoat asset list`

Usage is `topcoat asset list [OPTIONS]`.

| Flag | Value | Meaning |
|---|---|---|
| `--bin` | `<BIN>` | Inspect the named binary target. |
| `-p`, `--package` | `<PACKAGE>` | Build and inspect the named package. |
| `-r`, `--release` | — | Use the `release` profile. |
| `--profile` | `<NAME>` | Use the named Cargo profile. |
| `-h`, `--help` | — | Print help. |

### `topcoat asset bundle`

Usage is `topcoat asset bundle [OPTIONS]`.

| Flag | Value | Meaning |
|---|---|---|
| `--bin` | `<BIN>` | Bundle assets from the named binary target. |
| `-p`, `--package` | `<PACKAGE>` | Build and bundle the named package. |
| `-r`, `--release` | — | Use the `release` profile. |
| `--profile` | `<NAME>` | Use the named Cargo profile. |
| `-o`, `--out` | `<OUT>` | Write to this directory instead of the default beside the built executable. |
| `-h`, `--help` | — | Print help. |

Without `--out`, the bundle is written to an `assets` directory beside the executable. For a normal
release build that is `target/release/assets/`.

The bundle describes assets embedded in one particular build. Build and bundle the same package,
binary target, and profile. A release binary must not load a bundle produced from a debug build.

### `topcoat asset clean`

Usage is `topcoat asset clean [OPTIONS]`.

| Flag | Value | Meaning |
|---|---|---|
| `-o`, `--out` | `<OUT>` | Remove this bundle directory. Without it, remove every bundle in the Cargo target directory. |
| `-h`, `--help` | — | Print help. |

`clean` also deletes the asset build cache. Use it when diagnosing stale generated assets, then
rebuild the binary before bundling again.

## `topcoat ui`

Usage is `topcoat ui <COMMAND>`. The group itself accepts only `-h` and `--help`.

| Subcommand | Purpose |
|---|---|
| `topcoat ui init` | Create the package's UI install state. Run this before adding components. |
| `topcoat ui add` | Copy premade component source into the package. |
| `topcoat ui list` | List registry components and their installation status. |
| `topcoat ui remove` | Remove installed component source tracked by the package. |
| `topcoat ui help` | Print UI help or help for a UI subcommand. |

These commands operate on source you own. After `add`, inspect and edit the copied Rust modules like
any other project code.

There is no `topcoat ui update` command in the published CLI.
Refresh deliberately with `topcoat ui add --overwrite`, selecting the package and registry.
`topcoat ui add --all` adds every registry component and skips installed files by default; with
`--overwrite` it replaces them too. Use `--all` in a disposable package, and inspect any overwrite
before accepting it. Preserve local component customizations and merge theme tokens separately.

### `topcoat ui init`

Usage is `topcoat ui init [OPTIONS]`.

| Flag | Value | Meaning |
|---|---|---|
| `-c`, `--components-dir` | `<COMPONENTS_DIR>` | Set the component output directory. The default is `src/components`. |
| `-t`, `--theme` | `<THEME>` | Install the named theme. If omitted, the sole theme is used or you are prompted when several exist. |
| `-p`, `--package` | `<SPEC>` | Select a workspace package by name. Its root owns `components.toml`. The default is the package containing the current directory. |
| `-h`, `--help` | — | Print help. |

### `topcoat ui add`

Usage is `topcoat ui add [OPTIONS] <COMPONENTS>...`.

| Argument or flag | Value | Meaning |
|---|---|---|
| `<COMPONENTS>...` | Names | Add one or more components, such as `button card`. |
| `-r`, `--registry` | `<REGISTRY>` | Select a registry crate. The built-in default registry is used when omitted. |
| `-o`, `--overwrite` | — | Replace an existing component file. Review the diff because local edits are overwritten. |
| `-a`, `--all` | — | Add every component the registry offers, skipping installed components unless combined with `--overwrite`. |
| `-p`, `--package` | `<SPEC>` | Select the package whose root contains `components.toml`. |
| `-h`, `--help` | — | Print help. |

### `topcoat ui list`

Usage is `topcoat ui list [OPTIONS]`.

| Flag | Value | Meaning |
|---|---|---|
| `-r`, `--registry` | `<REGISTRY>` | Limit results to one named registry. |
| `-i`, `--installed` | — | Show only installed components. |
| `-p`, `--package` | `<SPEC>` | Select the package whose root contains `components.toml`. |
| `-h`, `--help` | — | Print help. |

> [!NOTE]
> In the `ui` command group, `-r` means `--registry`. In `dev` and `asset`, `-r` means `--release`.

### `topcoat ui remove`

Usage is `topcoat ui remove [OPTIONS] <COMPONENTS>...`.

| Argument or flag | Value | Meaning |
|---|---|---|
| `<COMPONENTS>...` | Names | Remove one or more installed components. |
| `-r`, `--registry` | `<REGISTRY>` | Specify the registry they came from. All registries are searched when omitted. |
| `-p`, `--package` | `<SPEC>` | Select the package whose root contains `components.toml`. |
| `-h`, `--help` | — | Print help. |

## Help dispatch

`topcoat help` prints the same top-level command list as `topcoat --help`. The nested forms
`topcoat asset help` and `topcoat ui help` print their group help. Add a nested command name after
`help` to request that command's help.

The normal `--help` form is available on every working command listed above. `topcoat --version`
prints the CLI release.

## Missing commands and flags in 0.10.0

The `Result` column quotes the 0.10.0 binary verbatim.

| Attempt | Result | Use instead |
|---|---|---|
| `topcoat build` | `error: unrecognized subcommand 'build'`, with `tip: a similar subcommand exists: 'ui'` | Run `cargo build`, followed by `topcoat asset bundle` when assets are used. The suggested `ui` command is unrelated. |
| `topcoat ui update` | `error: unrecognized subcommand 'update'` | Use `topcoat ui add --overwrite --package PACKAGE --registry topcoat COMPONENT`, then review owned changes. |

**See also:** [Lab 02 — The `view!` macro](../02-labs/lab-02.md),
[Lab 11 — Assets, Tailwind, and Topcoat UI](../02-labs/lab-11.md),
[Lab 12 — Build, containerise, and deploy](../02-labs/lab-12.md),
[`view!` cheat sheet](view-cheatsheet.md), and
[How to pin and upgrade Topcoat](howto/upgrade.md).
