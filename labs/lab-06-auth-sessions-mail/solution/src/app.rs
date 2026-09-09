use std::time::Duration;

use topcoat::{
    cookie::RouterBuilderCookieExt,
    mail::{FileTransport, MailConfig, RouterBuilderMailExt},
    session::{RouterBuilderSessionExt, SessionConfig},
};

use crate::auth::AppState;

mod _marketing;

// ANCHOR: app-router
pub fn router(state: AppState) -> topcoat::router::Router {
    topcoat::router::module_router!()
        .cookies()
        .sessions(
            SessionConfig::builder()
                .lifetime(Duration::from_secs(3600))
                .build(),
        )
        .mail(
            MailConfig::builder()
                .transport(FileTransport::new("mail"))
                .build(),
        )
        .app_context(state)
        .build()
}
// ANCHOR_END: app-router
