use std::time::Duration;

use topcoat::{
    cookie::CookieLayer,
    mail::{FileTransport, Mailer},
    session::SessionLifetime,
};

use crate::auth::AppState;

mod _marketing;

// ANCHOR: app-router
pub fn router(state: AppState) -> topcoat::router::Router {
    topcoat::router::module_router!()
        .layer(CookieLayer::new())
        .with(SessionLifetime(Duration::from_secs(3600)))
        .with(Mailer::new(FileTransport::new("mail")))
        .with(state)
        .build()
}
// ANCHOR_END: app-router
