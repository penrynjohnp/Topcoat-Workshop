# Lab 04 — Manual routing, then module-based routing

**Time:** 60 min · **Module:** 2 · **Prerequisites:** Lab 03 complete (or copy `labs/lab-03-components/solution`)

## What you'll learn
- Register pages, layouts and API routes explicitly with `Router::builder()`
- Read a string path parameter from `Cx` with `path_param!`
- Derive routes from a Rust module tree with `module_router!`

## Concepts (read first, 5 min)
Read [Routing and the request](https://github.com/penrynjohnp/Topcoat-Workshop/blob/main/docs/src/01-concepts/routing-and-the-request.md).

This lab starts from Lab 03's working Slipway solution. You keep its data and components; routing is
the new problem. A router matches an HTTP method and path to a page or route. You can register each
handler by hand with `.page(...)`, `.layout(...)` and `.route(...)`, or let Topcoat derive URLs from
Rust modules. The latter is not filesystem magic: Rust must compile each module through a `mod`
declaration, and `module_router!()` discovers the annotated items below its module.

Request-scoped values live in `Cx`. A path parameter is declared where the route becomes dynamic, then
read from `Cx` inside the handler. Router layouts are different from Lab 03's component `layout`:
`#[layout]` receives a `Slot<'_>` containing the matched page, so the router can wrap whole route
subtrees. API routes use `#[route(GET)]` and return a response value such as `Json<T>`.

## Steps

The starter is Lab 03's solution copied into one file. Run it before changing anything:

```bash
cd labs/lab-04-routing/starter
topcoat dev
```

### Step 1 — Register the existing app manually

The starter currently uses explicit `#[page("...")]` attributes and discovery. Replace its startup
with a `router()` function that registers the existing handlers explicitly:

```rust
fn router() -> Router {
    Router::builder()
        .page(home)
        .page(berths_page)
        .page(featured_berth_page)
        .build()
}
```

(Keep this first version simple; Step 5 adds the router-level layout and Step 6 replaces the manual
entries with module discovery.) Manual registration is useful when integrating legacy routes or when
you want registration order and ownership to be obvious.
> ✅ **Checkpoint:** `/`, `/berths`, and `/berths/featured` still render. The router has no `.discover()`
> call and every page appears in the builder by name.

### Step 2 — Put request data in `Cx`

A handler may declare `cx: &Cx`; Topcoat supplies it for the current request. Add it to a temporary
page and render the request URI while you learn the API:

```rust
#[page("/request-info")]
async fn request_info(cx: &Cx) -> Result<impl View> {
    Ok(view! { <p>(topcoat::router::request::uri(cx).path())</p> })
}
```

`Cx` is request-scoped, not global application state. Lab 05 goes deeper into app context and
memoization; here it is simply how a handler reads values selected by the router.
> ✅ **Checkpoint:** `/request-info` prints `/request-info`. Remove the temporary page after verifying
> it, or leave it while completing the lab.

### Step 3 — Add `/berths/{id}`

Create `src/app/berths/id.rs` in the solution shape, or temporarily add the equivalent declarations
beside the starter handlers:

```rust
path_param!(id);

#[page("/berths/{id}")]
async fn detail(cx: &Cx) -> Result<impl View> {
    let id = path_param::<Id>(cx);
    // find_berth(id), then render berth_card(slug: id)
}
```

For the final module form, the path string disappears. The file `app/berths/id.rs` is a dynamic
module: `path_param!(id)` turns its segment into `{id}`, so its `#[page]` serves `/berths/{id}`. The
parameter is a decoded `&str`, which fits Slipway's existing slugs (`a1`, `b7`). It is deliberately
not a function argument; handler arguments are reserved for `Cx` and body extractors.
> ✅ **Checkpoint:** `/berths/a1` returns the A1 card and contains `Lady Jane`. `/berths/a%201` is
> decoded before lookup. `/berths/does-not-exist` returns 404 instead of panicking.

### Step 4 — Add the JSON health route

Create `app/api/health.rs`:

```rust
#[derive(Serialize)]
struct Health { status: &'static str }

#[route(GET)]
async fn health() -> Result<Json<Health>> {
    Ok(Json(Health { status: "ok" }))
}
```

Under the module router, `app/api/health.rs` derives `GET /api/health`. `Json<T>` is explicit: a
plain string would be a text response, while `Json` serializes the value and sets
`Content-Type: application/json`.
> ✅ **Checkpoint:** `curl -i http://127.0.0.1:3000/api/health` shows `200`,
> `content-type: application/json`, and `{"status":"ok"}`.

### Step 5 — Replace the component shell with a router layout

Lab 03's `layout` component accepted `Child`. A router layout has a different job and a different
signature:

```rust
#[layout]
async fn root_layout(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    Ok(view! {
        <!DOCTYPE html>
        <html>
            <body>
                (slot)
            </body>
        </html>
    })
}
```

Put the layout in `app/_marketing.rs`. The leading underscore makes it a group: it contributes no URL
segment but still scopes the layout to its descendants. The page body moves into `(slot)`, and the
existing `site_nav`, `berth_card` and `site_footer` components remain reusable inside pages.
> ✅ **Checkpoint:** `/`, `/berths`, and `/berths/a1` all have one shared document shell, while the
> page-specific content remains different.

### Step 6 — Switch to `module_router!`

Declare the modules from `app.rs`; Rust must compile them for discovery to find them:

```rust
mod _marketing;

pub fn router() -> Router {
    module_router!().build()
}
```

The final solution tree is:

```text
src/
├── app.rs
├── app/
│   └── _marketing.rs
│       ├── berths.rs
│       ├── berths/id.rs
│       └── api/health.rs
├── shared.rs
└── main.rs
```

`_marketing` is omitted from served URLs; `berths` contributes `/berths`; `id` becomes `{id}` because
of `path_param!(id)`; `api/health` contributes `/api/health`. Function names do not determine URLs.
> ✅ **Checkpoint:** remove all manual `.page(...)`, `.layout(...)` and `.route(...)` calls from the
> final router. `cargo test -p lab04-solution` still finds every route.

### Step 7 — Test the route table, not just the markup

Use the in-process router test from earlier labs. The important difference is that the test calls
`app::router()`, not a builder that registers test-only routes:

```bash
cargo test -p lab04-solution --test pages
```

Read the test output and connect each assertion to a route: the slug page, unknown slug, and JSON
health response. This is the checkpoint for the complete module tree.
> ✅ **Checkpoint:** the integration test covers `/berths/{id}` and `GET /api/health`, and all tests pass
> with no manual route registration in the test.

## Stretch goals
- Add `app/_marketing/about.rs` and observe that a group module does not add `marketing` to the URL.
- Add a nested `app/_marketing/berths/_admin.rs` group and a layout just for that logical subtree.
- Add a typed parameter (`path_param!(id: u64, error = bad_request)`) in a scratch route and compare
  its 400 response with the string slug's 404 lookup.
- Add a `POST /api/health` route beside the GET route and confirm method matching with `curl -X POST`.
- Use `href!` with a declared path parameter to build a URL without hand-writing `/berths/`.

## Troubleshooting
- **`module_router!()` finds nothing** → the route file is not reachable from a `mod` declaration. It
  does not scan the filesystem.
- **`path parameter "id" was not found`** → the handler was called for a route that did not declare
  the matching parameter, or the declaration name and `{id}` placeholder differ.
- **`/berths/a1` is 404** → confirm `path_param!(id)` is in `app/berths/id.rs`, `find_berth` uses the
  decoded slug, and the module is declared.
- **Health returns text instead of JSON** → return `Json(Health { ... })`, not a bare string; the
  wrapper controls serialization and the content type.
- **The page shell appears twice** → do not keep the Lab 03 component document shell and the router
  `#[layout]` shell around the same page. Keep the shared components, but let the router layout own
  the document.
- **`#[layout]` rejects `child`** → router layouts receive `slot: Slot<'_>`, while ordinary components
  receive `child: Child<'_>`.

## What's next
Lab 05 uses `Cx` for app context and `#[memoize]` to deduplicate repeated berth loads across composed
components.
