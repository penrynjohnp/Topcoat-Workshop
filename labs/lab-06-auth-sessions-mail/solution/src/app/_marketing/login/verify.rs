use crate::auth::{AppState, begin_session};
use topcoat::{
    Result,
    context::{Cx, app_context},
    router::{error::SeeOther, error::see_other, request::uri, route},
};

#[route(GET)]
async fn verify(cx: &Cx) -> Result<SeeOther> {
    let token = uri(cx)
        .query()
        .and_then(|query| query.strip_prefix("token="))
        .unwrap_or_default();
    let link = app_context::<AppState>(cx)
        .consume_magic_link(token)
        .ok_or_else(topcoat::router::error::not_found)?;
    begin_session(cx, link.email).await?;
    Ok(see_other("/admin"))
}
