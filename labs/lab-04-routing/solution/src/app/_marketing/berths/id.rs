use crate::shared::{self, berth_card};
use topcoat::{
    Result,
    context::Cx,
    router::{page, path_param},
    view::{View, view},
};

// ANCHOR: path-param
path_param!(id);

#[page]
async fn detail(cx: &Cx) -> Result<impl View> {
    let id = path_param::<Id>(cx);
    if shared::find_berth(id).is_none() {
        return Err(topcoat::router::error::not_found().into());
    }
    Ok(view! {
        <h1>"Berth detail"</h1>
        berth_card(slug: id)
    })
}
// ANCHOR_END: path-param
