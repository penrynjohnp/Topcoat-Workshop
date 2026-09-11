# Topcoat vs Axum vs Leptos vs Dioxus

These projects solve different problems at different layers. Choose from the rendering and deployment
model your product needs, then consider maturity and stewardship alongside those technical facts.

This comparison is a snapshot dated **11 September 2026**. Topcoat examples elsewhere in this
workshop remain pinned to v0.7.0 even when a newer release exists.

## Technical comparison

| Dimension | Topcoat | Axum | Leptos | Dioxus |
|---|---|---|---|---|
| **Rendering model** | Server-rendered HTML. Async components can read the request and database directly. | HTTP routing and request handling, not a UI rendering model. You choose templates, JSON, streaming, or another frontend. | Full-stack, isomorphic web UI with CSR, SSR, hydration, and streamed HTML. | Virtual-DOM UI for web, desktop, mobile, and other renderers. Full-stack web supports SSR followed by hydration. |
| **Interactivity** | Small client interactions use signals and Rust expressions translated to JavaScript, without WebAssembly. Shards re-render HTML on the server; procedures call server functions. | Not prescribed. Add a browser framework, server-rendered fragments, WebSockets, or another interaction model. | Fine-grained browser reactivity compiled to WebAssembly. Server functions cross the client/server boundary with shared Rust types. | Component-scoped signals and virtual-DOM updates. Web builds use WebAssembly; full-stack mode adds server functions, SSR, and hydration. |
| **Concepts to learn** | `view!`, async components, `Cx`, module routing, and a restricted dual Rust/JavaScript expression vocabulary. | Routers, handlers, extractors, responses, shared state, and Tower middleware. UI, assets, sessions, and application structure come from your chosen crates. | Fine-grained signals, reactive ownership, resources, server functions, SSR/hydration boundaries, and browser/server feature selection. | `rsx!`, components, hooks/signals, virtual-DOM rendering, the `dx` toolchain, and platform/full-stack target selection. |
| **Deployment shape** | One server binary plus an asset bundle produced from the same build profile. A running server is required. | One HTTP server binary, plus whatever templates, static assets, or separately deployed frontend you choose. | CSR can deploy as static WebAssembly assets. Full-stack SSR normally deploys a server binary plus browser WebAssembly and assets built together. | Varies by target. Web may be static CSR or full-stack SSR/hydration; desktop and mobile produce native application bundles through `dx`. |
| **Scope supplied** | Pages, layouts, components, routing, request context, cookies, sessions, assets, UI source, mail, streaming, and selective reactivity. | A deliberately thin HTTP layer over Hyper with Tower interoperability. | A web UI framework with routing, reactive primitives, server functions, streaming, and hydration. | A cross-platform UI and full-stack toolchain, including routing, server functions, assets, and platform bundling. |
| **Maturity signal** | Its README calls it early-stage and experimental and says to expect breaking changes. | Pre-1.0, with an established 0.8 release line; its repository says work toward 0.9 introduces breaking changes on `main`. | Pre-1.0; its README says the APIs are basically settled while acknowledging bugs and that adopters may need to contribute missing pieces. | Pre-1.0; the stable line was 0.7.10 and 0.8 was in alpha when this snapshot was checked. It supports several platforms, which gives it a broader compatibility surface. |

