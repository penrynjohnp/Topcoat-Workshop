use crate::auth::end_session;
use topcoat::{
    Result,
    context::Cx,
    router::{
        error::{SeeOther, see_other},
        route,
    },
};

#[route(POST)]
async fn logout(cx: &Cx) -> Result<SeeOther> {
    end_session(cx).await?;
    Ok(see_other("/"))
}
