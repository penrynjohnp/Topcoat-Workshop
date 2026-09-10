mod rotate;

use crate::auth::{refresh_session, require_auth};
use topcoat::{
    Result,
    context::Cx,
    router::page,
    view::{View, view},
};

// ANCHOR: admin-page
#[page]
async fn admin(cx: &Cx) -> Result<impl View> {
    let email = require_auth(cx).await?;
    refresh_session(cx).await?;
    Ok(view! {
        <h1>"Admin"</h1>
        <p>
            "Signed in as "
            (email)
        </p>
        <form method="post" action="/logout">
            <button type="submit">"Log out"</button>
        </form>
    })
}
// ANCHOR_END: admin-page
