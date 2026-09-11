# Welcome

This is the documentation for the Topcoat Workshop: thirteen hands-on labs that build one real
application with [Topcoat](https://github.com/tokio-rs/topcoat), the server-rendered full-stack web
framework from tokio-rs.

You read this book at
[penrynjohnp.github.io/Topcoat-Workshop](https://penrynjohnp.github.io/Topcoat-Workshop/) and work
through the code in the [repository](https://github.com/penrynjohnp/Topcoat-Workshop).

## What you build

You build **Slipway**, a small marina and boatyard manager, one capability at a time. It starts as a
static page and finishes as a containerised application deployed to Azure Container Apps with a
database, sessions, authentication, streaming, an asset pipeline, and a mounted Axum API.

Every lab has a `starter/` crate that compiles with `TODO` markers and a `solution/` crate that CI
builds and tests. The finished application also lives in the repository's `slipway/` directory, so
you can compare your work against the end state at any point.

## Who it is for

- **Rust developers new to the web.** You know ownership, traits, and `async`. You do not need to
  know React. Read [HTML/HTTP in 30 min for Rustaceans](../04-appendix/web-for-rustaceans.md) first.
- **Web developers new to Rust.** You know HTML, HTTP, and a framework such as Rails, Next.js, or
  htmx. Read [Rust in 30 min for web devs](../04-appendix/rust-for-web-devs.md) first.
- **Architects evaluating Rust for the web tier.** Start with
  [Why Topcoat](../01-concepts/why-topcoat.md) and
  [Topcoat vs Axum vs Leptos vs Dioxus](../03-reference/when-to-use-topcoat.md).

## Time per module

| Module | Labs | Time |
|---|---|---|
| 0 Orientation | — | 45 min |
| 1 Foundations | [01](../02-labs/lab-01.md), [02](../02-labs/lab-02.md), [03](../02-labs/lab-03.md) | 2¼ h |
| 2 Routing and the request | [04](../02-labs/lab-04.md), [05](../02-labs/lab-05.md), [06](../02-labs/lab-06.md) | 3¼ h |
| 3 Reactivity | [07](../02-labs/lab-07.md), [08](../02-labs/lab-08.md), [09](../02-labs/lab-09.md) | 3 h |
| 4 Data with Toasty | [10](../02-labs/lab-10.md) | 1½ h |
| 5 Assets, styling, UI | [11](../02-labs/lab-11.md) | 1¼ h |
| 6 Production on Azure | [12](../02-labs/lab-12.md), [13](../02-labs/lab-13.md) | 2¾ h |

Labs 01–12 take about 13¼ hours of hands-on time, or about 14 hours including the optional Lab 13.
That fits a two-day instructor-led workshop, or a self-paced course of roughly one module per week.
Each lab is independently resumable: if you skip one, copy the previous lab's `solution/` and carry
on.

## Start here

1. Work through [Setup](setup.md) for your platform — Linux, macOS, Windows with WSL, a devcontainer,
   or a GitHub Codespace.
2. Read [Why Topcoat](../01-concepts/why-topcoat.md) and
   [Request lifecycle](../01-concepts/request-lifecycle.md). That is Module 0, and it is 45 minutes
   well spent before you write any code.
3. Start [Lab 01 — Hello, Topcoat](../02-labs/lab-01.md) and continue in order.

## How to use this book

The book is organised by what you need at the time, not by reading order:

| Section | Use it |
|---|---|
| **Concepts** | before or alongside each module, to understand *why* Topcoat works the way it does |
| **Labs** | as the tutorial itself; each page mirrors that lab's `README.md` in the repository |
| **Reference** | while building — the `view!` cheat sheet, the `Cx` API, routing conventions, CLI flags |
| **How-to** | when you have a specific task, such as deploying to Azure or upgrading the pins |
| **Appendix** | to fill a gap: Rust, the web platform, AI-assisted learning, or troubleshooting |

> [!WARNING]
> Topcoat is pre-1.0 and its API changes between minor versions. Every lab is pinned and tested
> against the versions in
> [`COMPATIBILITY.md`](https://github.com/penrynjohnp/Topcoat-Workshop/blob/main/COMPATIBILITY.md).
> When you read upstream Topcoat documentation, read it at the release tag, never at `main`.

If something does not behave as the lab describes, check
[Troubleshooting](../04-appendix/troubleshooting.md) first. It collects every lab's troubleshooting
entries in one page, grouped by symptom.

**See also:** [Setup](setup.md), [Lab 01](../02-labs/lab-01.md),
[Why Topcoat](../01-concepts/why-topcoat.md),
[Topcoat vs Axum vs Leptos vs Dioxus](../03-reference/when-to-use-topcoat.md), and
[Troubleshooting](../04-appendix/troubleshooting.md).
