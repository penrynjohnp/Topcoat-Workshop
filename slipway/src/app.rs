use std::time::Duration;

use axum::{Json, Router as AxumRouter, routing::get};
use serde::Serialize;
use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    cookie::RouterBuilderCookieExt,
    font::RouterBuilderFontExt,
    mail::{FileTransport, MailConfig, RouterBuilderMailExt},
    router::{
        Methods,
        tower::{TowerLayer, TowerRoute},
    },
    runtime::{RouterBuilderProcedureExt, RouterBuilderShardExt},
    session::{RouterBuilderSessionExt, SessionConfig},
};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

use crate::{assets, auth::AppState};

mod _marketing;

#[derive(Clone, Serialize)]
pub struct ApiHealthResponse {
    pub status: &'static str,
}

pub fn api_router() -> AxumRouter {
    AxumRouter::new().route(
        "/api/v1/health",
        get(|| async { Json(ApiHealthResponse { status: "ok" }) }),
    )
}

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
        .app_context(state)
        .route(TowerRoute::new(
            Methods::Any,
            "/api/v1/{*rest}",
            api_router(),
        ))
        .layer(TowerLayer::new(CompressionLayer::new()).at("/api/v1"))
        .layer(TowerLayer::new(TraceLayer::new_for_http()).at("/api/v1"));

    match assets {
        Some(bundle) => builder.assets(bundle).build(),
        None => builder.build(),
    }
}
// ANCHOR_END: app-router
