mod admin;
mod api;
mod berths;
mod dashboard;
mod login;
mod logout;

use crate::{
    auth::require_auth,
    shared::{self, berth_card},
};
use topcoat::{
    Result,
    context::Cx,
    router::{Slot, layout, page, request::uri},
    view::{View, component, view},
};

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
                    <h1>"Slipway Marina"</h1>
                    shared::nav(current_path: uri(cx).path())
                </header>
                <main>(slot)</main>
                shared::site_footer()
            </body>
        </html>
    })
}

// ANCHOR: protected-work-orders
#[component]
async fn work_orders(cx: &Cx) -> Result<impl View> {
    let email = require_auth(cx).await?;
    Ok(view! {
        <section data-component="work-orders">
            <h2>"Work orders"</h2>
            <p>
                "Private work orders for "
                (email)
            </p>
        </section>
    })
}
// ANCHOR_END: protected-work-orders

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
