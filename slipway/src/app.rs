use std::time::Duration;

use axum::{Json, Router as AxumRouter, routing::get};
use serde::Serialize;
use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    cookie::CookieLayer,
    mail::{FileTransport, Mailer},
    router::{
        Methods, RouterBuilderDiscoverExt,
        tower::{TowerLayer, TowerRoute},
    },
    runtime::RuntimeLayer,
    session::SessionLifetime,
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
        .discover()
        .layer(CookieLayer::new())
        .with(SessionLifetime(Duration::from_secs(3600)))
        .with(Mailer::new(FileTransport::new("mail")))
        .with(state)
        .route(TowerRoute::new(
            Methods::Any,
            "/api/v1/{*rest}",
            api_router(),
        ))
        .layer(TowerLayer::new(CompressionLayer::new()).at("/api/v1"))
        .layer(TowerLayer::new(TraceLayer::new_for_http()).at("/api/v1"))
        .layer(RuntimeLayer::new());

    match assets {
        Some(bundle) => builder.assets(bundle).build(),
        None => builder.build(),
    }
}
// ANCHOR_END: app-router
