mod id;

use crate::shared::berth_list;
use topcoat::{
    Result,
    context::Cx,
    router::page,
    view::{View, view},
};

#[page]
async fn list(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        <h1>"Berths"</h1>
        berth_list()
    })
}
