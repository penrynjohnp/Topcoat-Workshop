# Topcoat Workshop — Build Plan

**Target repo:** `penrynjohnp/Topcoat-Workshop`
**Subject:** [Topcoat](https://github.com/tokio-rs/topcoat) — tokio-rs's server-rendered, full-stack Rust web framework (announced 22 July 2026)
**Plan date:** 8 September 2026 · **Pinned:** topcoat 0.7.0, topcoat-cli 0.7.0, toasty 0.10.0, Rust stable

---

## 1. Framing decisions (read before building anything)

### 1.1 What Topcoat is, in one paragraph

Topcoat renders everything on the server. Components are ordinary `async fn`s that can hit the database directly. Reactivity comes from `$(...)` expressions — a type-checked subset of Rust that Topcoat evaluates on the server for first render and *also* cross-compiles to JavaScript so it re-runs in the browser without a round-trip. When the server *is* needed, a `#[shard]` component re-renders on the server and swaps its HTML in place. No WebAssembly, no separate client build, no serialisation boundary. Think "HTMX with a type checker and a component model," sitting on tokio, with Toasty as the intended ORM.

### 1.2 Who this workshop is for

| Persona | Assumed | Not assumed |
|---|---|---|
| Rust developer new to web | Rust ownership, traits, `async`/tokio basics, cargo | HTML/CSS beyond basics, web frameworks |
| Web developer new to Rust | HTML, HTTP, a React/Rails/HTMX mental model | Rust beyond "I've read the book's first half" |
| Architect evaluating Rust for web | Both, at a reading level | Wants to know *when* to use Topcoat vs Axum/Leptos/Dioxus |

Every lab should be completable by persona 1 and 2. Module 0 carries a "Rust in 30 minutes for web devs" and an "HTTP/HTML in 30 minutes for Rust devs" appendix so neither group is stranded.

### 1.3 The four risks specific to Topcoat and how the workshop handles them

1. **It's early-stage and breaking.** The README says so in bold. The workshop **pins exact crate versions** in every lab's `Cargo.toml`, records the tested `rust-toolchain.toml`, and ships a `COMPATIBILITY.md` with a tested-version matrix. A CI job runs all lab solutions weekly against pinned versions *and* against `latest`, so drift is detected rather than discovered by a learner.
2. **The reactivity runtime is admittedly incomplete.** The workshop teaches it honestly: one module on native `$(...)`/signals/shards, and one on the htmx and Alpine integrations as the pragmatic fallback. Learners come away knowing which to reach for.
3. **Deployment docs don't exist yet** (it's on the roadmap). We write them — that's a differentiator, and the Azure Container Apps path is one you can author authoritatively.
4. **No `topcoat new` yet.** Lab 1 covers manual scaffolding; a `scripts/new-lab.sh` template generator stands in for it.

### 1.4 Upstream has moved since the announcement (checked 8 Sep 2026)

`topcoat` 0.7.0 shipped on 5 September. Since the July post, several roadmap items have landed: `topcoat-mail` (SMTP/file/in-memory transports), `live!`/`emit!` streaming regions with `suspense` and `error_boundary` components, WebSockets, server-sent events, multipart uploads, sitemaps, a Datastar integration alongside htmx/Alpine, a tower bridge (mount tower services and layers — the Axum story), and a `Props` derive. Emailing, streaming SSR, WebSockets and SSE are gone from the roadmap; Localization, WebTransport and Markdown are new on it. The curriculum below folds these in (mail in Lab 06, `live!`/suspense in Lab 08, tower bridge in Lab 13). Re-run this check before starting each phase.

### 1.5 The capstone app: "Slipway"

One app grows across all modules rather than isolated toy examples. **Slipway** is a small marina/boatyard management app — berths, vessels, work orders, a public "what's on" page. It's domain-neutral enough to teach from, rich enough to need auth, sessions, forms, search, lists, and a database, and it gives every lab a reason to exist. (If you'd rather not show your hobbies to the field, a generic "workshop bookings" app is a drop-in swap; the lab structure doesn't change.)

---

## 2. Repository layout

