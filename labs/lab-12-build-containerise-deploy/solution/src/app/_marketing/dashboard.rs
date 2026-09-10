mod history;
mod work_orders;

use super::work_orders as work_orders_component;
use topcoat::{
    Result,
    router::page,
    view::{View, view},
};

#[page]
async fn dashboard() -> Result<impl View> {
    Ok(view! {
        <p class="text-sm font-semibold uppercase tracking-[0.2em] text-cyan-700">
            "Operations"
        </p>
        <h1 class="mt-2 text-3xl font-bold tracking-tight text-cyan-950">
            "Marina dashboard"
        </h1>
        <p class="mt-3 max-w-2xl text-slate-600">
            "This page is public, but the embedded work-orders component protects itself."
        </p>
        work_orders_component()
    })
}
