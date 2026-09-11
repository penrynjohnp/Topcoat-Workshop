# Why Topcoat

Topcoat starts from an organisational question, not a benchmark: if your organisation already uses
Rust, why must its higher-level web applications move to a second language and toolchain?

The answer used to be productivity. JavaScript, Ruby, and PHP have mature web ecosystems and let
teams ship ordinary business applications quickly. Rust offers speed and reliability, but those
advantages rarely justify a slower development loop for a dashboard, booking system, or admin
panel.

The [Topcoat announcement](https://tokio.rs/blog/2026-07-22-announcing-topcoat) argues that AI changes
part of that calculation.

## Rust as a green-field language

“Green-field” means a new system where you are choosing the stack, not rewriting one that already
works. The thesis is that modern coding tools reduce two historical costs of choosing Rust:
learning an unfamiliar language and writing routine application code more slowly.

They do not erase every difference between ecosystems. Libraries, documentation, operational
knowledge, and framework maturity still determine how quickly you can deliver. Topcoat and Toasty
exist to fill that library gap for full-stack web applications.

Nor is the thesis “rewrite everything in Rust.” A working Rails, Laravel, Django, or TypeScript
system already carries years of useful code and team knowledge. Unless you need Rust's properties,
a rewrite exchanges known value for migration risk.

The argument is strongest when your organisation has already adopted Rust for other systems. You
already maintain Rust libraries, build infrastructure, review practices, deployment pipelines, and
engineers who can support the language. Building the next internal tool in Rust may then be simpler
than introducing another stack, even when that tool does not need exceptional performance.

> [!NOTE]
> AI can shorten the learning loop, but it cannot make an immature API stable. Topcoat is pre-1.0,
> experimental, and expected to change. This workshop pins the version it teaches and records known
> constraints in the repository's
> [Compatibility matrix](https://github.com/penrynjohnp/Topcoat-Workshop/blob/main/COMPATIBILITY.md).

## Topcoat chooses the server

Topcoat is a high-level framework for server-rendered web applications. Pages and components run on
the server, so an `async` component can query a database, read the request context, or check a
permission before it returns HTML. Page data does not require a separate JSON API and client data
layer.

Interactivity does not automatically move the application into the browser. Topcoat can evaluate a
restricted, type-checked Rust expression on the server for the first render and translate it to
JavaScript for small client-only interactions. When fresh server data is required, a shard renders a
new HTML fragment on the server and swaps it into the page.

That model removes the WebAssembly target, hydration boundary, and client/server serialisation layer
from applications that do not need a substantial browser runtime. It fits forms, content sites,
admin tools, dashboards, and data-heavy business applications where the server remains the source
of truth.

This section follows the server-rendering and reactivity model in the Topcoat v0.7.0 README. The
mechanics are developed further in [How `$(...)` reaches the browser](dual-expressions.md) and
[Shards, procedures, live regions, htmx](reactivity-options.md).

## Where it sits

Topcoat, Axum, Leptos, and Dioxus all let you write Rust, but they optimise for different shapes of
application.

| Choose | Its centre of gravity | Reach for it when |
|---|---|---|
| **Topcoat** | Server-rendered full-stack web UI with selective client and server reactivity | The server owns page data and you want components, routing, sessions, assets, and UI tooling to work as one application model |
| **Axum** | HTTP routing, extraction, responses, and the Tower service ecosystem | You are building an API, protocol endpoint, or bespoke HTTP service and want explicit control over the request pipeline |
| **Leptos** | Fine-grained reactive web UI, from browser rendering to full-stack SSR and hydration | The browser hosts a richer Rust application and client-side reactivity is central to the product |
| **Dioxus** | A React-like Rust UI model across web, desktop, and mobile, with full-stack support | One UI architecture must span several platforms, or the product behaves more like an application than a server-rendered site |

These are not maturity rankings. They describe where each framework puts the boundary between the
server, browser, HTTP layer, and UI runtime.

### Axum is below the application model

Axum is an ergonomic HTTP routing and request-handling library. It excels at extractors, responses,
and composing Tower middleware. It deliberately does not prescribe a full server-rendered UI,
asset pipeline, component model, or application architecture.

Use Axum directly when HTTP is the product surface. Use Topcoat when the product surface is an HTML
application and you want the framework to remove the wiring around it.

You do not have to choose only one. Topcoat's Tower bridge can mount an Axum router beneath part of
the URL tree or expose a Topcoat router as a Tower service. Slipway uses this in
[Lab 13](../02-labs/lab-13.md): Topcoat renders the application while Axum serves a small JSON API.

### Leptos and Dioxus put more application in the client

Leptos supports client-side rendering and full-stack server-side rendering with hydration. Its
fine-grained reactive model is a better fit when browser state, client-side navigation, and rich UI
updates are central rather than occasional.

Dioxus targets web, desktop, and mobile from one Rust UI model. That cross-platform goal is valuable
when the same product must become a browser app and native application, but it introduces a broader
runtime and tooling model than a server-rendered website needs.

Topcoat chooses a narrower target. It keeps markup and data access on the server, then adds only the
browser behaviour a page needs. If you need an application-sized client runtime or a shared native
UI, choose the framework designed around that requirement rather than stretching Topcoat into it.

## A practical rule

Start with the dominant boundary in your system:

- Choose **Axum** when you are primarily designing HTTP endpoints.
- Choose **Leptos** when you are primarily designing a reactive web client in Rust.
- Choose **Dioxus** when you are primarily designing a cross-platform Rust UI.
- Choose **Topcoat** when you are primarily designing a server-rendered web application and want
  selective reactivity without a separate frontend architecture.

Then combine tools where the boundary changes. A Topcoat application can own the pages while Axum
owns an API subtree. The useful question is not “which Rust framework wins?” but “where should this
part of the application run, and how much framework should sit around it?”

**See also:** [Lab 01 — Hello, Topcoat](../02-labs/lab-01.md),
[Lab 13 — Tower bridge and Axum](../02-labs/lab-13.md),
[Request lifecycle](request-lifecycle.md), and
[Topcoat vs Axum vs Leptos vs Dioxus](../03-reference/when-to-use-topcoat.md).
