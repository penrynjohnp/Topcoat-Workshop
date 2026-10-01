# Assets, styling, and owned UI

Topcoat's asset system joins Rust declarations to deployable files. `asset!` leaves a compact declaration in the binary; the bundler scans that binary, obtains each local or remote source, and writes a content-hashed filename plus a manifest. At runtime the manifest turns the handle into a URL.

```rust
{{#include ../../../labs/lab-11-assets-tailwind-ui/solution/src/assets.rs:asset-declarations}}
```

This makes the binary and bundle inseparable. A debug manifest cannot safely describe a release binary, and a bundle from another checkout may use different asset identifiers. Build and bundle the same profile, then ship both outputs together.

## One pipeline, several producers

Tailwind generates CSS during Cargo's build script without Node. Iconify stages metadata there too, while Fontsource turns selected font files into ordinary assets:

```rust
{{#include ../../../labs/lab-11-assets-tailwind-ui/solution/build.rs:asset-build-script}}
```

```rust
{{#include ../../../labs/lab-11-assets-tailwind-ui/solution/src/assets.rs:fontsource-font}}
```

Iconify icons themselves render inline as SVG, so they inherit text color and add no browser request. Font files and Tailwind output are bundled, content-hashed files.

## Vendoring means ownership

`topcoat ui add` copies component source into the application rather than adding an opaque UI runtime. The local card is therefore free to change:

```rust
{{#include ../../../labs/lab-11-assets-tailwind-ui/solution/src/components/card.rs:owned-card-style}}
```

The component keeps using semantic theme tokens, while the installed `styles.css` chooses the actual palette. Updating a vendored component is source maintenance: inspect registry changes, overwrite deliberately, and reapply local decisions where they still belong.

CLI 0.9.0 refreshes owned components with `topcoat ui add --overwrite`, not `topcoat ui update`.
Retain custom card classes and merge required theme tokens.
The install-state hash records the registry baseline rather than pretending local customization is upstream source.

## Fields Preserve Server Validation

Compose the title and berth controls with the owned `field`, `field_label`, and `field_error` components.
You still validate on the server and choose when an error appears.
Link the feedback with `aria-describedby` and keep control names stable:

```rust
{{#include ../../../labs/lab-11-assets-tailwind-ui/solution/src/app/_marketing/dashboard/work_orders/new.rs:work-order-form}}
```

*Guide basis: the v0.9.0 [UI guide](https://raw.githubusercontent.com/tokio-rs/topcoat/v0.9.0/crates/topcoat/docs/ui.md)
and [field source](https://raw.githubusercontent.com/tokio-rs/topcoat/v0.9.0/crates/topcoat-ui/registry/src/components/field.rs).*

The result remains a server-rendered Topcoat app. Tailwind produces CSS, fonts become assets, icons become inline markup, and UI components remain Rust functions in the repository; none of these choices creates a separate client application or page-data API.

**See also:** [Lab 11](../02-labs/lab-11.md), [Lab 12](../02-labs/lab-12.md),
[CLI commands](../03-reference/cli.md), and [How expressions reach the browser](dual-expressions.md).
