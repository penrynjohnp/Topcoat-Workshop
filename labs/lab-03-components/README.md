# Lab 03 — Components and composition

**Time:** 45 min · **Module:** 1 · **Prerequisites:** Lab 02 complete (or copy `labs/lab-02-view-macro/solution`)

## What you'll learn
- Give a component child content with `#[default] child: Child<'_>` and use it to own the page shell
- Let a component fetch the data it renders instead of receiving it as props
- Accept and merge caller-supplied attributes with `#[default] attrs: Attributes`

## Concepts (read first, 5 min)
Read [Components and composition](https://github.com/penrynjohnp/Topcoat-Workshop/blob/main/docs/src/01-concepts/components-and-composition.md).

A Topcoat component is an ordinary `async fn` with `#[component]` on it, returning
`Result<impl View>`. There is no component base class, no props object, no lifecycle: parameters are
function parameters, and calling one inside `view!` uses named-argument syntax —
`berth_card(slug: "a1")`. Two parameter names are special. A parameter called `child` of type
`Child<'_>` receives whatever extra nodes the caller wrote inside the call, which is how a component
wraps content it knows nothing about. `key` is reserved and cannot be a parameter at all.

Because a component is an `async fn` running on the server, it can *fetch what it renders*. That is
"locality of behaviour": instead of a page loading data and threading it down through three layers of
props, the component that needs a berth asks for the berth. Everything the card does — query, markup,
and later its authorisation check — sits in one file you can read top to bottom. The cost is that the
same data may be asked for several times on one page; Lab 05's `#[memoize]` is the answer to that,
and Lab 10 turns these calls into real queries.

## Steps

Work in `labs/lab-03-components/starter`, and keep `topcoat dev` running:

```bash
cd labs/lab-03-components/starter
topcoat dev
```

### Step 1 — Find the duplication
Open `src/main.rs`. This is your Lab 02 solution, unchanged — nothing has been broken on purpose.
`site_nav`, `site_footer`, `status_badge` and `berth_list` are already components and stay as they
are. Two things are wrong with it, though:

1. `home` and `berths_page` each carry their own copy of `<!DOCTYPE html>`, `<head>`, `<header>` and
   the calls to `site_nav`/`site_footer`. Adding a third page means copying it a third time.
2. The berth card's `<article>` is trapped inside `berth_list`. Nothing else can show a berth.
> ✅ **Checkpoint:** you can point at both problems in the file. `grep -c "<!DOCTYPE html>" src/main.rs`
> prints `2`.

### Step 2 — A `layout` component with child content
Add a component that owns the document and takes the page's own markup as its child:

```rust
#[component]
async fn layout(title: &str, current_path: &str, #[default] child: Child<'_>) -> Result<impl View> {
    Ok(view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <title>(title) " · Slipway"</title>
                topcoat::dev::script()
            </head>
            <body>
                <header>
                    <h1>(title)</h1>
                    site_nav(current_path: current_path)
                </header>
                <main>(child)</main>
                site_footer()
            </body>
        </html>
    })
}
```

Import `Child` from `topcoat::view`. Now both pages collapse to their own content — trailing nodes in
a component call become the `child` argument:

```rust
#[page("/")]
async fn home() -> Result<impl View> {
    Ok(view! {
        layout(
            title: "Slipway",
            current_path: "/",
            <p>"Berths, vessels and work orders for a small marina."</p>
        )
    })
}
```

`#[default]` is what makes `child` optional; without it, every caller would have to pass children.
Note that `current_path` is still a prop threaded through `layout` into `site_nav` — Lab 05 replaces
that with a `cx: &Cx` parameter and `uri(cx).path()`.
> ✅ **Checkpoint:** `grep -c "<!DOCTYPE html>" src/main.rs` now prints `1`, and both pages still
> render with nav and footer.

### Step 3 — Extract `berth_card`
Lift the `<article>` out of `berth_list` into its own component:

```rust
#[component]
async fn berth_card(berth: &Berth) -> Result<impl View> {
    Ok(view! {
        <article class="berth-card" data-berth=(berth.slug) data-length=(berth.length_m.to_string())>
            <h2>
                <a href=(berth.url())>"Berth " (berth.name)</a>
            </h2>
            <p class="berth-length">(berth.length_m) "m"</p>
            status_badge(status: &berth.status)
        </article>
    })
}
```

`berth_list`'s loop body becomes `<li>berth_card(berth: &berth)</li>`.
> ✅ **Checkpoint:** `/berths` looks exactly as it did before. Extracting a component is not supposed
> to change the output.

### Step 4 — Let the card fetch its own berth
Now the interesting change. Add a lookup:

```rust
pub fn find_berth(slug: &str) -> Option<Berth> {
    berths().into_iter().find(|berth| berth.slug == slug)
}
```

and change the card to take a slug instead of a `Berth`:

```rust
#[component]
async fn berth_card(slug: &str) -> Result<impl View> {
    let Some(berth) = find_berth(slug) else {
        return Ok(view! {
            <article class="berth-card berth-card-missing">
                <p>"No berth " (slug) "."</p>
            </article>
        }
        .boxed());
    };

    Ok(view! { /* as before */ }.boxed())
}
```

Two branches return two different anonymous view types, so erase them with `.boxed()` from
`ViewExt` — the same trick recursive components need. Do the same to `berth_list`: drop its `berths`
argument and call `berths()` inside instead, so the pages pass nothing.

This is the locality-of-behaviour trade. The card now owns its query *and* its not-found case, and
any caller with a slug can render one. In exchange, a list of twenty berths performs twenty lookups —
that is the N+1 you will meet properly in Lab 10, and `#[memoize]` in Lab 05 is how Topcoat takes the
sting out of it.
> ✅ **Checkpoint:** `berths_page` now reads `layout(title: "Berths", current_path: "/berths", berth_list())`
> — no data passed anywhere — and `/berths` is unchanged.

### Step 5 — A second page that reuses the card
Add a detail page and call the card from it exactly as the home page does:

```rust
#[page("/berths/featured")]
async fn featured_berth_page() -> Result<impl View> {
    Ok(view! {
        layout(title: "Featured berth", current_path: "/berths", berth_card(slug: FEATURED))
    })
}
```

(Add `const FEATURED: &str = "a1";`, and call `berth_card(slug: FEATURED)` on the home page too. A
real `/berths/{slug}` route needs path parameters, which is Lab 04.)
> ✅ **Checkpoint:** view source on `/` and on `/berths/featured`. The `<article>` for berth A1 is the
> same markup on both — same classes, same `data-` attributes, same children. Run
> `../../../scripts/check-lab.sh 03`; the solution's test asserts exactly this.

### Step 6 — `attrs:` pass-through
A caller often needs to decorate a component without the component knowing why. Add an attributes
parameter and spread it:

```rust
#[component]
async fn berth_card(slug: &str, #[default] attrs: Attributes) -> Result<impl View> {
    let mut attrs = attrs;
    let caller_class = attrs.remove("class");
    // ...
    <article
        class=(class!("berth-card", caller_class))
        data-berth=(berth.slug)
        data-length=(berth.length_m.to_string())
        (attrs)
    >
```

Taking `class` out of the collection first and passing it to `class!` as an entry *merges* the
caller's classes with the card's own — spreading it directly would replace `berth-card` instead. Call
it from the detail page with `attrs: attributes! { class="berth-card-detail" data-detail="" }`.
> ✅ **Checkpoint:** the detail card's class is `berth-card berth-card-detail` (in some order) and it
> carries `data-detail`; the cards on `/berths` carry neither.

### Step 7 — A slot on the card
Give the card child content so a page can add to it without editing it:

```rust
#[component]
async fn berth_card(
    slug: &str,
    #[default] attrs: Attributes,
    #[default] child: Child<'_>,
) -> Result<impl View> {
```

Render `(child)` as the last node inside the `<article>`, then pass a note from the detail page:

```rust
berth_card(
    slug: "a2",
    attrs: attributes! { class="berth-card-detail" data-detail="" },
    <p class="berth-note">"Ask at the office about long-stay rates."</p>
)
```
> ✅ **Checkpoint:** the note appears on that one card only. Every other card is untouched, and the
> Step 5 checkpoint still holds.

### Step 8 — Optional and converted props
Try the remaining parameter attributes on `berth_card`:

- `#[default]` — the parameter falls back to `Default::default()`.
- `#[default(expr)]` — a custom fallback, evaluated only when the argument is omitted. The type does
  not have to implement `Default`.
- `#[into]` — callers may pass anything that converts via `Into`, converted outside the function body
  so the component is not monomorphised per caller.

Then try adding a parameter called `key` and read the compile error: `key:` is reserved on component
calls for keying an invocation's identity.
> ✅ **Checkpoint:** `cargo build` is clean, and you can explain when you would reach for each of the
> three attributes. Compare with `../solution/src/lib.rs`.

## Stretch goals
- Give `layout` its own `#[default] attrs: Attributes` and use it to put a page-specific class on
  `<body>`.
- Write a recursive component — a nested comment thread, or the `countdown` from the docs — and work
  out from the compile error why one of the components in the cycle must `.boxed()`.
- Make `status_badge` generic over anything that can describe itself, and see what `Send + Sync`
  bounds the compiler asks for.
- Split the file into modules (`components.rs`, `pages.rs`) and check that `discover()` still finds
  every page. Lab 04 makes the module tree meaningful.

## Troubleshooting
- **`missing argument child`** — the parameter needs `#[default]` to be optional.
- **`` `impl Trait` … opaque type ``, or two arms returning different types** — a component with two
  `view!` returns needs `.boxed()` on both (import `ViewExt`).
- **The caller's `class` replaced the component's** — you spread `attrs` without removing `class`
  first. Take it out with `attrs.remove("class")` and pass it to `class!` as an entry.
- **`use of moved value: attrs`** — spreading an `Attributes` consumes it. `clone()` if you need it
  twice.
- **Attribute order changes between renders** — expected. Spreading an `Attributes` value routes that
  element's attributes through a map, and 0.7.0 documents that render order is not guaranteed. Assert
  on the set of attributes, not on a byte-for-byte string, as the solution's test does.
- **`cannot find value cx`** — components only get the request context if they declare `cx: &Cx`.
  That is Lab 05.

## What's next
Lab 04 gets rid of the hard-coded `/berths/featured` route: explicit routing first, then module-based
routing with `.discover()`, path parameters and nested layouts.
