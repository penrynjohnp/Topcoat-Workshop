# `view!` cheat sheet

Everything below is pinned to **topcoat 0.7.0** (see [COMPATIBILITY.md](https://github.com/penrynjohnp/Topcoat-Workshop/blob/main/COMPATIBILITY.md)).
Code is included from the [Lab 02 solution](../02-labs/lab-02.md), so it is compiled and tested by CI.

## Syntax at a glance

| You want | You write |
|---|---|
| Literal text | `<h1>"Berths"</h1>` |
| A Rust value | `<h1>(berth.name)</h1>` |
| An attribute value | `href=(berth.url())` |
| A dynamic attribute name | `(name)=(value)` |
| A boolean attribute, known statically | `disabled=""` |
| A boolean attribute, known at run time | `disabled=(is_disabled)` — absent when `false` |
| An optional attribute | `title=(maybe_note)` — absent when `None` |
| A conditional attribute | `aria-current=(is_current.then_some("page"))` |
| Several conditional attributes | `if is_current { aria-current="page" class="active" }` |
| A conditional class | `class=(class!("nav-link", "active" if is_current))` |
| An enumerated attribute | `aria-expanded=(if open { "true" } else { "false" })` |
| A reusable attribute set | `<article (attributes! { class="card" data-id=(id) })>` |
| A component | `status_badge(status: &berth.status)` |
| Rendering outside a component | `view! { cx => site_nav(current_path: "/") }` |

## Data into markup

```rust
{{#include ../../../labs/lab-02-view-macro/solution/src/lib.rs:model}}
```

`for` and `if` in child position:

```rust
{{#include ../../../labs/lab-02-view-macro/solution/src/lib.rs:berth_list}}
```

`match`, with pattern bindings in scope inside each arm:

```rust
{{#include ../../../labs/lab-02-view-macro/solution/src/lib.rs:status_badge}}
```

## Conditional attributes and class lists

```rust
{{#include ../../../labs/lab-02-view-macro/solution/src/lib.rs:nav}}
```

`class!` entries may be `expr`, `expr if cond`, or `expr if cond else alt`. An entry is *absent* when
it is `None`, an empty string, or its condition is false — absent entries leave no stray separator,
and when every entry is absent the `class` attribute is dropped. A list of literals is a
`StaticClass` and can live in a `const`.

`attributes!` accepts the full attribute syntax — literals, expressions, dynamic names, spreads, and
attribute-level `if`/`for`/`match` — and produces a map-like `Attributes` value. Keys are unique;
inserting the same key twice replaces it; render order is not guaranteed. Spreading consumes the
value, so clone it to use it twice.

## A page

```rust
{{#include ../../../labs/lab-02-view-macro/solution/src/lib.rs:page}}
```

Pages, layouts, components, shards and procedures all have the request context in scope implicitly.
In a plain function, name it first: `view! { cx => ... }`.

## Gotchas

- Bare words are Rust. Literal text needs quotes.
- `rustfmt` will not format inside `view!`. Run `topcoat fmt` as well — and note that 0.7.0's
  `topcoat fmt` has no `--check`, so CI formats and then runs `git diff --exit-code`.
- `for berth in berths` moves the collection, exactly as it does in ordinary Rust.
- `true` renders a present, empty attribute; `false` and `None` remove it.
- Views are lazy: nothing renders until the view becomes a response or is interpolated into one.
