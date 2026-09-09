mod request;
mod verify;

use topcoat::{
    Result,
    router::{page, request::uri},
    view::{View, view},
};

#[page]
async fn login(cx: &topcoat::context::Cx) -> Result<impl View> {
    let email = uri(cx)
        .query()
        .and_then(|query| query.strip_prefix("email="))
        .unwrap_or("captain@example.com");
    Ok(view! {
        <h1>"Log in"</h1>
        <p>
            "Request a magic link for "
            (email)
        </p>
        <a href=(format!("/login/request?email={email}"))>"Send magic link"</a>
    })
}
