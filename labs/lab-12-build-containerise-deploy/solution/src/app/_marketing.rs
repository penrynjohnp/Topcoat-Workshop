mod admin;
mod api;
mod berths;
mod dashboard;
mod login;
mod logout;
mod vessels;

use crate::{
    assets,
    auth::{AppState, require_auth},
    components::{
        button::{ButtonSize, ButtonVariant, button_variants},
        card::{card, card_content, card_header, card_title},
    },
    models::WorkOrder,
    shared::{self, berth_card, work_order_exists},
};
use topcoat::{
    Result,
    context::{Cx, app_context},
    icon::icon,
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
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <title>"Slipway Marina"</title>
                topcoat::dev::script()
                // ANCHOR: asset-head
                if state.runtime_script {
                    topcoat::font::link(font: assets::GEIST)
                    <link rel="stylesheet" href=(assets::STYLESHEET)>
                    topcoat::runtime::script()
                    <script type="module" src=(assets::HTMX)></script>
                }
                // ANCHOR_END: asset-head
            </head>
            <body>
                <header
                    class="border-b border-cyan-900/10 bg-white/75 backdrop-blur-lg"
                >
                    <div
                        class="mx-auto flex max-w-6xl flex-col gap-4 px-5 py-5 sm:flex-row sm:items-center sm:justify-between"
                    >
                        <a
                            href="/"
                            class="flex items-center gap-3 text-xl font-bold tracking-tight text-cyan-950"
                        >
                            if state.runtime_script {
                                <img src=(assets::SLIPWAY_MARK) alt="" class="size-11">
                            } else {
                                icon(
                                    data: assets::ANCHOR,
                                    attrs: topcoat::view::attributes! { class="size-8 text-cyan-700" }
                                )
                            }
                            <span>"Slipway Marina"</span>
                        </a>
                        shared::nav(current_path: uri(cx).path())
                    </div>
                </header>
                <main class="mx-auto min-h-[70vh] max-w-6xl px-5 py-10">(slot)</main>
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
            :class=$(if completed.get() {
                "work-order complete flex items-center justify-between gap-4 rounded-xl bg-emerald-50 p-4 text-emerald-900"
            } else {
                "work-order flex items-center justify-between gap-4 rounded-xl border border-cyan-900/10 bg-cyan-50/50 p-4"
            })
        >
            <span>(&order.title)</span>
            <button
                type="button"
                :disabled=$(completed.get())
                @click=$(async |_event| {
                    let saved = mark_work_order_complete(id).await;
                    completed.set(saved);
                })
                class="inline-flex h-9 items-center gap-2 rounded-lg border border-cyan-900/10 bg-white px-3 text-sm font-medium shadow-xs disabled:opacity-60"
            >
                icon(data: assets::CIRCLE_CHECK)
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
        card(
            attrs: topcoat::view::attributes! { class="mt-7" },
            <section data-component="work-orders">
                card_header(
                    card_title(
                        attrs: topcoat::view::attributes! { class="text-xl text-cyan-950" },
                        "Work orders"
                    )
                    <p class="text-sm text-slate-600">
                        "Private work orders for "
                        (email)
                    </p>
                )
                card_content(
                    <ul class="grid gap-3">
                        for order in orders {
                            work_order_row(order: &order, key: order.id)
                        }
                    </ul>
                    <div class="mt-6 flex flex-wrap gap-3">
                        <a
                            href="/dashboard/work-orders/new"
                            class=(button_variants(
                                ButtonVariant::Primary,
                                ButtonSize::Md,
                            ))
                        >
                            icon(data: assets::PLUS)
                            "Create a work order"
                        </a>
                        <a
                            href="/dashboard/history"
                            class=(button_variants(
                                ButtonVariant::Outline,
                                ButtonSize::Md,
                            ))
                        >
                            "Stream work-order history"
                        </a>
                    </div>
                )
            </section>
        )
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
            <section class="grid items-center gap-8 py-8 lg:grid-cols-[1.2fr_0.8fr]">
                <div>
                    <p
                        class="text-sm font-semibold uppercase tracking-[0.2em] text-cyan-700"
                    >
                        "Marina operations"
                    </p>
                    <h1
                        class="mt-3 text-4xl font-bold tracking-tight text-cyan-950 sm:text-5xl"
                    >
                        "Calm water. Clear work."
                    </h1>
                    <p class="mt-5 max-w-2xl text-lg leading-8 text-slate-600">
                        "Berths, vessels and work orders for a small marina, rendered on the server and styled without Node."
                    </p>
                    <div class="mt-7 flex flex-wrap gap-3">
                        <a
                            href="/berths"
                            class=(button_variants(
                                ButtonVariant::Primary,
                                ButtonSize::Lg,
                            ))
                        >
                            "Browse berths"
                        </a>
                        <a
                            href="/dashboard"
                            class=(button_variants(
                                ButtonVariant::Outline,
                                ButtonSize::Lg,
                            ))
                        >
                            "Open dashboard"
                        </a>
                    </div>
                </div>
                <div>
                    <h2
                        class="mb-3 text-sm font-semibold uppercase tracking-[0.16em] text-slate-500"
                    >
                        "Featured berth"
                    </h2>
                    berth_card(slug: shared::FEATURED)
                </div>
            </section>
        },
    )
}
