use std::time::Duration;

use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    cookie::RouterBuilderCookieExt,
    mail::{FileTransport, MailConfig, RouterBuilderMailExt},
    router::RouterBuilderDiscoverExt,
    runtime::RouterBuilderRuntimeExt,
    session::{RouterBuilderSessionExt, SessionConfig},
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
        .runtime()
        .discover()
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
        .app_context(state);

    match assets {
        Some(bundle) => builder.assets(bundle).build(),
        None => builder.build(),
    }
}
// ANCHOR_END: app-router
