# Lab 02 — The `view!` macro

**Time:** 60 min · **Module:** 1 · **Prerequisites:** Lab 01 complete (or copy `labs/lab-01-hello-topcoat/solution`)

## What you'll learn
- Write HTML-faithful markup in `view!` and interpolate Rust with `(expr)`
- Drive markup from data with `for`, `if` and `match`, including in attribute position
- Build class lists and reusable attribute sets with `class!` and `attributes!`

## Concepts (read first, 5 min)
Read [Views and markup](https://github.com/penrynjohnp/Topcoat-Workshop/blob/main/docs/src/01-concepts/views-and-markup.md).

`view!` is not a string template. It parses real HTML, so tags must nest and close the way HTML says
they do, and the macro lowers your markup into ordinary Rust statements that append to a view. That
is why control flow inside `view!` is just Rust control flow with markup bodies: `if`, `else`, `for`
and `match` mean exactly what they mean elsewhere, and the borrow checker still applies.

The one place it deviates from HTML is text: bare words are Rust, so literal text is written as a
quoted string (`<h1>"Berths"</h1>`) and dynamic values are parenthesised (`<h1>(berth.name)</h1>`).
The same parentheses work in attribute values, and an expression attribute that evaluates to `false`
or `None` removes itself from the output entirely — which is exactly what a conditional attribute
like `aria-current` needs.

## Steps

Work in `labs/lab-02-view-macro/starter`. Run the dev server in one terminal and leave it there:

```bash
cd labs/lab-02-view-macro/starter
topcoat dev
```

### Step 1 — Read the shell
Open `src/main.rs`. There are two `#[page]`s (`/` and `/berths`), a `site_nav`, a `site_footer`, a
`status_badge` and a `berth_list`, plus a `Berth` struct, a `Status` enum and a `berths()` function
that returns a `Vec<Berth>` literal. Everything is markup — no database, no request context.

Note the shape of the markup: `<!DOCTYPE html>`, a real `<html>`/`<head>`/`<body>` tree, and every
piece of literal text in quotes.
> [!NOTE]
> **Checkpoint:** http://127.0.0.1:3000 and http://127.0.0.1:3000/berths both render, with a nav
> and a footer on each. The berth list shows a single hard-coded card.

### Step 2 — Interpolate the data
In `berth_list`, replace the hard-coded card's text and URL with expressions:

```rust
<h2>
    <a href=(berth.url())>"Berth " (berth.name)</a>
</h2>
<p class="berth-length">(berth.length_m) "m"</p>
```

(For now, add `let berth = &berths[0];` above the `view!` so it compiles; Step 3 removes it.)
`(expr)` interpolates in child position *and* in attribute-value position — same syntax, and both
escape their output, so a vessel called `Rock & Roll` renders safely without you thinking about it.
> [!NOTE]
> **Checkpoint:** the single card now reads "Berth A1", "8m", and links to `/berths/a1`.

### Step 3 — Render the list with `for`
Delete the `let berth = ...` line and wrap the `<li>` in a loop:

```rust
<ul class="berths">
    for berth in berths {
        <li>
            ...
        </li>
    }
</ul>
```

Do the same in `site_nav`, looping over the `NAV` array so the links come from data:

```rust
for (href, label) in NAV {
    <li>
        <a href=(href) class="nav-link">(label)</a>
    </li>
}
```

Delete the `let _ = ...` lines the starter used to keep the unused arguments quiet. `for` in `view!`
is Rust's `for`: `berths` is moved into the loop, so if you need it afterwards, iterate `&berths`.
> [!NOTE]
> **Checkpoint:** all four berths render, and the nav still shows Home and Berths.

### Step 4 — Choose markup with `if` and `match`
Give `status_badge` a `match` over the status, so each variant picks its own label:

```rust
match status {
    Status::Vacant => {
        <span class="badge">"Vacant"</span>
    }
    Status::Occupied { vessel } => {
        <span class="badge">"Occupied by " (vessel)</span>
    }
    Status::Maintenance => {
        <span class="badge">"Maintenance"</span>
    }
}
```

Pattern bindings like `vessel` are in scope inside the arm's markup. Then guard the empty case in
`berth_list`:

```rust
if berths.is_empty() {
    <p class="empty">"No berths yet."</p>
} else {
    <ul class="berths">
        ...
    </ul>
}
```
> [!NOTE]
> **Checkpoint:** A1 shows "Occupied by Lady Jane" and B7 shows "Maintenance". Temporarily change
> `berths()` to `vec![]` and the page reads "No berths yet." — then change it back.

### Step 5 — The conditional attribute
`site_nav` already receives `current_path`. Mark the current link:

```rust
<a
    href=(href)
    class="nav-link"
    aria-current=((href == current_path).then_some("page"))
>
    (label)
</a>
```

`then_some` gives `Some("page")` or `None`, and an expression attribute whose value is `None` (or
`false`) is omitted from the HTML entirely — the attribute is *absent*, not empty, which is what
assistive technology expects. The attribute-position form is available too when a condition should
emit several attributes at once:

```rust
if href == current_path {
    aria-current="page"
    data-current=""
}
```
> [!NOTE]
> **Checkpoint:** view source on `/`. The Home link has `aria-current="page"` and the Berths link
> has no `aria-current` at all. On `/berths` it is the other way round.

### Step 6 — Class lists with `class!`
Styling the active link means a *conditional class*, not a conditional attribute. Import `class` and
use it in the value position:

```rust
class=(class!("nav-link", "nav-link-active" if href == current_path))
```

Do the same for the badges, giving each `match` arm its own modifier:
`class!("badge", "badge-vacant")`, `badge-occupied`, `badge-maintenance`. An absent entry — a
`None`, an empty string, or one whose condition is false — contributes neither text nor a separator,
so you never get the double spaces that string concatenation produces. If every entry is absent, the
`class` attribute is dropped altogether.
> [!NOTE]
> **Checkpoint:** on `/`, the Home link's class is exactly `nav-link nav-link-active` and the
> Berths link's is exactly `nav-link` — no trailing space.

### Step 7 — Attribute sets with `attributes!`
The berth card needs a few `data-*` attributes. Build them as a value and spread it:

```rust
<article (attributes! {
    class="berth-card"
    data-berth=(berth.slug)
    data-length=(berth.length_m.to_string())
})>
```

`attributes!` takes the same syntax as attributes inside an element and produces an `Attributes`
value: ordinary Rust data you can build in a helper, insert into, pass through several layers, and
finally spread into an element. Lab 03 uses this to forward caller-supplied attributes through a
component.
> [!NOTE]
> **Checkpoint:** view source — each `<article>` carries `data-berth` and `data-length` with the
> right values.

### Step 8 — `topcoat fmt`
Mangle the indentation inside one `view!` block, then:

```bash
cargo fmt
topcoat fmt src/main.rs
git diff
```

`cargo fmt` leaves `view!` bodies alone — to rustfmt they are opaque token trees. `topcoat fmt` is
the formatter that understands them, so run both. Note that CLI 0.8.0 has no `--check` flag, which
is why CI formats and then runs `git diff --exit-code` (see `COMPATIBILITY.md`).
> [!NOTE]
> **Checkpoint:** `git diff` shows your markup restored to the canonical layout. Compare your file
> with `../solution/src/lib.rs`, then run `../../../scripts/check-lab.sh 02` for the full build,
> test and lint.

## Stretch goals
- Pull the card markup out into a `berth_card` component that takes `berth: &Berth`. That is Lab 03's
  opening move — see how far you get before needing child content.
- Add a `match` guard (`Status::Occupied { vessel } if vessel.starts_with("Lady") => ...`) and give
  those berths an extra class.
- Add an `Option<&str>` note to `Berth` and render `title=(berth.note)` — confirm the attribute
  disappears for berths without one.
- Render the nav's `aria-expanded` state. It is an *enumerated* attribute, not a boolean one, so it
  needs the strings `"true"`/`"false"`, not a `bool`. Work out why from the rendered HTML.

## Troubleshooting
- **`expected one of ... found "Berths"`** — literal text needs quotes: `<h1>"Berths"</h1>`. Bare
  words inside `view!` are parsed as Rust.
- **The class list renders as `nav-link nav-link-active"` or similar** — you put `class!` in the
  attribute *name* position or forgot the parentheses. It belongs in the value: `class=(class!(...))`.
- **`cannot borrow ... after move`** — `for berth in berths` moves the `Vec`. Iterate `&berths` if
  you need it again afterwards.
- **The attribute renders as `aria-current=""` when you wanted it gone** — you passed `true`, not
  `Some(...)`/`None`. `true` renders a present, empty attribute; `false` and `None` remove it.
- **`topcoat: command not found`** — `cargo install topcoat-cli --version 0.8.0 --locked`.

## What's next
Lab 03 turns this one-file page into composed components — `layout`, `nav`, `berth_card` — with
arguments, child content and `attrs:` pass-through.