```
Topcoat-Workshop/
├── README.md                     # landing page: what, who, how long, start here
├── COMPATIBILITY.md              # tested versions matrix, known breakages
├── CONTRIBUTING.md
├── LICENSE                       # MIT, matching Topcoat
├── AGENTS.md                     # Copilot/Claude guidance for the repo (see §6)
├── .github/
│   ├── workflows/
│   │   ├── ci.yml                # build + test every lab solution, pinned versions
│   │   ├── drift.yml             # weekly: rebuild against latest topcoat, open issue on failure
│   │   └── docs.yml              # build & publish mdBook to GitHub Pages
│   ├── ISSUE_TEMPLATE/lab-bug.yml
│   └── copilot-instructions.md
├── .devcontainer/
│   └── devcontainer.json         # Rust toolchain + topcoat-cli + sqlite preinstalled
├── docs/                         # mdBook source — the "excellent documentation"
│   ├── book.toml
│   └── src/
│       ├── SUMMARY.md
│       ├── 00-welcome/
│       ├── 01-concepts/          # explainers, read before or alongside labs
│       ├── 02-labs/              # one page per lab, mirrors labs/
│       ├── 03-reference/         # cheat sheets, decision guides, glossary
│       └── 04-appendix/          # Rust-for-web-devs, HTML-for-Rustaceans, troubleshooting
├── labs/
│   ├── lab-01-hello-topcoat/
│   │   ├── README.md             # objectives, prereqs, steps, checkpoints, stretch goals
│   │   ├── starter/              # compiles, has TODOs
│   │   └── solution/             # complete, tested by CI
│   ├── lab-02-.../
│   └── ...
├── slipway/                      # the capstone at its final state (= lab-12 solution)
├── slides/                       # per-module decks (markdown/Marp, exportable to pptx)
├── scripts/
│   ├── setup.sh                  # install toolchain, topcoat-cli, verify
│   ├── check-lab.sh              # build+test one lab's starter and solution
│   └── new-lab.sh                # scaffold a lab directory from template
└── Cargo.toml                    # workspace over all starter/ and solution/ crates
```

**Why a Cargo workspace over every lab:** one `cargo build --workspace` proves everything compiles; one `Cargo.lock` pins everything; `rust-analyzer` works out of the box for a learner who opens the repo root.

---

## 3. Curriculum

Total: **12 labs across 6 modules**, roughly 10–12 hours of hands-on time. Designed to run as a 2-day instructor-led workshop or a self-paced course. Each module has a concept page in `docs/`, one or more labs, and a short deck.

### Module 0 — Orientation (45 min, no lab)

