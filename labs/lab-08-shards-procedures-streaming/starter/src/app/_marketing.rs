mod admin;
mod api;
mod berths;
mod dashboard;
mod login;
mod logout;
mod vessels;

use crate::{
    auth::{AppState, require_auth},
    shared::{self, berth_card},
};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{Slot, layout, page, request::uri},
    runtime::procedure,
    view::{View, component, view},
};

// ANCHOR: runtime-script
#[layout]
async fn root_layout(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    let state: &AppState = app_context(cx);
    Ok(view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <title>"Slipway"</title>
                topcoat::dev::script()
                if state.runtime_script {
                    topcoat::runtime::script()
                }
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
// ANCHOR_END: runtime-script

#[procedure]
pub(crate) async fn mark_work_order_complete(cx: &Cx, id: String) -> Result<bool> {
    require_auth(cx).await?;
    // TODO(lab-08): reject unknown IDs, persist the completion in AppState,
    // and return true only after the server update succeeds.
    let _ = id;
    Ok(false)
}

#[component]
async fn work_orders(cx: &Cx) -> Result<impl View> {
    let email = require_auth(cx).await?;
    // TODO(lab-08): render one component per work order. Give each row a
    // completed signal and call the procedure from an async @click handler.
    Ok(view! {
        <section data-component="work-orders">
            <h2>"Work orders"</h2>
            <p>
                "Private work orders for "
                (email)
            </p>
            <p><a href="/dashboard/history">"Stream work-order history"</a></p>
        </section>
    })
}

// ANCHOR: unsupported-expression
// The runtime vocabulary rejects `match`, so this does not compile. Uncomment it
// inside the button above, read `error: unsupported expression`, then put the
// `if`/`else` spelling back.
//
//     $(match open.get() {
//         true => "Hide work orders",
//         false => "Show work orders",
//     })
//
// Integer literals are rejected for the same reason: every number is an `f64`,
// so `$(count.get() + 1)` fails and `$(count.get() + 1.0)` compiles.
// ANCHOR_END: unsupported-expression

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
