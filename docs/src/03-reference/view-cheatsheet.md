# `view!` cheat sheet

Use this page when you know what markup you want and need the Topcoat syntax. Every example comes
from the compiled solutions for Labs 02–03 and targets Topcoat 0.8.0.

The syntax follows the pinned v0.8.0 [`view!`
guide](https://raw.githubusercontent.com/tokio-rs/topcoat/v0.8.0/crates/topcoat-view/macro/docs/view.md).

## Syntax at a glance

| Construct | One-line example | What it does |
|---|---|---|
| Interpolation | `{{#include ../../../labs/lab-02-view-macro/solution/src/lib.rs:cheatsheet-interpolation}}` | Inserts a Rust expression as an escaped child node. Parentheses also supply dynamic attribute values. |
| Loop | `{{#include ../../../labs/lab-02-view-macro/solution/src/lib.rs:cheatsheet-loop}}` | Repeats the body for each item using ordinary Rust `for` syntax. |
| Conditional | `{{#include ../../../labs/lab-02-view-macro/solution/src/lib.rs:cheatsheet-conditional}}` | Renders the selected `if`/`else` branch. `match` is also supported in child position. |
| Conditional attribute | `{{#include ../../../labs/lab-02-view-macro/solution/src/lib.rs:cheatsheet-conditional-attribute}}` | Omits the whole attribute when the expression is `None` or `false`. |
| `class!` | `{{#include ../../../labs/lab-02-view-macro/solution/src/lib.rs:cheatsheet-class}}` | Joins static and conditional classes without leaving extra separators. |
| `attributes!` | `{{#include ../../../labs/lab-03-components/solution/src/lib.rs:cheatsheet-attributes}}` | Builds a reusable `Attributes` value with the same attribute syntax as `view!`. |
| Component call | `{{#include ../../../labs/lab-02-view-macro/solution/src/lib.rs:cheatsheet-component-call}}` | Calls a component with named properties. |
| Child content | `{{#include ../../../labs/lab-03-components/solution/src/lib.rs:cheatsheet-child-content}}` | Passes unnamed nodes after the named properties into the component's `Child<'_>` parameter. |

## Interpolation

Write literal text in quotes. Put a Rust expression in parentheses:

- child position produces a node;
- attribute-value position produces an escaped value;
- a `bool` attribute value is present when `true` and absent when `false`;
- an `Option` attribute value is present for `Some` and absent for `None`.

Views are lazy. An interpolated expression runs when its view renders, not merely when the `view!`
value is created.

## Loops and conditionals

`view!` uses Rust-shaped `for`, `if`, and `match` control flow with markup bodies. A loop body renders
once per item. A conditional renders only its selected branch.

Values follow ordinary Rust ownership rules. For example, `for berth in berths` consumes `berths`.
Sibling components and loop iterations may render concurrently, but their HTML remains in source
order. Do not use component execution order for coordination.

## Conditional attributes

An expression-valued attribute removes itself when its value is `false` or `None`. This is the right
shape for HTML boolean attributes and optional values.

A literal attribute is always present. `disabled="false"` still disables an HTML control because the
attribute exists. Use `disabled=(is_disabled)` when presence is conditional.

For enumerated attributes such as `aria-expanded`, return the strings `"true"` and `"false"`. Those
values have meaning and should not be replaced with attribute omission.

## `class!` and `attributes!`

`class!` accepts ordinary entries and conditional entries such as `"active" if is_current`. Empty,
absent, and false entries are omitted. If every entry is absent, the whole `class` attribute is
omitted.

`attributes!` creates an `Attributes` collection outside an element. You can pass it as a component
property or spread it into an element with `(attrs)`. Spreading consumes the collection. Clone it if
you need to spread it more than once.

Attribute keys in an `Attributes` collection are unique, and insertion order is not a rendering
contract. Tests should compare the attribute set rather than an exact opening-tag string.

## Component calls and child content

Component properties use `name: value`. Child nodes follow the named properties without a property
name or commas. The receiving component declares a `Child<'_>` parameter and interpolates it where
the caller's content belongs.

Lab 03's document layout shows the receiving side:

```rust
{{#include ../../../labs/lab-03-components/solution/src/lib.rs:layout}}
```

A component call may include another component as child content. The children become one view passed
to the receiving component; they are not ordinary positional Rust arguments.

> [!NOTE]
> `key` is reserved on component calls. Use it to give repeated component invocations stable identity,
> especially inside loops; do not declare a component property named `key`.

## Formatting and common mistakes

- Bare words are Rust expressions. Quote literal text.
- HTML void elements such as `<img>` and `<input>` have no closing tag.
- Run both `cargo fmt` and `topcoat fmt`; `rustfmt` does not format the inside of `view!`.
- In a plain function, provide the request context explicitly with `view! { cx => ... }`. Pages,
  layouts, components, and shards receive it implicitly.
- Spreading `Attributes` consumes the value.
- A `view!` value captures referenced variables like an `async move` block.

**See also:** [Lab 02 — The `view!` macro](../02-labs/lab-02.md),
[Lab 03 — Components](../02-labs/lab-03.md),
[Views and markup](../01-concepts/views-and-markup.md), and
[Components and composition](../01-concepts/components-and-composition.md).
