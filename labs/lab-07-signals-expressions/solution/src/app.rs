use std::time::Duration;

use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    cookie::CookieLayer,
    mail::{FileTransport, Mailer},
    runtime::RuntimeLayer,
    session::SessionLifetime,
};

use crate::auth::AppState;

mod _marketing;

// ANCHOR: app-router
pub fn router(state: AppState, assets: Option<AssetBundle>) -> topcoat::router::Router {
    let state = AppState {
        runtime_script: assets.is_some(),
        ..state
    };

    let builder = topcoat::router::module_router!()
        .layer(CookieLayer::new())
        .with(SessionLifetime(Duration::from_secs(3600)))
        .with(Mailer::new(FileTransport::new("mail")))
        .with(state)
        .layer(RuntimeLayer::new());

    match assets {
        Some(bundle) => builder.assets(bundle).build(),
        None => builder.build(),
    }
}
// ANCHOR_END: app-router
