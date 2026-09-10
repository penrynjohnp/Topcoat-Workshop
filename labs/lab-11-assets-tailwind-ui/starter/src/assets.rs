use topcoat::{
    asset::{Asset, asset},
    font::{Font, fontsource::fontsource_font},
    icon::iconify,
};

// TODO(lab-11): Declare the local mark and checksummed htmx URL with `asset!`.
pub const SLIPWAY_MARK: Asset = asset!("assets/slipway-mark.svg", rename: "slipway-mark");
pub const HTMX: Asset = asset!(
    "https://cdn.jsdelivr.net/npm/htmx.org@2.0.10/dist/htmx.min.js",
    rename: "htmx",
    checksum: "sha256:71ea67185bfa8c98c39d31717c6fce5d852370fcdfd129db4543774d3145c0de",
);
pub const STYLESHEET: Asset = topcoat::tailwind::stylesheet!();

// TODO(lab-11): Narrow Geist to the weights the app uses and self-host it as assets.
pub const GEIST: Font =
    fontsource_font!(GEIST, weight: [400, 500, 600, 700], style: Normal, host: Asset);

// TODO(lab-11): Include compile-checked Lucide icons from the staged set.
iconify::include!(pub(crate) "lucide:anchor");
iconify::include!(pub(crate) "lucide:search");
iconify::include!(pub(crate) "lucide:plus");
iconify::include!(pub(crate) "lucide:circle-check");
