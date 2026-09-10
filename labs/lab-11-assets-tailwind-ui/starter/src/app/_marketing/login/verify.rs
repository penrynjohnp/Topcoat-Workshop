use crate::auth::{AppState, begin_session, consume_magic_link};
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
    let link = consume_magic_link(app_context::<AppState>(cx), token)
        .await?
        .ok_or_else(topcoat::router::error::not_found)?;
    begin_session(cx, link.email).await?;
    Ok(see_other("/admin"))
}
