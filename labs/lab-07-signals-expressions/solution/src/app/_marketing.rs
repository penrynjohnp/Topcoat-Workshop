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
    runtime::{Event, Signal, signal},
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

// ANCHOR: work-order-panel
#[component]
#[rustfmt::skip]
async fn work_orders(cx: &Cx) -> Result<impl View> {
    let email = require_auth(cx).await?;
    let open = signal(cx, || false);

    Ok(view! {
        <section data-component="work-orders">
            <h2>"Work orders"</h2>
            <button
                type="button"
                data-toggle="work-orders"
                :aria-expanded=$(open.get())
                :class=$(if open.get() {
                    "disclosure disclosure-open"
                } else {
                    "disclosure"
                })
                @click=$(|_e| open.toggle())
            >
                $(if open.get() { "Hide work orders" } else { "Show work orders" })
            </button>
            <div id="work-order-panel" data-panel="work-orders" :hidden=$(!open.get())>
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
// ANCHOR_END: work-order-panel

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

// ANCHOR: server-read-signals
#[component]
#[rustfmt::skip]
async fn server_read_demo(cx: &Cx) -> Result<impl View> {
    let tracked = signal(cx, || String::from(""));
    let untracked = signal(cx, || String::from(""));

    Ok(view! {
        <section data-component="server-read-signals">
            <h2>"Server-read signals"</h2>
            <label for="tracked-value">"Tracked value"</label>
            <input
                id="tracked-value"
                :value=$(tracked.get())
                @input=$(|e: Event| tracked.set(e.target.value))
            >
            <label for="untracked-value">"Untracked value"</label>
            <input
                id="untracked-value"
                :value=$(untracked.get())
                @input=$(|e: Event| untracked.set(e.target.value))
            >
            server_read_values(tracked: &tracked, untracked: &untracked)
        </section>
    })
}

#[component]
async fn server_read_values(
    tracked: &Signal<String>,
    untracked: &Signal<String>,
) -> Result<impl View> {
    let tracked_value = tracked.get();
    let untracked_value = untracked.get_untracked();

    Ok(view! {
        <p data-server-read="tracked">
            "Tracked on the server: "
            (tracked_value)
        </p>
        <p data-server-read="untracked">
            "Read without tracking: "
            (untracked_value)
        </p>
    })
}
// ANCHOR_END: server-read-signals

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
