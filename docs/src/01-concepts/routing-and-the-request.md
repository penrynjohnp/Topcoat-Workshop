# Routing and the request

Routing answers two questions: which handler should run for this HTTP method and path, and what
request-scoped values should that handler be able to read? Topcoat keeps the handler signature small.
Pages and routes may take `Cx` and, at most, one body extractor. The router stores path and query
matches in `Cx` so helpers and nested components can read the same request data.

## Manual registration and discovery

A manual builder makes ownership explicit: register layouts, pages and API routes by name, then build
the router. It is useful for small apps and for integrating a route tree incrementally. Discovery
moves the registration boundary to annotations; `RouterBuilder::discover` collects annotated items,
while `module_router!()` additionally derives URLs from the Rust module tree.

```rust
{{#include ../../../labs/lab-04-routing/solution/src/app.rs:app-router}}
```

`module_router!()` does not scan directories. Rust must compile every route module through `mod`
declarations. A module contributes one URL segment, static names become kebab-case, and a leading
underscore makes a group that contributes no served segment. Function names do not affect URLs.

## Path parameters

A dynamic segment is declared with `path_param!` in the module that contributes it. `path_param!(id)`
turns `app/berths/id.rs` into `/berths/{id}` and generates the `Id` type. The handler reads the
percent-decoded `&str` from `Cx`:

```rust
{{#include ../../../labs/lab-04-routing/solution/src/app/_marketing/berths/id.rs:path-param}}
```

Typed declarations such as `path_param!(id: u64, error = bad_request)` parse with `FromStr` and map
failures to a router error. Unparsed slugs cannot fail parsing; the application decides whether an
unknown slug is a 404.

## Layouts versus components

Lab 03's component `layout` accepted `Child<'_>` and was called inside a page. A router `#[layout]`
receives `Slot<'_>` and wraps the matched page automatically. Layouts apply by path prefix, and group
modules let you share a layout without adding a URL segment. Keep the two concepts distinct: a
component is explicit composition; a router layout is route-tree composition.

## API routes

Pages render views. API routes return response values. `#[route(GET)]` in `app/api/health.rs` derives
`GET /api/health`; wrapping a serializable value in `Json<T>` opts into JSON serialization and sets
the response content type:

```rust
{{#include ../../../labs/lab-04-routing/solution/src/app/_marketing/api/health.rs:health-route}}
```

A bare string is not silently converted to JSON. That explicit boundary makes it easy to inspect an
API response and keeps the response contract visible in the handler.

## What belongs in `Cx`

`Cx` is request-scoped. It is the place for the URI, path/query parameters, cookies, sessions and
other values whose lifetime is one request. It is not a global state bag. Lab 05 adds long-lived app
context and memoization, proving how repeated component calls can share request-scoped work without
threading data through every prop.
