# Module routing conventions

`module_router!()` derives paths from the compiled Rust module tree. Start it in the root module of
your route tree, declare every child with `mod`, and omit path strings from handlers whose paths
should be derived.

This reference maps every convention exercised in Lab 04. It follows Topcoat v0.7.0's
[`module_router!` reference](https://docs.rs/topcoat/0.7.0/topcoat/router/macro.module_router.html).

## Convention table

| Module-tree convention | Lab 04 example | Resulting route or scope |
|---|---|---|
| The module containing `module_router!()` is the route root. | `app` | `/` |
| A regular child module contributes one static segment. | `app::_marketing::berths` | `/berths` |
| Nested regular modules contribute segments in order. | `app::_marketing::api::health` | `/api/health` |
| Static module names are converted from snake case to kebab case. | The Lab 04 names contain no underscore; a module named `work_orders` would follow the same rule. | `/work-orders` |
| A leading underscore makes a logical group. The group contributes no served segment. | `app::_marketing` | Still `/`; there is no `/marketing`. |
| A `#[layout]` uses its module's logical path and wraps descendant pages. | `root_layout` in `app::_marketing` | Wraps `/`, `/berths`, and `/berths/{id}`; it does not wrap sibling groups. |
| A pathless `#[page]` uses its enclosing module path and serves `GET`. | `home` in `app::_marketing` | `GET /` |
| A handler's function name does not add or rename a segment. | `list` in the `berths` module | `GET /berths`, not `/berths/list`. |
| `path_param!(name)` replaces the enclosing module's static segment with a parameter. | `path_param!(id)` in `berths::id` | `/berths/{id}` |
| `path_param::<GeneratedType>(cx)` reads that request's decoded segment. | `path_param::<Id>(cx)` | The decoded `{id}` value as `&str`. |
| A pathless `#[route(METHOD)]` uses the module path and only the declared method. | `#[route(GET)]` in `api::health` | `GET /api/health` |
| `mod child;` makes a route module reachable by Rust and therefore discoverable. | `mod _marketing`, `mod berths`, `mod id`, `mod api`, `mod health` | The corresponding descendants are registered. Without the declaration, the file adds no route. |
| A path string on `#[page]`, `#[layout]`, or `#[route]` disables derivation for that item. | Used during Lab 04's manual-routing steps, then removed from the final solution. | The explicit path wins; the final table above applies only to pathless handlers. |

The module path and the served path are related but not identical. Groups remain in Topcoat's
logical path so they can scope layouts, even though their names are absent from the browser URL.

## Root and discovery

Lab 04 places `module_router!()` in `app.rs`. The `mod _marketing;` declaration makes the complete
route subtree reachable:

```rust
{{#include ../../../labs/lab-04-routing/solution/src/app.rs:app-router}}
```

`module_router!()` uses link-time discovery; it does not scan `src/app/` at runtime. A file that is
not compiled through the `mod` chain contributes nothing. The macro registers module-derived pages,
layouts, layers, and routes beneath its module.

Handlers with explicit path strings and feature-specific items are not automatically part of that
module-derived set. Register them explicitly or add the appropriate discovery extension when your
application uses them.

## Groups, layouts, and the root page

`_marketing` is a group because its name starts with an underscore. Its `#[layout]` and `#[page]`
share a logical location, but the group contributes no URL segment:

```rust
{{#include ../../../labs/lab-04-routing/solution/src/app/_marketing.rs:group-layout-home}}
```

The page therefore serves `/`. The layout wraps the page through `Slot<'_>` and also wraps pages in
the group's descendant modules. A group lets another top-level subtree use a different layout
without adding a visible prefix to either subtree.

## Static and dynamic segments

Each regular module adds one static segment. `berths.rs` therefore serves `/berths`. Its child module
would normally add `/id`, but `path_param!(id)` changes that module segment into `{id}`:

```rust
{{#include ../../../labs/lab-04-routing/solution/src/app/_marketing/berths/id.rs:path-param}}
```

The macro generates the Pascal-cased accessor type `Id`. An untyped declaration returns the
percent-decoded segment as `&str` and cannot fail during parsing. The page still validates that the
slug identifies a real berth and returns `404` when it does not.

For a parsed parameter, declare a type such as `path_param!(id: u64, error = bad_request)`. The
accessor then parses with `FromStr`, and the declared error policy controls a failed parse. Lab 04
uses the untyped form because berth slugs are strings.

A module contributes one segment and can declare one path parameter. Nest dynamic modules when a
route needs several parameters.

## API routes and methods

`api` and `health` are ordinary static modules, so they derive `/api/health`. `#[route(GET)]` supplies
the method without repeating the path:

```rust
{{#include ../../../labs/lab-04-routing/solution/src/app/_marketing/api/health.rs:health-route}}
```

`Json<Health>` controls serialization and the response content type; it does not affect route
derivation. A second handler in the same module could use the same path with a non-overlapping HTTP
method. Overlapping methods at the same derived path are rejected when the router is built.

## Reading a route declaration

Read a pathless handler from the outside in:

1. Start at the module containing `module_router!()`.
2. Follow the compiled `mod` declarations to the handler.
3. Remove group segments whose names begin with `_`.
4. Convert regular module names to kebab-case.
5. Replace modules declaring `path_param!` with their parameter placeholders.
6. Read the HTTP method from `#[page]` or `#[route(...)]`.
7. Apply layouts from matching logical ancestor modules.

For Lab 04, that procedure produces `GET /`, `GET /berths`, `GET /berths/{id}`, and
`GET /api/health`.

**See also:** [Lab 04 — Manual and module-based routing](../02-labs/lab-04.md),
[Routing and the request](../01-concepts/routing-and-the-request.md),
[Request lifecycle](../01-concepts/request-lifecycle.md), and
[`Cx` API summary](cx.md).
