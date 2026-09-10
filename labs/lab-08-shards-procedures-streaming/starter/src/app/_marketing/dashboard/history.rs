use crate::auth::require_auth;
use topcoat::{
    Result,
    context::Cx,
    router::page,
    view::{View, view},
};

#[page]
async fn history(cx: &Cx) -> Result<impl View> {
    require_auth(cx).await?;
    // TODO(lab-08): add a direct live!/emit! progress region, then wrap a slow
    // history component in suspense and error_boundary. Use ?fail=true to
    // exercise the fallback deterministically.
    Ok(view! {
        <h1>"Work-order history"</h1>
        <p>"History goes here."</p>
    })
}
