use topcoat::{
    asset::{Asset, asset},
    font::{Font, fontsource::fontsource_font},
    icon::iconify,
};

// ANCHOR: asset-declarations
pub const SLIPWAY_MARK: Asset = asset!("assets/slipway-mark.svg", rename: "slipway-mark");
pub const HTMX: Asset = asset!(
    "https://cdn.jsdelivr.net/npm/htmx.org@2.0.10/dist/htmx.min.js",
    rename: "htmx",
    checksum: "sha256:71ea67185bfa8c98c39d31717c6fce5d852370fcdfd129db4543774d3145c0de",
);
pub const STYLESHEET: Asset = topcoat::tailwind::stylesheet!();
// ANCHOR_END: asset-declarations

// ANCHOR: fontsource-font
pub const GEIST: Font = fontsource_font!(
    GEIST,
    weight: [400, 500, 600, 700],
    style: Normal,
    host: Asset,
);
// ANCHOR_END: fontsource-font

// ANCHOR: iconify-icons
iconify::include!(pub(crate) "lucide:anchor");
iconify::include!(pub(crate) "lucide:search");
iconify::include!(pub(crate) "lucide:plus");
iconify::include!(pub(crate) "lucide:circle-check");
// ANCHOR_END: iconify-icons