- Why Topcoat exists: the "Rust as a green-field language" thesis from the announcement, and the organisational argument (you're already on Rust; stay on Rust for the web tier too).
- Where it sits: Axum (low-level HTTP, use it for APIs — Topcoat expects you'll use both), Leptos/Dioxus (WASM, full client interactivity), Topcoat (server-rendered, sprinkled reactivity). Decision table in `docs/03-reference/when-to-use-topcoat.md`.
- The mental model diagram: request → `Cx` → page/layout → components → `view!` → HTML + reactive metadata → browser runtime.
- Environment check: `scripts/setup.sh` (rustup, pinned toolchain, `cargo install topcoat-cli`, sqlite3), or open in the devcontainer.

### Module 1 — Foundations

**Lab 01 — Hello, Topcoat** (30 min)
`cargo new`, add `topcoat` + `tokio`, the hello-world from the getting-started guide, `topcoat dev`, hot reload via `topcoat::dev::script()`, `HOST`/`PORT`. Checkpoint: change the greeting, see it reload without restarting.

**Lab 02 — The `view!` macro** (60 min)
HTML-faithful templating; `(expr)` interpolation; `for`/`if`/`match` inside markup; conditional attributes (`if item.url == current_path { class="active" }`); the `class!` and `attributes!` macros; `topcoat fmt`. Build Slipway's static shell: nav, footer, a berth list rendered from a `Vec<Berth>` literal. Checkpoint: nav highlights the current page.

**Lab 03 — Components and composition** (45 min)
`#[component]` on async fns, arguments, child content, `attrs:` pass-through. Refactor Lab 02 into `layout`, `nav`, `berth_card`. Explore what "locality of behaviour" means in practice: the component fetches its own data rather than receiving it as props. Checkpoint: `berth_card` renders identically whether used on the home page or the berth detail page.

### Module 2 — Routing and the request

**Lab 04 — Manual routing, then module-based routing** (60 min)
Start with explicit `Router::builder()` pages and layouts. Then switch to `.discover()` and restructure `src/` into the module tree convention (`app.rs`, `app/about.rs`, `_marketing.rs` layouts, `posts/id.rs` → `/posts/{post_id}`, `api/health.rs`). Path parameters, nested layouts, an API route returning JSON. Checkpoint: `/berths/{id}` and `GET /api/health` both work with no manual router entries.

**Lab 05 — `Cx`, app context, and memoization** (60 min)
The request context `Cx` as the thing everything reads from; app context for long-lived state keyed by type (a config struct, a DB pool); `#[memoize]` to dedupe fan-out — prove it by adding a counter to `load_berth` and rendering the same berth in three components on one page. Checkpoint: counter reads 1.

**Lab 06 — Cookies, sessions, and "functions, not middleware"** (75 min)
Signed and encrypted cookies; the session API with bring-your-own storage (start with an in-memory map, swap to SQLite in Module 4); login/logout lifecycle, sliding expiry, token rotation. Send a "magic link" login email with `mail!` using the file transport locally. Implement `require_auth(cx)` exactly as the announcement shows it — a plain async fn that returns a redirect error — and use it inside components rather than as router middleware. Discussion page: why this composes better than middleware, and what you give up. Checkpoint: `/admin` redirects to `/login` unauthenticated; the work-orders component protects itself even when embedded on a public page.

### Module 3 — Reactivity

**Lab 07 — Signals and `$(...)` expressions** (60 min)
`signal` declarations, `@click`/`@input` handlers, `:hidden`/`:class` bind attributes, what the dual Rust/JS expression vocabulary does and does not support (deliberately try something unsupported and read the compile error). Build a berth filter toggle and a collapsible work-order panel with zero server calls. Checkpoint: DevTools network tab shows no requests during interaction.

**Lab 08 — Shards and procedures** (75 min)
`#[shard]` for server re-render on argument change: live search over vessels as the user types. `#[procedure]` for async server functions called from the browser: "mark work order complete." Observe the HTML swap in DevTools. Discuss request storms and debouncing. Finish with `live!`/`emit!`: wrap the slow work-order history in `suspense` so the page streams in, and add an `error_boundary`. Checkpoint: typing "Lady" narrows the list without a page load.

**Lab 09 — htmx and Alpine as the pragmatic path** (45 min)
Rebuild one interaction from Lab 08 with the `topcoat::htmx` helpers (request/response header helpers, partial swaps). Side-by-side comparison page in the docs: native runtime vs htmx vs Alpine — when each is the right call given the runtime's current limits. Checkpoint: both implementations coexist in the app.

### Module 4 — Data with Toasty

**Lab 10 — Toasty models and queries** (90 min)
Add Toasty with SQLite. Define `Berth`, `Vessel`, `WorkOrder` models with relations. Replace every in-memory `Vec` in Slipway with real queries inside components. Move session storage to the DB. Forms: a create-work-order page with a POST handler, server-side validation by hand (Topcoat validations are on the roadmap — say so). Checkpoint: data survives a restart; a bad form submission re-renders with errors.

### Module 5 — Assets, styling, and UI

**Lab 11 — Asset pipeline, fonts, icons, Tailwind, Topcoat UI** (75 min)
`asset!` with content-hashed URLs; `fontsource_font!`; `iconify::include!`; the `tailwind` feature and `tailwind::stylesheet!()` with no Node; `topcoat ui` vendoring shadcn-style components into `src/` and editing one to prove you own it. Give Slipway a real look. Checkpoint: `topcoat build` output directory contains hashed assets; changing `card` styling in the vendored file changes every card.

### Module 6 — Production

**Lab 12 — Build, containerise, deploy with `azd`** (120 min)
Release build; multi-stage Dockerfile; asset bundling in the image; env-based config; health endpoint from Lab 04; structured logging with `tracing`. Then `azd init` → `azd up` to **Azure Container Apps** using Bicep-only IaC, managed identity for ACR pull, Log Analytics, and a PostgreSQL Flexible Server swap for SQLite. Full design in **§10**. A second, shorter path for Fly.io or a plain VM so the material isn't Azure-only. Checkpoint: `azd up` prints a public URL that serves Slipway; `azd monitor --logs` streams `tracing` output.

**Optional Lab 13 — Topcoat + Axum via the tower bridge** (45 min)
Use `TowerRoute` to mount an Axum router for a JSON API under `/api/v1` inside the Topcoat router, and `TowerLayer` to apply `tower-http` compression and tracing, sharing the DB pool via app context. Demonstrates the "you'll use both" guidance from the announcement.

---

## 4. Lab README template (every lab follows this exactly)

```markdown
# Lab NN — Title

**Time:** 60 min · **Module:** 2 · **Prerequisites:** Lab 04 complete (or copy `labs/lab-04/solution`)

## What you'll learn
- Three bullets, each a verb phrase ("Read a signed cookie from `Cx`")

## Concepts (read first, 5 min)
Link to docs/01-concepts page. Two-paragraph summary inline so the lab stands alone.

## Steps
### Step 1 — ...
Instruction. Code block with the exact change. Explanation of *why*.
> ✅ **Checkpoint:** what you should see. Command to verify.

### Step 2 — ...

## Stretch goals
- Optional harder extensions with hints, not solutions.

## Troubleshooting
- Symptom → cause → fix. Populated from real learner issues over time.

## What's next
One sentence bridging to the next lab.
```

Rules for lab quality:
- `starter/` always compiles. TODOs are marked `// TODO(lab-05): ...` so `grep` finds them.
- `solution/` has at least one integration test (spin up the router, hit a page, assert on HTML) so CI proves it works, not just that it builds.
- Every checkpoint is verifiable by a command or a specific screen, never "it should feel snappier."
- No step depends on a learner having read the slides.

---

## 5. Documentation strategy

**Tooling:** mdBook, published to GitHub Pages by `docs.yml`. It's the Rust ecosystem's native doc format, learners already know the UI from *The Book*, and it's just markdown so Copilot and PRs work naturally. Add `mdbook-linkcheck` and `mdbook-admonish` (for the ✅/⚠️ callouts).

**Four doc types, kept deliberately separate** (Diátaxis):

| Type | Lives in | Examples |
|---|---|---|
| Tutorials | `labs/*/README.md` + `docs/02-labs/` | The 12 labs |
| Explanations | `docs/01-concepts/` | *How `$(...)` gets to the browser*, *Locality of behaviour vs middleware*, *Shards vs procedures vs htmx*, *What Toasty does that sqlx doesn't* |
| Reference | `docs/03-reference/` | `view!` syntax cheat sheet, `Cx` API summary, module-routing conventions table, CLI commands, Tailwind feature flags, **decision guide: Topcoat vs Axum vs Leptos vs Dioxus** |
| How-to | `docs/03-reference/howto/` | *Deploy to Azure Container Apps*, *Swap SQLite for Postgres*, *Add a health check*, *Pin and upgrade Topcoat* |

**Diagrams:** Mermaid, inline in markdown (mdBook renders it with `mdbook-mermaid`). Required diagrams: request lifecycle; the `$(...)` dual-evaluation flow; shard re-render sequence; module tree → route table mapping; deployment topology.

**Voice:** second person, present tense, short paragraphs, one idea per sentence. Every code block is copy-pasteable and comes from a file that CI compiles (use `{{#include}}` from `labs/*/solution` rather than hand-copying, so docs can't drift from code).

---

## 6. AI-assisted learning layer

Topcoat's own repo ships `AGENTS.md` and `CLAUDE.md`, and the announcement explicitly frames AI tools as how non-Rust engineers will onboard. Lean into that:

- `AGENTS.md` / `.github/copilot-instructions.md` at the repo root describing the lab structure, the pinned versions, and the rule "never suggest `localStorage`-style client state; Topcoat is server-rendered."
- A short `docs/04-appendix/learning-with-copilot.md`: how to ask Copilot to explain a `view!` compile error, how to have it generate a shard from a description, and where it tends to hallucinate (pre-1.0 API surface — always check `docs.rs`).
- Optional: an MCP server config pointing at the Topcoat docs.rs pages so Copilot Chat can ground itself. You've already got the MCP tooling pattern from your Copilot configuration work; reuse it.

---

## 7. CI and maintenance

| Workflow | Trigger | What it does |
|---|---|---|
| `ci.yml` | push, PR | `cargo fmt --check`, `topcoat fmt --check`, `cargo clippy -D warnings`, `cargo test --workspace`, build every `starter/` and `solution/` |
| `drift.yml` | weekly cron | Same build with `cargo update` applied; on failure, opens an issue titled "Topcoat drift: <crate> <old> → <new>" with the error |
| `docs.yml` | push to `main` | `mdbook build`, linkcheck, deploy to Pages |

`COMPATIBILITY.md` gets a row per tested Topcoat/Toasty/toolchain combination with a date. When drift breaks a lab, the fix PR updates that table.

---

## 8. Delivery phases

| Phase | Scope | Effort (evenings/weekends, solo) |
|---|---|---|
| **0 — Skeleton** | Repo layout, workspace, devcontainer, `setup.sh`, CI green on an empty workspace, mdBook publishing a placeholder | 1 weekend |
| **1 — Foundations** | Module 0 docs + Labs 01–03, concept pages, cheat sheet | 1–2 weeks |
| **2 — Routing/request** | Labs 04–06, `Cx`/sessions explainers | 2 weeks |
| **3 — Reactivity** | Labs 07–09, the runtime deep-dive, decision guide | 2 weeks (expect API churn here) |
| **4 — Data + UI** | Labs 10–11, Toasty explainer | 2 weeks |
| **5 — Production** | Lab 12 (+13), Azure Bicep, deployment how-to | 1–2 weeks |
| **6 — Polish** | Slides, drift CI, troubleshooting sections from a dry run with two colleagues, README landing page | 1 week |

Phase 1 is a publishable milestone on its own; announce it in the `#topcoat` Discord channel for early feedback before investing in Phases 3–5, since those are most exposed to breaking changes.

---

## 9. First commit checklist

- [ ] `README.md` with the elevator pitch, personas, time estimate, "Start here → Lab 01" link
- [ ] `rust-toolchain.toml` pinned to whatever tokio-rs/topcoat pins
- [ ] Workspace `Cargo.toml`; `labs/lab-01-hello-topcoat/{starter,solution}` both compiling
- [ ] `scripts/setup.sh` verified on a clean Ubuntu VM and on macOS
- [ ] `.devcontainer/devcontainer.json`
- [ ] `ci.yml` green
- [ ] `docs/` mdBook skeleton with `SUMMARY.md` listing every planned page (empty pages are fine — the table of contents is the contract)
- [ ] `COMPATIBILITY.md` with one row
- [ ] `AGENTS.md`

---

## 10. Deploying the capstone: Azure Developer CLI + Azure Container Apps

This section is the design for Lab 12 and for the `docs/03-reference/howto/deploy-azure.md` how-to. It gives learners a one-command deploy (`azd up`) with nothing hand-clicked in the portal, and it stays consistent with your existing house rules: Bicep-only IaC, managed identity everywhere, no secrets in templates.

### 10.1 Why Container Apps and why azd

Container Apps is the right target for a Topcoat app because Topcoat compiles to a single static-ish binary that serves HTTP on one port and needs nothing else. ACA runs containers with no cluster to manage, scales to zero when idle and back out on HTTP concurrency (the default HTTP rule adds a replica per 10 concurrent requests), gives you managed TLS ingress, and pulls from ACR with a managed identity so there are no registry passwords anywhere. The [Bicep quickstart](https://containerapps.azure.com/docs/containerapps/quickstart/setup-bicep) shows the two core resources — `Microsoft.App/managedEnvironments` and `Microsoft.App/containerApps` (API version `2026-01-01`) — and the new `kind: 'Express'` environment, which is the fastest-provisioning option and ideal for a workshop.

`azd` wraps the whole loop: provision infra (`azd provision`), build and push the image to ACR, deploy the revision (`azd deploy`), stream logs (`azd monitor`), tear it all down (`azd down`). Learners run three commands and never touch `az containerapp` directly, but the Bicep is right there when they want to see what happened.

### 10.2 Repo additions

```
slipway/
├── azure.yaml                    # azd project definition
├── Dockerfile                    # multi-stage, ~30 MB final image
├── .dockerignore
├── infra/
│   ├── main.bicep                # subscription-scope entry point: RG + modules
│   ├── main.parameters.json      # azd injects AZURE_ENV_NAME, AZURE_LOCATION, etc.
│   ├── abbreviations.json        # standard azd naming prefixes
│   └── modules/
│       ├── monitoring.bicep      # Log Analytics + Application Insights
│       ├── registry.bicep        # ACR (Basic), admin user disabled
│       ├── environment.bicep     # Container Apps environment (Express or standard w/ LAW)
│       ├── postgres.bicep        # PostgreSQL Flexible Server, Entra auth only (optional)
│       └── app.bicep             # the container app, MI, AcrPull role assignment
└── .github/workflows/azure-dev.yml   # azd pipeline via OIDC federated credential
```

### 10.3 `azure.yaml`

```yaml
name: slipway
metadata:
  template: topcoat-workshop-slipway@0.1.0
services:
  web:
    project: .
    language: docker          # azd builds the Dockerfile and pushes to ACR
    host: containerapp
    docker:
      path: ./Dockerfile
      context: .
hooks:
  postprovision:
    shell: sh
    run: ./scripts/azd-postprovision.sh   # e.g. run Toasty migrations against the DB
```

`language: docker` is the key line — there is no Rust language handler in azd, and there doesn't need to be; azd just needs a Dockerfile to build and a `containerapp` host to deploy to.

### 10.4 Dockerfile

Two stages. The builder stage caches the dependency layer so a code-only change rebuilds in seconds, runs `topcoat` CLI to bundle assets (fonts, icons, Tailwind output, `asset!` files) into a directory, then copies binary + assets into a minimal runtime image.

```dockerfile
# ---- build ----
FROM rust:1.9x-bookworm AS builder          # pin to rust-toolchain.toml
WORKDIR /app
RUN cargo install topcoat-cli --locked
# dependency cache layer
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main(){}' > src/main.rs && cargo build --release && rm -rf src
COPY . .
RUN topcoat asset bundle --release      # builds, scans the binary, writes target/release/assets/

# ---- runtime ----
FROM gcr.io/distroless/cc-debian12
WORKDIR /app
# AssetBundle::load() reads ./assets next to the executable — keep them together
COPY --from=builder /app/target/release/slipway ./slipway
COPY --from=builder /app/target/release/assets ./assets
ENV HOST=0.0.0.0 PORT=8080 RUST_LOG=info
EXPOSE 8080
USER nonroot
ENTRYPOINT ["./slipway"]
```

Lab note: `HOST=0.0.0.0` matters — the getting-started default binds `127.0.0.1`, which is unreachable from ACA's ingress. This is the number-one "it deployed but 502s" mistake; put it in the troubleshooting section.

### 10.5 Bicep design

`main.bicep` is subscription-scoped (azd convention), creates `rg-${environmentName}`, and calls the modules. Highlights of `app.bicep`, modelled on Example 2 of the ACA Bicep quickstart:

```bicep
resource app 'Microsoft.App/containerApps@2026-01-01' = {
  name: name
  location: location
  tags: union(tags, { 'azd-service-name': 'web' })   // azd matches this to services.web
  identity: { type: 'SystemAssigned' }
  properties: {
    environmentId: environmentId
    configuration: {
      ingress: { external: true, targetPort: 8080, transport: 'auto' }
      registries: [ { server: acrLoginServer, identity: 'system' } ]
      secrets: []                                     // none — DB uses Entra token, cookie key from Key Vault ref
    }
    template: {
      containers: [{
        name: 'web'
        image: image                                  // azd substitutes on deploy
        resources: { cpu: json('0.5'), memory: '1Gi' }
        env: [
          { name: 'HOST', value: '0.0.0.0' }
          { name: 'PORT', value: '8080' }
          { name: 'DATABASE_URL', value: dbConnectionString }
          { name: 'APPLICATIONINSIGHTS_CONNECTION_STRING', value: appInsightsConnStr }
        ]
        probes: [
          { type: 'Liveness',  httpGet: { path: '/api/health', port: 8080 }, periodSeconds: 10 }
          { type: 'Readiness', httpGet: { path: '/api/health', port: 8080 } }
        ]
      }]
      scale: {
        minReplicas: 0
        maxReplicas: 3
        rules: [{ name: 'http', http: { metadata: { concurrentRequests: '20' } } }]
      }
    }
  }
}

resource acrPull 'Microsoft.Authorization/roleAssignments@2022-04-01' = {
  scope: acr
  name: guid(acr.id, app.id, acrPullRoleId)
  properties: {
    principalId: app.identity.principalId
    roleDefinitionId: subscriptionResourceId('Microsoft.Authorization/roleDefinitions', acrPullRoleId)
    principalType: 'ServicePrincipal'
  }
}
```

Design decisions to explain in the docs:

| Decision | Choice | Why |
|---|---|---|
| Environment kind | `Express` for the lab; standard with Log Analytics as the "production" variant | Express provisions fastest; the standard one is what you'd really run |
| Registry auth | System-assigned MI + `AcrPull` role, `identity: 'system'` | No admin user, no password secret — the quickstart's own recommendation |
| Scale | `minReplicas: 0` | Costs nothing between workshop sessions; learners see cold-start behaviour and discuss it |
| Health probes | Reuse `/api/health` from Lab 04 | Ties routing lab to ops; readiness probe prevents traffic before Topcoat has started |
| Database | SQLite on a mounted Azure Files volume for the lab; PostgreSQL Flexible Server with Entra auth as the stretch | SQLite keeps Lab 12 to one service; Postgres is what the how-to recommends and shows how Toasty connects with a token instead of a password |
| Cookie signing key | Key Vault secret referenced via MI | Session cookies from Lab 06 need a stable key across replicas and restarts |
| Sticky sessions | Not needed | Sessions are DB-backed, so any replica can serve any request — call this out as a benefit of the Lab 06 design |

### 10.6 Learner flow for Lab 12

```bash
# 0. prerequisites: az, azd, docker (or use the devcontainer, which has all three)
az login
azd auth login

# 1. from slipway/
azd init            # picks up azure.yaml; prompts for env name + region
azd up              # provision (Bicep) → build (Docker) → push (ACR) → deploy (ACA)
                    # prints: https://slipway-web.<random>.<region>.azurecontainerapps.io

# 2. verify
curl https://.../api/health
azd monitor --logs  # tail tracing output
azd monitor --overview   # opens App Insights

# 3. iterate
#    edit a component → azd deploy   (skips provision, ~2 min incl. cached build)

# 4. teardown
azd down --purge
```

Checkpoints in the lab README: (a) `azd provision` alone succeeds and `az containerapp show` reports the MI has AcrPull; (b) `azd deploy` succeeds and the URL renders the home page; (c) log in — session persists across a `az containerapp revision restart`; (d) `azd deploy` after a code change creates a new revision with zero downtime.

### 10.7 CI/CD: `azd pipeline config`

Run `azd pipeline config --provider github` once; it creates a federated credential (OIDC, no stored secrets) and writes `.github/workflows/azure-dev.yml`. The workshop ships that file already committed, with `azd provision` + `azd deploy` on push to `main` and a `workflow_dispatch` for manual runs. Stretch goal: add an `azd down` job on a nightly schedule for workshop tenants so nobody leaves an environment running.

### 10.8 Stretch goals for Lab 12

- Swap SQLite → PostgreSQL Flexible Server (module already in `infra/`, gated by a `deployPostgres` parameter).
- Add a second revision with 20% traffic for a blue/green demo using ACA revision traffic splitting.
- Put a Topcoat `#[procedure]` endpoint behind an internal-only ingress in a second container app, and call it from the first — a taste of the microservices story.
- Replace `Express` environment with a VNet-integrated one and private endpoint to Postgres, matching the private-networking constraints from your chargeback platform work.

### 10.9 Things to verify before writing the lab

- Asset bundling is settled: `topcoat asset bundle --release` writes `target/release/assets/`, and the binary and bundle must come from the same build (rendering an asset missing from the bundle panics). `AssetBundle::load()` reads the default location next to the executable; `load_dir` for custom paths.
- `Express` environment limits (regions, VNet support) as of the API version you pin.
- Toasty's PostgreSQL driver support for Entra token auth; if it needs a password, use a Key Vault reference rather than dropping the Entra-only rule.

---

## 11. Source links used

- Announcement: https://tokio.rs/blog/2026-07-22-announcing-topcoat
- Repo and README (Learn Topcoat index, roadmap): https://github.com/tokio-rs/topcoat
- Getting started: https://github.com/tokio-rs/topcoat/blob/main/crates/topcoat/docs/getting_started.md
- API docs: https://docs.rs/topcoat
- Toasty: https://github.com/tokio-rs/toasty
- Azure Container Apps docs: https://containerapps.azure.com/docs/containerapps
- ACA Bicep quickstart (API 2026-01-01, Express environments, MI registry pull): https://containerapps.azure.com/docs/containerapps/quickstart/setup-bicep
