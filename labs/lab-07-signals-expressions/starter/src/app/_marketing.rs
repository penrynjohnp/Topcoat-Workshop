mod admin;
mod api;
mod berths;
mod dashboard;
mod login;
mod logout;

use crate::{
    auth::{AppState, require_auth},
    shared::{self, berth_card},
};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{Slot, layout, page, request::uri},
    view::{View, component, view},
};

// ANCHOR: runtime-script
#[layout]
async fn root_layout(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    let _state: &AppState = app_context(cx);
    Ok(view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <title>"Slipway"</title>
                topcoat::dev::script()
                // TODO(lab-07): render `topcoat::runtime::script()` when
                // `state.runtime_script` is set — interactive pages need it.
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

#[component]
async fn work_orders(cx: &Cx) -> Result<impl View> {
    let email = require_auth(cx).await?;
    // TODO(lab-07): create `open` here with `signal(cx, || false)` before
    // entering `view!`, then add a disclosure button with `@click`, `:class`
    // and `:aria-expanded`, and hide the panel with `:hidden`.
    Ok(view! {
        <section data-component="work-orders">
            <h2>"Work orders"</h2>
            <div id="work-order-panel" data-panel="work-orders">
                <p>
                    "Private work orders for "
                    (email)
                </p>
                <ul>
                    <li>"Anti-foul A1 · due Friday"</li>
                    <li>"Replace C3 mooring line"</li>
                </ul>
            </div>
        </section>
    })
}

// TODO(lab-07): once the button works, paste the `match` version of its label into
// the expression and read the compile error, then restore the `if`/`else` spelling.

#[component]
async fn server_read_demo(cx: &Cx) -> Result<impl View> {
    let _ = cx;
    // TODO(lab-07): create tracked and untracked signals, add controls that update
    // them, then pass both to a child component as `&Signal<String>`. Read one with
    // `.get()` and the other with `.get_untracked()` in that child's Rust body.
    Ok(view! {
        <section data-component="server-read-signals">
            <h2>"Server-read signals"</h2>
            <p>"Add the tracked and untracked signal demonstration here."</p>
        </section>
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
            server_read_demo()
        },
    )
}
