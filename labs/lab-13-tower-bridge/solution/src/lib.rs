use axum::{Json, Router as AxumRouter, routing::get};
use serde::Serialize;
use topcoat::{
    Result,
    router::{
        Methods, Router, RouterBuilderDiscoverExt, page,
        tower::{TowerLayer, TowerRoute},
    },
    view::{View, view},
};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

#[derive(Clone, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

pub fn api_router() -> AxumRouter {
    AxumRouter::new().route(
        "/health",
        get(|| async { Json(HealthResponse { status: "ok" }) }),
    )
}

pub fn build_router() -> Router {
    Router::builder()
        .route(TowerRoute::new(
            Methods::Any,
            "/api/v1/{*rest}",
            api_router(),
        ))
        .layer(TowerLayer::new(CompressionLayer::new()).at("/api/v1"))
        .layer(TowerLayer::new(TraceLayer::new_for_http()).at("/api/v1"))
        .discover()
        .build()
}

#[page("/")]
pub async fn home() -> Result<impl View> {
    Ok(view! {
        <!DOCTYPE html>
        <html>
            <head><title>"Slipway"</title></head>
            <body>
                <h1>"Slipway + Axum"</h1>
                <p>"The API lives under /api/v1."</p>
            </body>
        </html>
    })
}
