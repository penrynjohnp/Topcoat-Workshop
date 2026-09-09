# Views and markup

Topcoat has no template language. `view!` is a macro that parses HTML at compile time and lowers it
into ordinary Rust statements that append to a view value. Three consequences follow, and they
explain nearly every surprise the macro produces.

## It really is HTML

Tags must nest and close as HTML says they do; void elements (`<meta>`, `<input>`, `<br>`) have no
closing tag. Attribute names keep their hyphens — `aria-current`, `data-berth` — because they are
parsed as HTML attribute names, not Rust identifiers. Mistyped markup is a compile error rather than
broken output in a browser.

## Bare words are Rust, so text is quoted

Inside `view!`, an unquoted word is Rust. Literal text is therefore a string literal, and a dynamic
value is a parenthesised expression:

```rust
{{#include ../../../labs/lab-02-view-macro/solution/src/lib.rs:status_badge}}
```

The same parentheses work in attribute-value position (`href=(berth.url())`), for dynamic attribute
names, and for dynamic element names. Every interpolated value is escaped on the way out, so a
vessel named `Rock & Roll` is safe without any thought from you.

## Control flow is Rust control flow with markup bodies

`if`, `else`, `for` and `match` inside `view!` compile to the constructs they look like. Pattern
bindings are in scope inside the arm, moves and borrows work as usual, and there is no separate
expression vocabulary to learn — this is the *server-side* markup, quite unlike the deliberately
restricted `$(...)` expressions of [Lab 07](../02-labs/lab-07.md), which must also compile to
JavaScript.

```rust
{{#include ../../../labs/lab-02-view-macro/solution/src/lib.rs:berth_list}}
```

Control flow works in attribute position too, where each branch emits attributes instead of child
nodes.

## Attributes can remove themselves

A literal attribute is always present. An *expression* attribute is omitted entirely when its value
is `false` or `None`, and renders with an empty value when it is `true`. That gives HTML's boolean
attributes (`disabled`, `required`) exactly the presence semantics they are specified to have, and
extends the same rule to attributes that carry a value:

```rust
{{#include ../../../labs/lab-02-view-macro/solution/src/lib.rs:nav}}
```

Beware the attributes that only *look* boolean. `aria-expanded` and `contenteditable` are
*enumerated* attributes: for them `"false"` means something different from being absent, so pass the
strings, not a `bool`.

## Views are lazy

A `view!` value is not a rendered string; it is a description that renders when it becomes a
response, or when it is interpolated into another view. Components are `async fn`s, so sibling
components in a single view can render concurrently. That is what makes "the component fetches its
own data" — the locality-of-behaviour argument in [Lab 03](../02-labs/lab-03.md) — affordable.

## Formatting

`rustfmt` treats a `view!` body as an opaque token tree and will not touch it. `topcoat fmt` is the
formatter that understands the markup. Run both; CI runs both.

See also the [`view!` cheat sheet](../03-reference/view-cheatsheet.md).
