# Module routing conventions

Topcoat module routing derives URL structure from compiled Rust modules. The route tree is explicit in
`mod` declarations and registered with `module_router!()`.

## Lab 04 tree

```text
src/app.rs                         # module_router!() root → /
src/app/_marketing.rs              # group + router layout; no /marketing segment
src/app/_marketing/berths.rs       # /berths
src/app/_marketing/berths/id.rs    # /berths/{id}
src/app/_marketing/api/health.rs   # GET /api/health
```

The app router is the root of the module-derived tree:

```rust
{{#include ../../../labs/lab-04-routing/solution/src/app.rs:app-router}}
```

## Dynamic segments

Declare the parameter in the module that becomes dynamic. The parameter name must match the route
placeholder, and the handler reads it from `Cx`:

```rust
{{#include ../../../labs/lab-04-routing/solution/src/app/_marketing/berths/id.rs:path-param}}
```

`path_param!(id)` returns the decoded segment as `&str`. A typed declaration parses with `FromStr` and
can map failures to `bad_request`, `not_found`, or another router error.

## Module-derived API routes

A method annotation without a path derives the URL from the module tree:

```rust
{{#include ../../../labs/lab-04-routing/solution/src/app/_marketing/api/health.rs:health-route}}
```

`module_router!()` discovers the route because the API module is reachable from `app.rs`. It does not
scan the filesystem; an unreferenced `.rs` file contributes nothing.

## Groups and layouts

A leading underscore creates a group. It is absent from the served URL but remains part of the logical
path used for layout and layer matching. `#[layout]` in the group can wrap all descendant pages, while
`Slot<'_>` is where the matched page renders.

For a complete explanation and the manual-registration-to-discovery progression, see [Lab 04](../02-labs/lab-04.md)
and [Routing and the request](../01-concepts/routing-and-the-request.md).