Sources for the rendering models are the [Topcoat v0.7.0
README](https://github.com/tokio-rs/topcoat/blob/v0.7.0/README.md), [Axum crate
documentation](https://docs.rs/axum/latest/axum/), [Leptos
README](https://github.com/leptos-rs/leptos), and [Dioxus
README](https://github.com/DioxusLabs/dioxus) plus its [full-stack
guide](https://dioxuslabs.com/learn/0.7/essentials/fullstack/).

## Where each framework places the boundary

### Topcoat: HTML application on the server

Topcoat keeps markup generation, data access, and request policy on the server. The browser receives
HTML. It runs a small JavaScript runtime only when the page uses signals, shards, or procedures.
There is no WebAssembly target or hydration step in this model.

This fits applications where the server owns page data and most interactions are forms, navigation,
small local state changes, or server-rendered partial updates. The framework also supplies application
facilities that Axum leaves for you to assemble.

The cost is the early-stage API and a narrower browser model. Topcoat's runtime expression language
supports only a subset of Rust, and the framework does not aim to become a client-side Rust
application runtime.

### Axum: HTTP is the framework boundary

Axum routes requests to async handlers, extracts typed request data, converts values into responses,
and composes Tower services. It does not decide whether your product renders HTML on the server,
serves a JSON API, or supports a separately built frontend.

This fits APIs, protocol endpoints, and services where explicit HTTP control is the main requirement.
For an HTML application, you choose and integrate templating, asset, form, session, and interaction
libraries yourself.

Axum and Topcoat are not exclusive. Topcoat's Tower bridge can mount an Axum router for an API subtree
while Topcoat owns the HTML application.

### Leptos: reactive web application across server and browser

Leptos uses fine-grained reactivity and supports browser-only CSR, server rendering with hydration,
and full-stack server functions. A signal can update a specific DOM node without re-running a whole
component or diffing a virtual DOM.

This fits web products where substantial state and interaction live in the browser, but you still
want shared Rust types and colocated server functions. Full-stack deployment includes both server and
browser build concerns; CSR can be deployed as static assets.

The additional concepts are the client/server compilation boundary, hydration, WebAssembly packaging,
and reactive ownership. Those costs buy a richer browser-resident Rust application than Topcoat is
designed to provide.

### Dioxus: one UI model across several platforms

Dioxus uses a React-like component and virtual-DOM model across web, desktop, mobile, and other
renderers. Its full-stack web mode integrates with Axum for server functions, SSR, streaming,
WebSockets, and hydration.

This fits products where sharing one Rust UI architecture across browser and native targets is an
explicit requirement. It also fits teams that prefer component re-rendering and virtual-DOM diffing
to Leptos's fine-grained model.

A web-only team still takes on concepts and tooling designed for a broader platform set. That is a
scope difference, not a quality ranking.

## Stewardship and stated direction

Stewardship changes who reviews work, where priorities are discussed, and which use cases receive
maintainer time. It does not by itself determine whether a framework fits your application.

| Project | Maintainer and organisation | Dated status | Publicly stated investment or direction |
|---|---|---|---|
| **Topcoat** | Maintained as a [`tokio-rs` project](https://github.com/tokio-rs/topcoat), announced on the [Tokio site](https://tokio.rs/blog/2026-07-22-announcing-topcoat). | **22 July 2026:** first public announcement. The v0.7.0 README describes the project as early-stage and experimental, with breaking changes expected. | Its [published roadmap](https://github.com/tokio-rs/topcoat/blob/v0.7.0/README.md#roadmap) lists more reactivity, Topcoat UI, Toasty integration, validation, authentication, static export, deployment documentation, and other full-stack facilities. |
| **Axum** | Maintained under [`tokio-rs`](https://github.com/tokio-rs/axum), the same GitHub organisation as Tokio and Topcoat. | **11 September 2026 snapshot:** crates.io's released line is 0.8; the repository states that `main` contains breaking work toward Axum 0.9. | Axum's stated centre remains ergonomic, modular HTTP routing and request handling with [Tower ecosystem](https://docs.rs/axum/latest/axum/#high-level-features) interoperability. The repository identifies Axum 0.9 as the current release direction rather than publishing a broader product roadmap. |
| **Leptos** | Community-maintained in the [`leptos-rs` organisation](https://github.com/leptos-rs/leptos). Its book directs users to community discussions, issues, Discord, and the [community crate ecosystem](https://book.leptos.dev/getting_started/community_crates.html). | **11 September 2026 snapshot:** the stable crate line is 0.8 and a 0.9 beta is published. The README says its APIs are basically settled, while describing production adoption as a relationship that may still include contributing missing pieces. | Its public project description continues to focus on full-stack isomorphic web development, fine-grained reactivity, SSR/hydration, streamed HTML, and server functions. Priorities are discussed through the repository and community channels rather than a single published roadmap. |
| **Dioxus** | On **10 September 2026**, the [Dioxus team announced that it had joined Cognition](https://dioxuslabs.com/blog/joining-cognition). The post says the team continues to maintain Dioxus and its related open-source projects, with **one full-time engineer**, lead Blitz engineer Nico Burns, devoted to maintaining and improving Dioxus. | **10 September 2026:** the announcement says joining Cognition leaves the team less time devoted entirely to Dioxus, while Cognition supports continued open-source work. | The same announcement says future investment is weighted toward **Dioxus-Native and Blitz**, its native HTML/CSS renderer, rather than the web target. It also names Blitz, Taffy, Parley, Subsecond, `wasm-split`, and DX as work intended to benefit users beyond Dioxus itself. |

The Dioxus statement describes an allocation of maintainer effort. It does not say that the web target
is discontinued. Likewise, a published roadmap records intent rather than a delivery guarantee. Use
these facts as one input alongside platform fit, current releases, issue history, and your team's
ability to maintain application code.

## Which one, when

| Typical scenario | Start with | Why |
|---|---|---|
| Server-rendered admin tool, dashboard, content site, or line-of-business application | **Topcoat** | Page data stays on the server, while the framework supplies components and selective reactivity without a separate browser application. |
| JSON API, webhook receiver, protocol service, or bespoke HTTP pipeline | **Axum** | HTTP routing, extraction, responses, and Tower middleware are the centre of the design. |
| Highly interactive web application that should run substantial Rust logic in the browser | **Leptos** | Fine-grained WebAssembly reactivity and full-stack server functions are primary capabilities. |
| Product sharing a Rust UI across web, desktop, and mobile | **Dioxus** | Cross-platform rendering and bundling are part of the framework's core scope. |
| Server-rendered application with a specialised API subtree | **Topcoat + Axum** | Topcoat can own HTML pages while Axum owns lower-level endpoints through the Tower bridge. |
| Existing working application on another stack with no Rust-specific requirement | **Keep the existing stack** | Migration cost and operational knowledge are part of the decision; none of these frameworks makes a rewrite automatically worthwhile. |

> [!NOTE]
> This page is dated **11 September 2026**. Maintainers, organisations, roadmaps, release maturity, and
> investment priorities can change. Re-check the linked primary sources before relying on this
> comparison for a project decision.

**See also:** [Why Topcoat](../01-concepts/why-topcoat.md),
[Lab 01 — Hello, Topcoat](../02-labs/lab-01.md),
[Lab 13 — Tower bridge and Axum](../02-labs/lab-13.md),
[Request lifecycle](../01-concepts/request-lifecycle.md), and
[Shards, procedures, live regions, htmx](../01-concepts/reactivity-options.md).
