use crate::auth::{require_auth, rotate_session};
use topcoat::{
    Result,
    context::Cx,
    router::{
        error::{SeeOther, see_other},
        route,
    },
};

#[route(POST)]
async fn rotate(cx: &Cx) -> Result<SeeOther> {
    require_auth(cx).await?;
    rotate_session(cx).await?;
    Ok(see_other("/admin"))
}
