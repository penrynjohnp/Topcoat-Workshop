mod admin;
mod api;
mod berths;
mod dashboard;
mod login;
mod logout;
mod vessels;

use crate::{
    auth::{AppState, require_auth},
    models::WorkOrder,
    shared::{self, berth_card, work_order_exists},
};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{Slot, layout, page, request::uri},
    runtime::{procedure, signal},
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
                // ANCHOR: htmx-script
                // Pinned, not floating: an htmx major bump changes attribute
                // semantics. Lab 11 vendors this with `asset!` so it is served
                // from our own origin.
                <script
                    src="https://cdn.jsdelivr.net/npm/htmx.org@2.0.10/dist/htmx.min.js"
                    defer="defer"
                ></script>
                // ANCHOR_END: htmx-script
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

// ANCHOR: complete-work-order-procedure
#[procedure]
pub(crate) async fn mark_work_order_complete(cx: &Cx, id: f64) -> Result<bool> {
    require_auth(cx).await?;
    if !id.is_finite() || id.fract() != 0.0 || id < 0.0 || id > u64::MAX as f64 {
        return Ok(false);
    }

    let id = id as u64;
    if !work_order_exists(cx, id).await? {
        return Err(
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "unknown work order").into(),
        );
    }
    let mut db = app_context::<AppState>(cx).db.clone();
    let mut order = WorkOrder::get_by_id(&mut db, &id).await?;
    toasty::update!(order { completed: true })
        .exec(&mut db)
        .await?;
    Ok(true)
}
// ANCHOR_END: complete-work-order-procedure

#[component]
async fn work_order_row(cx: &Cx, order: &WorkOrder) -> Result<impl View> {
    let id = order.id as f64;
    let completed_initially = order.completed;
    let completed = signal(cx, || completed_initially);

    Ok(view! {
        <li
            id=(order.id)
            :class=$(if completed.get() { "work-order complete" } else { "work-order" })
        >
            <span>(&order.title)</span>
            <button
                type="button"
                :disabled=$(completed.get())
                @click=$(async |_event| {
                    let saved = mark_work_order_complete(id).await;
                    completed.set(saved);
                })
            >
                $(if completed.get() { "Complete" } else { "Mark complete" })
            </button>
        </li>
    })
}

// ANCHOR: work-order-procedure-ui
#[component]
async fn work_orders(cx: &Cx) -> Result<impl View> {
    let email = require_auth(cx).await?;
    let orders = shared::work_orders(cx).await?;
    Ok(view! {
        <section data-component="work-orders">
            <h2>"Work orders"</h2>
            <p>
                "Private work orders for "
                (email)
            </p>
            <ul>
                for order in orders {
                    work_order_row(order: &order, key: order.id)
                }
            </ul>
            <p><a href="/dashboard/work-orders/new">"Create a work order"</a></p>
            <p><a href="/dashboard/history">"Stream work-order history"</a></p>
        </section>
    })
}
// ANCHOR_END: work-order-procedure-ui

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
