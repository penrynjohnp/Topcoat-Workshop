# Topcoat Workshop

Hands-on labs and documentation for learning [Topcoat](https://github.com/tokio-rs/topcoat), tokio-rs's server-rendered, full-stack Rust web framework.

> **Status: Phase 0 (skeleton).** Lab 01 is in place; the rest are being built following [PLAN.md](PLAN.md). Topcoat is early-stage and changes fast — see [COMPATIBILITY.md](COMPATIBILITY.md) for the versions every lab is tested against.

## Who this is for

- **Rust developers new to the web** — you know ownership, traits, and `async`; you don't need to know React.
- **Web developers new to Rust** — you know HTML, HTTP, and a framework like Rails/Next/HTMX; you've skimmed the Rust book.
- **Architects** deciding whether Rust belongs in the web tier.

## What you'll build

One app, **Slipway** (a small marina/boatyard manager), grows across 12 labs: templating and components → routing, sessions, and auth → client reactivity with signals, shards, and streaming → Toasty for the database → assets, Tailwind, and Topcoat UI → containerise and deploy to Azure Container Apps with `azd up`.

| Module | Labs | Time |
|---|---|---|
| 0 Orientation | — | 45 min |
| 1 Foundations | 01–03 | 2¼ h |
| 2 Routing & the request | 04–06 | 3¼ h |
| 3 Reactivity | 07–09 | 3 h |
| 4 Data with Toasty | 10 | 1½ h |
| 5 Assets, styling, UI | 11 | 1¼ h |
| 6 Production on Azure | 12 (+13) | 2–2¾ h |

## Start here

```bash
git clone https://github.com/penrynjohnp/Topcoat-Workshop
cd Topcoat-Workshop
./scripts/setup.sh          # installs pinned toolchain + topcoat-cli, verifies
cd labs/lab-01-hello-topcoat && cat README.md
```

Or open the repo in a devcontainer / Codespace — everything is preinstalled.

## Layout

- `labs/` — one directory per lab, each with `README.md`, `starter/` (compiles, has TODOs) and `solution/` (tested in CI)
- `docs/` — mdBook: concepts, lab pages, reference, how-tos
- `slipway/` — the capstone at its finished state
- `PLAN.md` — the full workshop design and delivery plan

## License

MIT — same as Topcoat.
