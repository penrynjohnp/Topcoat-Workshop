use std::time::Duration;

use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    cookie::RouterBuilderCookieExt,
    mail::{FileTransport, MailConfig, RouterBuilderMailExt},
    session::{RouterBuilderSessionExt, SessionConfig},
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

    // TODO(lab-07): import `RouterBuilderRuntimeExt` and call `.runtime()` so
    // tracked server-side signal reads can request a page re-run.
    let builder = topcoat::router::module_router!()
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
