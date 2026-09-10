use std::time::Duration;

use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    cookie::RouterBuilderCookieExt,
    font::RouterBuilderFontExt,
    mail::{FileTransport, MailConfig, RouterBuilderMailExt},
    runtime::{RouterBuilderProcedureExt, RouterBuilderShardExt},
    session::{RouterBuilderSessionExt, SessionConfig},
};

use crate::{assets, auth::AppState};

mod _marketing;

// ANCHOR: app-router
pub fn router(state: AppState, assets: Option<AssetBundle>) -> topcoat::router::Router {
    let state = AppState {
        runtime_script: assets.is_some(),
        ..state
    };

    let builder = topcoat::router::module_router!()
        .discover_shards()
        .discover_procedures()
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
        .font(assets::GEIST)
        .app_context(state);

    match assets {
        Some(bundle) => builder.assets(bundle).build(),
        None => builder.build(),
    }
}
// ANCHOR_END: app-router
