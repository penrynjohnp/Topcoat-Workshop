# Troubleshooting

This page collects every troubleshooting entry from the lab READMEs in one place, grouped by what you
observe rather than by which lab you are on. Find your symptom, then follow the link to the lab that
explains the surrounding concept.

Everything here is specific to the versions in
[`COMPATIBILITY.md`](https://github.com/penrynjohnp/Topcoat-Workshop/blob/main/COMPATIBILITY.md):
Topcoat and Topcoat CLI 0.7.0, Toasty 0.10.0. Topcoat is pre-1.0, so check that file before assuming
an error is your mistake.

## Toolchain and CLI

| Symptom | Cause | Fix |
|---|---|---|
| `topcoat: command not found` | The CLI is not installed, or `~/.cargo/bin` is not on `PATH` | `cargo install topcoat-cli --version 0.7.0 --locked`, then confirm `PATH` |
| A compile error names a missing import | Topcoat's facade crate re-exports through feature-gated modules, so a path can exist only when its feature is on | Compare your `use topcoat::{...}` block with that lab's solution, and check the crate's enabled features |
| `topcoat --version` errors with an unexpected argument | CLI 0.7.0 has **no top-level `--version` flag** | Read the pin in `COMPATIBILITY.md`. `topcoat fmt --version` exists and prints `topcoat-fmt 0.7.0`, but only checks the formatter binary |
| `topcoat fmt --check` is rejected | The formatter has no `--check` flag at 0.7.0 | Run `topcoat fmt <files>` and then `git diff --exit-code -- '*.rs'`, which is what CI does |
| `topcoat build` is unknown | That command does not exist in CLI 0.7.0 | Use `cargo build` plus `topcoat asset bundle`, with matching profile flags on both |
| A build fails reading an empty Iconify cache file, usually right after `cargo clean` | `topcoat-icon`'s build script stages its cache non-atomically, so parallel workspace builds can read a half-written file | Stage one crate first: `find target/topcoat/cache -type f -empty -delete` then `cargo build -p lab11-solution`, then build the workspace. CI does exactly this |

See [CLI commands](../03-reference/cli.md) for the full `--help` surface and
[How-to: pin and upgrade Topcoat](../03-reference/howto/upgrade.md) for changing any of these pins.

## `view!` and markup

| Symptom | Cause | Fix |
|---|---|---|
| `expected one of ... found "Berths"` | Bare words inside `view!` are parsed as Rust | Quote literal text: `<h1>"Berths"</h1>` |
| The class list renders oddly, such as `nav-link nav-link-active"` | `class!` was placed in the attribute *name* position, or the parentheses are missing | It belongs in the value: `class=(class!(...))` |
| An attribute renders as `aria-current=""` when you wanted it removed | You passed `true` rather than `Some(...)`/`None` | `true` renders a present, empty attribute; `false` and `None` remove it |
| `` expected `=` `` on a `<script>` tag | `view!` rejects bare boolean attributes | Write `defer="defer"`, not `defer` |
| `cannot borrow ... after move` | `for berth in berths` moves the `Vec` | Iterate `&berths` if you need the collection afterwards |

Labs [02](../02-labs/lab-02.md) and [09](../02-labs/lab-09.md). The
[view! cheat sheet](../03-reference/view-cheatsheet.md) lists every construct.

## Components and composition

| Symptom | Cause | Fix |
|---|---|---|
| `missing argument child` | The parameter is required | Mark it `#[default]` to make it optional |
| `` `impl Trait` … opaque type ``, or two arms returning different types | A component returns two different `view!` types | Call `.boxed()` on both arms and import `ViewExt` |
| The caller's `class` replaced the component's own | `attrs` was spread without removing `class` first | `attrs.remove("class")`, then pass the removed value to `class!` as an entry |
| `use of moved value: attrs` | Spreading an `Attributes` consumes it | `clone()` when you need it twice |
| Attribute order changes between renders | Expected: spreading `Attributes` routes that element through a map, and 0.7.0 does not guarantee render order | Assert on the *set* of attributes, not on a byte-for-byte string |
| `cannot find value cx` | Components only receive the request context when they declare it | Add `cx: &Cx` to the signature — introduced in [Lab 05](../02-labs/lab-05.md) |

Lab [03](../02-labs/lab-03.md).

## Routing and 404s

| Symptom | Cause | Fix |
|---|---|---|
| `module_router!()` finds nothing | The route file is not reachable from a `mod` declaration; discovery does not scan the filesystem | Declare the module so Rust compiles it |
| `path parameter "id" was not found` | The handler ran for a route with no matching parameter, or the declaration and `{id}` placeholder differ | Align `path_param!(id)` with the module segment |
| `/berths/a1` returns 404 | The parameter module is missing, undeclared, or the lookup uses a different slug | Confirm `path_param!(id)` is in `app/berths/id.rs`, the module is declared, and the lookup uses the decoded slug |
| A health endpoint returns text instead of JSON | A bare string was returned | Return `Json(Health { ... })`; the wrapper sets serialization and content type |
| The page shell appears twice | Both a component document shell and a router `#[layout]` shell wrap the same page | Keep the shared components, but let the router layout own the document |
| `#[layout]` rejects `child` | Router layouts and components take different parameters | Router layouts receive `slot: Slot<'_>`; components receive `child: Child<'_>` |
| A fragment endpoint 404s | `#[route(GET)]` derives its path from the module path, so renaming the file changes the URL | Point `hx-get` at the real path, for example `/vessels/htmx/results` |
| The whole page appears inside a results element | The request targeted a `#[page]` instead of a `#[route]` | Pages are wrapped by layouts; routes are not |

Labs [04](../02-labs/lab-04.md) and [09](../02-labs/lab-09.md), plus
[Module routing conventions](../03-reference/module-routing.md).

## Request context and memoization

| Symptom | Cause | Fix |
|---|---|---|
| `app_context` panics | The value was never registered on the router, or the type does not match exactly | Register it with `.app_context(...)`; lookup is by concrete Rust type |
| A memoized load counter is greater than 1 | The memoized call is re-entered with a different key, or runs outside the same request | Share the same `cx` and the same argument values; `#[memoize]` deduplicates within one request only |
| The page compiles but renders more than expected | Copies of the same request are not sharing a context or key | Pass the same `cx` and slug to every caller |
| A test cannot observe the count | The state was moved into the router | Hold a clone of `AppState` outside the router and read it after the request completes |

Lab [05](../02-labs/lab-05.md), plus the [Cx API summary](../03-reference/cx.md).

## Auth, sessions, cookies, and mail

| Symptom | Cause | Fix |
|---|---|---|
| A protected page returns 500 | Cookies or sessions were never registered on the router | Add `.cookies().sessions(...)` before `.build()` |
| The session cookie is present but the user is anonymous | The token hash has no live application record | Persist the `Session` returned by `session::start` and check its expiry |
| A protected component works on one page but not another | The guard lives in the route instead of the component | Call `require_auth(cx)` as the component's first line |
| A cookie write panics during streaming | Response headers had already been sent | Set or remove cookies *before* returning a streaming view |
| An old browser cookie stops working after a restart | The cookie encryption key changed | Persist the key in deployment configuration; never hard-code or commit it |
| The mail directory is empty | The mail feature or configuration is missing | Enable Topcoat's `mail` feature and register `MailConfig` with `FileTransport` |

Labs [06](../02-labs/lab-06.md) and [10](../02-labs/lab-10.md), plus
[Locality of behaviour vs middleware](../01-concepts/functions-not-middleware.md).

## Runtime expressions and signals

| Symptom | Cause | Fix |
|---|---|---|
| `error: unsupported operator` on `&&` or `\|\|` | `bool` has `!`, comparisons, `then`, and `then_some`, but no logical operators in the shared vocabulary | Restructure with `let` bindings and `if`/`else` |
| `error: unsupported expression` | `match`, struct literals, or multi-segment paths are outside the vocabulary | Restructure, or escape deliberately with `raw!` |
| Type errors around `1` or `2` | Every runtime number is `f64`, so integer literals are rejected | Write `1.0` |
| The page renders but nothing reacts | The runtime script is missing, or the bundle came from another profile | Build the router with a bundle and re-run `topcoat dev` or the matching `topcoat asset bundle` |
| Panic: an asset is not present in the loaded bundle | Binary and bundle came from different builds | Rebuild both with the same profile, or pass `None` in tests |
| A captured value never updates | Captures are snapshots taken during the server render | Use a shard when markup must follow later server state |
| `internal compiler error: encountered incremental compilation error` after uncommenting the `match` example | A stale incremental-cache artifact, not your code | `cargo clean -p lab07-solution`, then rebuild |

Lab [07](../02-labs/lab-07.md), plus [How `$(...)` reaches the browser](../01-concepts/dual-expressions.md).

## Shards, procedures, and streaming

| Symptom | Cause | Fix |
|---|---|---|
| A shard route returns 404 | Shards are not discovered, or the shard is not linked into the binary | Call `.discover_shards()` on the router and make the shard reachable |
| A procedure route returns 404 | Procedures are not discovered, or the procedure is never rendered or captured | Call `.discover_procedures()` and reference it from reachable code |
| A shard `POST` returns 415 or 400 | The body is not a JSON tuple | Send `Content-Type: application/json` with a tuple body such as `["Lady"]` |
| A signal resets after each search | It was declared *inside* the shard, which is replaced wholesale | Declare it in the page and pass it in as an argument |
| Private data appears without the page guard | Shard and procedure endpoints are independently callable, so page and layout code never runs | Call `require_auth(cx)` inside every private endpoint and validate arguments |
| Every keystroke creates a request | Expected at 0.7.0: same-tick changes coalesce and stale requests abort, but debouncing is application logic | Use htmx's `delay:` trigger, or implement debouncing yourself |
| A cookie or header panics during streaming | Headers changed after the first emission | Authenticate and mutate cookies before returning the live view |
| Only the final streamed content appears in a test | `to_bytes` collects the whole body | Assert on fallback text and `data-topcoat-swap` envelopes, or poll frames to observe timing |

Lab [08](../02-labs/lab-08.md), plus
[Shards, procedures, live regions, htmx](../01-concepts/reactivity-options.md).

## htmx

| Symptom | Cause | Fix |
|---|---|---|
| The htmx page renders but typing does nothing | htmx never loaded | Check the `<script>` tag in view-source, and whether a CSP or offline environment blocks `cdn.jsdelivr.net`. Vendor it with `asset!` if CDNs are blocked |
| `unresolved import topcoat::htmx` | The `htmx` feature is not enabled; it is not in `default` | Enable it per-crate: `topcoat = { workspace = true, features = ["htmx"] }`. `topcoat.workspace = true` alone is not enough |
| Response headers are missing | A responder was placed last and treated as the body | Responders must precede the body: `(HxPushUrl(url), trigger, body)` |
| `HX-Push-Url` appears but the reloaded link shows nothing | The page does not read `?q=` and server-render | Render the initial results from the query string; a pushed URL is a promise the page must keep |
| The two-transport equality test fails after an edit | One transport stopped calling the shared markup function | Restore the shared call — that test exists to catch the copies drifting apart |

Lab [09](../02-labs/lab-09.md).

## Database and Toasty

| Symptom | Cause | Fix |
|---|---|---|
| Startup says a table already exists | `push_schema()` ran against an existing file, and it is not idempotent at 0.10.0 | Delete `slipway.db` while learning; use real migrations before changing a deployed schema |
| The database looks empty after changing `SLIPWAY_DATABASE_URL` | A different SQLite file was opened | Print or inspect the exact URL the process used |
| A relation is empty even though the child row exists | The query did not call `.include(...)`, or the child has the wrong foreign key | Add `.include(...)` and check `berth_slug` |
| A `POST` returns 415 or fails to extract `Form` | Wrong request content type | Send `Content-Type: application/x-www-form-urlencoded` |
| A procedure rejects a numeric ID | Topcoat 0.7 transports runtime numbers as `f64` | Accept the `f64`, validate it is a non-negative integer, then convert it to the database key |

Lab [10](../02-labs/lab-10.md), plus [How-to: SQLite to PostgreSQL](../03-reference/howto/postgres.md).

## Assets, Tailwind, and owned UI

| Symptom | Cause | Fix |
|---|---|---|
| `tailwindcss` cannot download | The standalone CLI is fetched on the first build | Retry with network access, or point `BuildConfig` at a preinstalled executable. npm is not required |
| `iconify::include!` says the set was not staged | The set is missing from the build script or build dependencies | Add it to `build.rs`, keep `icon-iconify` in build dependencies, and rebuild |
| `fontsource_font!` cannot find `topcoat_asset` | Self-hosting expands through that crate | Keep the workspace-pinned `topcoat-asset` dependency enabled |
| Rendering panics with "failed to resolve asset" | The loaded manifest came from a different binary, checkout, or profile | Re-run the bundler for the exact binary you will execute |
| Tailwind omits a class | Dynamically assembled class fragments are invisible to its scanner | Keep class names as literal strings in Rust or `styles.css` |
| A local component edit vanished | `topcoat ui add card --overwrite` replaced the owned file | Restore the diff, then reapply upstream changes deliberately |

Lab [11](../02-labs/lab-11.md).

## Containers and deployment

| Symptom | Cause | Fix |
|---|---|---|
| Deployed, but every request times out or 502s | The server is bound to `127.0.0.1`, which platform ingress cannot reach | Set `HOST=0.0.0.0` and match the platform's `PORT`. Check the Dockerfile `ENV` line and any `-e HOST=` override. Every prior lab and `topcoat dev` default to loopback |
| `ConfigError("SLIPWAY_DATABASE_HOST is required in production unless SLIPWAY_DATABASE_URL is explicitly set")` | `SLIPWAY_ENV=production` with no database configured | On Azure set `SLIPWAY_DATABASE_HOST`/`_NAME`/`_USER` and `AZURE_CLIENT_ID`. Off Azure set an explicit `SLIPWAY_DATABASE_URL` on a persistent volume, or run locally with `-e SLIPWAY_ENV=development` |
| `azd deploy` cannot pull from ACR right after `azd provision` | The `AcrPull` role assignment can take up to a minute to propagate | Retry |
| New connections fail hours after a successful deploy while existing ones still work | Toasty 0.10.0's PostgreSQL driver holds one managed-identity token and has no refresh callback | Restart the revision (`az containerapp revision restart`). This is a known limitation, not a misconfiguration |
| `azd provision` fails on the `app` module with a missing Key Vault secret reference | The bootstrap script's RBAC-propagation retry loop lost a race on a very fresh subscription | Re-run `azd provision`; the script is idempotent and `dependsOn` guarantees ordering |
| `azd down` succeeds, but a later `azd up` fails with a Key Vault name conflict | The vault was soft-deleted and its name stays reserved | Use `azd down --purge` |

Lab [12](../02-labs/lab-12.md), plus
[How-to: deploy to Azure Container Apps](../03-reference/howto/deploy-azure.md).

## Tower and Axum bridge

| Symptom | Cause | Fix |
|---|---|---|
| `TowerRoute` does not match the bare prefix | A catch-all route does not cover the prefix itself | Register `"/api/v1/{*rest}"`, and add `"/api/v1"` if you serve the bare prefix too |
| The JSON response is empty or has the wrong `Content-Type` | The endpoint did not return an Axum JSON response, or the route is not nested inside the mounted router | Return `axum::Json(...)` and check the nesting |
| Middleware wraps every route, not just the API path | The layer was applied without a path | Use `.at("/api/v1")` on each `TowerLayer` call |
| Compilation fails on the bridge | The `tower` feature is off, or dependency versions disagree | Enable `tower` on `topcoat` and keep `axum` and `tower-http` on the workspace pins |

Lab [13](../02-labs/lab-13.md).

## When none of this matches

> [!TIP]
> Reproduce the failure with the smallest command that shows it — usually a single package, such as
> `cargo test -p lab08-solution` — before changing anything else.

1. Compare your code with that lab's `solution/`, which CI compiles and tests on every push.
2. Check `COMPATIBILITY.md` for a recorded breakage against the pinned versions.
3. Read the Topcoat guides at the **release tag**, never `main`. The unreleased API is ahead of 0.7.0
   and its examples will not compile here.
4. Open an issue with the lab number, the exact command, and the full error.

**See also:** [Learning with Copilot](learning-with-copilot.md),
[Rust in 30 min for web devs](rust-for-web-devs.md),
[HTML/HTTP in 30 min for Rustaceans](web-for-rustaceans.md),
[CLI commands](../03-reference/cli.md), and
[How-to: pin and upgrade Topcoat](../03-reference/howto/upgrade.md).
