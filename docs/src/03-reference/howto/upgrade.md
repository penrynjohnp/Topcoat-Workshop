# How-to: pin and upgrade Topcoat

Topcoat is pre-1.0, so an upgrade is a compatibility exercise, not only a version edit. You update the workspace pins, the CLI and documentation toolchain, run the drift check, then record what actually passes in [`COMPATIBILITY.md`](https://github.com/penrynjohnp/Topcoat-Workshop/blob/main/COMPATIBILITY.md).

> [!WARNING]
> Read the guides from the release tag that you are testing, such as [`v0.8.0`](https://github.com/tokio-rs/topcoat/tree/v0.8.0), not `main`. The unreleased API can already differ from the version in this workshop.

## 1. Establish a clean baseline

Start from a clean working tree and read the current compatibility row before changing anything. Keep the old row available until the new versions pass:

```bash
git status --short
sed -n '1,40p' COMPATIBILITY.md
```

Record the versions you intend to test. The Rust version comes from the active toolchain; the crate versions come from the workspace manifest; the CLI and mdBook versions are pinned in setup and workflow files.

## 2. Change every pin

Update the exact versions in [`Cargo.toml`](https://github.com/penrynjohnp/Topcoat-Workshop/blob/main/Cargo.toml), under `[workspace.dependencies]`. In this repository, the Topcoat family includes `topcoat` and `topcoat-asset`; Toasty is pinned there as `toasty`. Do not add a version to an individual lab crate.

Update the CLI pin in all installation paths:

- [`scripts/setup.sh`](https://github.com/penrynjohnp/Topcoat-Workshop/blob/main/scripts/setup.sh) installs `topcoat-cli` for learners.
- [`.github/workflows/ci.yml`](https://github.com/penrynjohnp/Topcoat-Workshop/blob/main/.github/workflows/ci.yml) installs the same CLI for formatting checks.
- [`.devcontainer/devcontainer.json`](https://github.com/penrynjohnp/Topcoat-Workshop/blob/main/.devcontainer/devcontainer.json) installs it when the container is created.

Use a repository search to find stale copies rather than assuming these are the only locations:

```bash
grep -RIn --exclude-dir=.git --exclude-dir=target \
  -E 'topcoat-cli|topcoat =|topcoat-asset|toasty =|mdbook@|mdbook-mermaid@|mdbook-linkcheck2@' \
  Cargo.toml scripts .github .devcontainer COMPATIBILITY.md
```

The docs toolchain is a set. Change `mdbook`, `mdbook-mermaid`, and `mdbook-linkcheck2` together in [`docs.yml`](https://github.com/penrynjohnp/Topcoat-Workshop/blob/main/.github/workflows/docs.yml) and the devcontainer. Keep [`docs/book.toml`](https://github.com/penrynjohnp/Topcoat-Workshop/blob/main/docs/book.toml) compatible with that set. For example, mdBook 0.5 uses its built-in GitHub alert syntax and this book uses the `linkcheck2` output backend; do not reintroduce an older admonition or linkcheck setup without testing the whole set.

Install the new tools locally, then refresh the lockfile after changing the Rust pins:

```bash
cargo install topcoat-cli --version X.Y.Z --locked --force
cargo install mdbook@A.B.C mdbook-mermaid@D.E.F mdbook-linkcheck2@G.H.I --locked
cargo update
```

Replace the placeholder versions with the versions under test. Keep `Cargo.lock` in the change because it records the resolved graph that you validated.

## 3. Run the pinned-version checks

Run the same checks as CI, in an order that exposes inexpensive failures first:

```bash
cargo fmt --all --check
topcoat fmt $(git ls-files 'labs/**/*.rs' 'slipway/**/*.rs')
git diff --exit-code -- '*.rs'
cargo clippy --workspace -- -D warnings
cargo build --workspace
cargo test --workspace
```

`topcoat fmt` has no `--check` flag in the pinned CLI. It formats in place, so the following `git diff` is the check. If the formatter changes source, inspect the diff and keep the formatting change only when it is part of the upgrade.

If you ran `cargo clean`, stage the icon cache before parallel workspace builds, as CI does:

```bash
find target/topcoat/cache -type f -empty -delete 2>/dev/null || true
cargo build -p lab11-solution
```

For asset-bearing labs and Slipway, build and bundle with the same profile. A release binary must use the release bundle at `target/release/assets/`:

```bash
cargo build --workspace --release
topcoat asset bundle --release
```

## 4. Run the upstream drift workflow

The weekly [drift workflow](https://github.com/penrynjohnp/Topcoat-Workshop/blob/main/.github/workflows/drift.yml) tests the workspace against the latest published Topcoat crates instead of the exact pin. It changes only the CI checkout: it removes the exact `topcoat` requirement, runs `cargo update`, prints the resolved Topcoat version, and runs the workspace build and tests. It does not test a new Toasty pin or replace the versions in your branch.

Dispatch it from GitHub Actions after the local checks pass, or use `gh`:

```bash
gh workflow run drift.yml
gh run list --workflow drift.yml --limit 1
```

Open the run and inspect the `cargo tree -p topcoat --depth 0` output. A successful drift run tells you that the current labs also build against the latest resolved Topcoat. A failed run creates a `drift` issue with the run link; use that issue to identify the first breaking API before deciding whether to upgrade the pinned workshop version.

> [!NOTE]
> Drift is a signal, not the compatibility record. The pinned version still needs the full lab, formatting, clippy, asset, and documentation checks below.

## 5. Build the book and check links/includes

Build the book with the new mdBook set and make sure the generated HTML contains no unresolved include directives:

```bash
mdbook build docs 2>&1 | tee build.log
! grep -q ' ERROR ' build.log
needle='{{'\'#include'
! grep -rl "$needle" docs/book/html
rm -rf docs/book build.log
```

If you installed `mdbook-linkcheck2`, run the configured output as part of the build and fix broken links before recording the new row. Keep generated `docs/book/` out of the commit.

## 6. Update `COMPATIBILITY.md`

Add a new dated row only after the checks pass. Record:

- the tested `topcoat`, `topcoat-cli`, Toasty, and Rust versions;
- the mdBook, Mermaid, and linkcheck2 versions in the docs-toolchain table;
- the labs and capstone that passed;
- API changes, workarounds, feature flags, asset constraints, or known limitations.

Keep notes concrete and reproducible. For example, record when a guide must be read from a release tag, when a CLI flag is absent, or when a generated asset must be bundled from the same profile as its binary. If an upgrade leaves a known limitation, state it rather than implying that the version is production-ready.

Finally, review the complete change and verify that the two instruction files remain identical:

```bash
git diff --check
diff -q AGENTS.md .github/copilot-instructions.md
git status --short
```

Commit the manifest, lockfile, toolchain pins, compatibility row, and any lab fixes together so a reader can reproduce the tested set.

**See also:** [CLI commands](../cli.md), [How-to: deploy to Azure Container Apps](deploy-azure.md), [How-to: SQLite to PostgreSQL](postgres.md), [Lab 12](../../02-labs/lab-12.md), and [Lab 13](../../02-labs/lab-13.md).
