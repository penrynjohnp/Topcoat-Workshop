# Components and composition

A component is an `async fn` with `#[component]` on it that returns `Result<impl View>`. That is the
whole idea. There is no base class, no props object, no lifecycle, no re-render loop — the things a
component can do are the things a Rust function can do, which includes `await`ing a query.

## Arguments are parameters

Parameters are ordinary function parameters, and a call inside `view!` uses named arguments:

```rust
{{#include ../../../labs/lab-03-components/solution/src/lib.rs:pages}}
```

Three parameter attributes tune the call site:

| Attribute | Effect |
|---|---|
| `#[default]` | the argument may be omitted; falls back to `Default::default()` |
| `#[default(expr)]` | custom fallback, evaluated only when the argument is omitted; no `Default` bound needed |
| `#[into]` | callers may pass anything `Into<T>`, converted outside the body so the component isn't monomorphised per caller |

`key` is reserved: `key:` on a call keys that invocation's identity rather than setting a prop, so no
component may declare a `key` parameter.

## Child content is a parameter too

A parameter named `child` of type `Child<'_>` collects whatever extra nodes the caller wrote inside
the call. Give it `#[default]` so the component can also be called with none. This is how a component
wraps content it knows nothing about — the page shell being the obvious case:

```rust
{{#include ../../../labs/lab-03-components/solution/src/lib.rs:layout}}
```

Every page then supplies only its own markup, and there is exactly one copy of the document.

## Locality of behaviour: fetch, don't thread

The interesting consequence of components being `async fn`s is that a component can fetch what it
renders. The alternative — the page loads everything and threads it down as props — spreads one
feature across every layer between the route and the markup.

```rust
{{#include ../../../labs/lab-03-components/solution/src/lib.rs:berth_card}}
```

The card takes a slug, not a `Berth`. Everything about showing a berth — the lookup, the not-found
case, the markup, and in a later lab the authorisation check — is in one function you can read top to
bottom, and any caller holding a slug can render one. The same argument, applied to auth instead of
data, is the subject of [locality of behaviour vs middleware](functions-not-middleware.md).

The cost is real and worth stating: a list of twenty berths performs twenty lookups. Topcoat's answer
is [`#[memoize]`](../02-labs/lab-05.md), which dedupes identical calls within a request; the deeper
answer is Lab 10's query design. Do not reach for prop-threading as the fix before measuring.

```rust
{{#include ../../../labs/lab-03-components/solution/src/lib.rs:berth_list}}
```

## Attribute pass-through

A component that renders an element should usually let its caller decorate that element. Accept
`#[default] attrs: Attributes` and spread it with `<article (attrs)>`.

Merging beats replacing for `class`: take the caller's value out with `attrs.remove("class")` — it
returns an `AttributeValue`, which `class!` accepts as an entry — and combine it with the component's
own classes. Spreading a `class` straight through would silently drop them.

Two caveats, both documented upstream:

- Spreading consumes the `Attributes`; `clone()` it to use it twice.
- Once an element's attributes go through an `Attributes` map, their **render order is not
  guaranteed**. Tests should assert on the set of attributes, not on an exact string.

## Composition, not inheritance

Sibling components in one view render concurrently, since each is a future. A component that calls
itself — directly or through a cycle — describes a view type containing itself; break the cycle by
erasing one of them with `.boxed()` from `ViewExt`. The same trick is needed whenever one component
returns two different `view!` types, as `berth_card` does for its not-found branch.

Composition is therefore just function composition: pass arguments, accept children, forward
attributes, and let each component answer for itself.
