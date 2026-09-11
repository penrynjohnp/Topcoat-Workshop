# Topcoat Workshop

Hands-on labs and documentation for learning [Topcoat](https://github.com/tokio-rs/topcoat),
tokio-rs's server-rendered, full-stack Rust web framework.

**📖 Read the book: <https://penrynjohnp.github.io/Topcoat-Workshop/>**

Thirteen labs build one application from a static page to a containerised deployment on Azure
Container Apps. Every `solution/` is compiled and tested in CI against the pinned versions in
[COMPATIBILITY.md](COMPATIBILITY.md) — Topcoat is pre-1.0 and its API changes between minor versions.

## Who this is for

- **Rust developers new to the web** — you know ownership, traits, and `async`; you don't need to
  know React. Start with the [HTML/HTTP appendix](https://penrynjohnp.github.io/Topcoat-Workshop/04-appendix/web-for-rustaceans.html).
- **Web developers new to Rust** — you know HTML, HTTP, and a framework like Rails, Next.js, or htmx;
  you've skimmed the Rust book. Start with the [Rust appendix](https://penrynjohnp.github.io/Topcoat-Workshop/04-appendix/rust-for-web-devs.html).
- **Architects** deciding whether Rust belongs in the web tier — read
  [Why Topcoat](https://penrynjohnp.github.io/Topcoat-Workshop/01-concepts/why-topcoat.html) and the
  [decision guide](https://penrynjohnp.github.io/Topcoat-Workshop/03-reference/when-to-use-topcoat.html).

## What you'll build

One app, **Slipway** (a small marina and boatyard manager), grows across the labs: templating and
components → routing, sessions, and auth → client reactivity with signals, shards, and streaming →
Toasty for the database → assets, Tailwind, and Topcoat UI → containerise and deploy with `azd up` →
mount an Axum API through the tower bridge.

| Module | Labs | Time |
|---|---|---|
| 0 Orientation | — | 45 min |
| 1 Foundations | 01–03 | 2¼ h |
| 2 Routing & the request | 04–06 | 3¼ h |
| 3 Reactivity | 07–09 | 3 h |
| 4 Data with Toasty | 10 | 1½ h |
| 5 Assets, styling, UI | 11 | 1¼ h |
| 6 Production on Azure | 12 (+13) | 2¾ h |

About 13¼ hours of hands-on time for Labs 01–12, or 14 hours including the optional Lab 13. Run it as
a two-day instructor-led workshop, or self-paced at roughly a module a week. Each lab is resumable —
skip one and copy the previous lab's `solution/`.

| # | Lab | Time |
|---|---|---|
| 01 | [Hello, Topcoat](labs/lab-01-hello-topcoat/README.md) | 30 min |
| 02 | [The `view!` macro](labs/lab-02-view-macro/README.md) | 60 min |
| 03 | [Components and composition](labs/lab-03-components/README.md) | 45 min |
| 04 | [Manual routing, then module-based routing](labs/lab-04-routing/README.md) | 60 min |
| 05 | [Cx, app context, and memoization](labs/lab-05-cx-memoize/README.md) | 60 min |
| 06 | [Cookies, sessions, and functions—not middleware](labs/lab-06-auth-sessions-mail/README.md) | 75 min |
| 07 | [Signals and `$(...)` expressions](labs/lab-07-signals-expressions/README.md) | 60 min |
| 08 | [Shards, procedures, and streaming](labs/lab-08-shards-procedures-streaming/README.md) | 75 min |
| 09 | [htmx and Alpine as the pragmatic path](labs/lab-09-htmx-alpine/README.md) | 45 min |
| 10 | [Toasty models and queries](labs/lab-10-toasty-sqlite/README.md) | 90 min |
| 11 | [Asset pipeline, fonts, icons, Tailwind, and Topcoat UI](labs/lab-11-assets-tailwind-ui/README.md) | 75 min |
| 12 | [Build, containerise, and deploy with azd](labs/lab-12-build-containerise-deploy/README.md) | 120 min |
| 13 | [Tower bridge and Axum](labs/lab-13-tower-bridge/README.md) | 45 min |

## Start here

```bash
git clone https://github.com/penrynjohnp/Topcoat-Workshop
cd Topcoat-Workshop
./scripts/setup.sh          # installs the pinned toolchain + topcoat-cli, then verifies
cd labs/lab-01-hello-topcoat && cat README.md
```

Or open the repo in a devcontainer or Codespace — the toolchain, `sqlite3`, `az`, `azd`, Docker, and
mdBook are all preinstalled. Full platform instructions for Linux, macOS, Windows/WSL, devcontainers,
and Codespaces, plus the Azure prerequisites for Lab 12, are in
[Setup](https://penrynjohnp.github.io/Topcoat-Workshop/00-welcome/setup.html).

Read [Why Topcoat](https://penrynjohnp.github.io/Topcoat-Workshop/01-concepts/why-topcoat.html) and
the [request lifecycle](https://penrynjohnp.github.io/Topcoat-Workshop/01-concepts/request-lifecycle.html)
before Lab 01 — that's Module 0.

## Layout

- `labs/` — one directory per lab, each with `README.md`, `starter/` (compiles, has TODOs) and
  `solution/` (built and tested in CI)
- `docs/` — the mdBook source: concepts, lab pages, reference, how-tos, appendix
- `slipway/` — the capstone at its finished state, deployable with `azd up`
- `scripts/` — `setup.sh` and the repository's helper scripts
- [`COMPATIBILITY.md`](COMPATIBILITY.md) — tested version matrix and every known breakage
- [`PLAN.md`](PLAN.md) — the full workshop design and delivery plan
- [`AGENTS.md`](AGENTS.md) — conventions for contributors and AI tools

## Stuck?

[Troubleshooting](https://penrynjohnp.github.io/Topcoat-Workshop/04-appendix/troubleshooting.html)
collects every lab's troubleshooting entries in one page, grouped by symptom. If that doesn't help,
open an issue with the lab number, the exact command, and the full error.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). In short: `cargo fmt`, `topcoat fmt`, and
`cargo clippy --workspace -- -D warnings` must be clean, every `starter/` must compile, and every
`solution/` needs at least one integration test.

## License

MIT — same as Topcoat.
