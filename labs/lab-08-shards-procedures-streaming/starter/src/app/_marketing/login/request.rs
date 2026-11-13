use crate::auth::{AppState, remember_email};
use topcoat::{
    Result,
    context::{Cx, context},
    mail::{Mail, Mailbox, send},
    router::{request::uri, route},
    view::view,
};

// ANCHOR: magic-link
#[route(GET)]
async fn request_link(cx: &Cx) -> Result<String> {
    let email = uri(cx)
        .query()
        .and_then(|query| query.strip_prefix("email="))
        .unwrap_or("captain@example.com")
        .to_owned();
    let token = context::<AppState>(cx).new_magic_link(email.clone());
    remember_email(cx, &email);
    let link = format!("/login/verify?token={token}");
    let response_link = link.clone();
    let message = Mail::builder()
        .from(Mailbox::named("Slipway Marina", "no-reply@slipway.test")?)
        .to([Mailbox::new(email.as_str())?])
        .subject("Your Slipway magic link")
        .html(
            cx,
            view! {
                <h1>"Log in to Slipway"</h1>
                <p><a href=(response_link)>"Use this magic link"</a></p>
            },
        )
        .await?
        .build()?;
    send(cx, message).await?;
    Ok(format!("Magic link sent. Open {link}"))
}
// ANCHOR_END: magic-link
