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
        <h1>"Marina dashboard"</h1>
        <p>
            "This page is public, but the embedded work-orders component is protected."
        </p>
        work_orders_component()
    })
}
