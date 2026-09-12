use std::time::Duration;

use axum::{Json, Router as AxumRouter, routing::get};
use serde::Serialize;
use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    cookie::RouterBuilderCookieExt,
    mail::{FileTransport, MailConfig, RouterBuilderMailExt},
    router::{
        Methods, RouterBuilderDiscoverExt,
        tower::{TowerLayer, TowerRoute},
    },
    runtime::RouterBuilderRuntimeExt,
    session::{RouterBuilderSessionExt, SessionConfig},
};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

use crate::auth::AppState;

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
