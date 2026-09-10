mod history;

use super::work_orders;
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
        work_orders()
    })
}
