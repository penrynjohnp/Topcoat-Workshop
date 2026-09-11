mod api;
mod berths;

use crate::shared::{self, berth_card};
use topcoat::{
    Result,
    context::Cx,
    router::{Slot, layout, page, request::uri},
    view::{View, view},
};

// ANCHOR: group-layout-home
#[layout]
async fn root_layout(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    Ok(view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <title>"Slipway"</title>
                topcoat::dev::script()
            </head>
            <body>
                <header>
                    <h1>"Slipway"</h1>
                    shared::nav(current_path: uri(cx).path())
                </header>
                <main>(slot)</main>
                shared::site_footer()
            </body>
        </html>
    })
}

#[page]
#[rustfmt::skip]
async fn home() -> Result<impl View> {
    Ok(
        view! {
            <p>"Berths, vessels and work orders for a small marina."</p>
            <h2>"Featured berth"</h2>
            berth_card(slug: shared::FEATURED)
        },
    )
}
// ANCHOR_END: group-layout-home
