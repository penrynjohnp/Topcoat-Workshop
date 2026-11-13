use std::time::Duration;

use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    cookie::CookieLayer,
    mail::{FileTransport, Mailer},
    session::SessionLifetime,
};

use crate::auth::AppState;

mod _marketing;

// ANCHOR: app-router
pub fn router(state: AppState, assets: Option<AssetBundle>) -> topcoat::router::Router {
    // TODO(lab-07): flag the runtime script when an asset bundle was loaded, so the
    // layout can render `topcoat::runtime::script()` only when it can be served.
    let state = AppState {
        runtime_script: false,
        ..state
    };
    let _ = &assets;

    // TODO(lab-07): import `topcoat::runtime::RuntimeLayer` and add
    // `.layer(RuntimeLayer::new())` after the cookie layer so
    // tracked server-side signal reads can request a page re-run.
    let builder = topcoat::router::module_router!()
        .layer(CookieLayer::new())
        .with(SessionLifetime(Duration::from_secs(3600)))
        .with(Mailer::new(FileTransport::new("mail")))
        .with(state);

    match assets {
        Some(bundle) => builder.assets(bundle).build(),
        None => builder.build(),
    }
}
// ANCHOR_END: app-router
