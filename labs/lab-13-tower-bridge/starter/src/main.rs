use axum::{Json, Router as AxumRouter, routing::get};
use serde::Serialize;
use topcoat::{
    Result,
    router::{
        Methods, Router, page,
        tower::{TowerLayer, TowerRoute},
    },
    view::{View, view},
};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[tokio::main]
async fn main() {
    topcoat::start(build_router()).await.unwrap();
}

fn build_api_router() -> AxumRouter {
    // TODO(lab-13): replace this placeholder with a small /health API and keep it mounted in the Topcoat router.
    AxumRouter::new().route(
        "/health",
        get(|| async { Json(HealthResponse { status: "ok" }) }),
    )
}

fn build_router() -> Router {
    Router::builder()
        .route(TowerRoute::new(
            Methods::Any,
            "/api/v1/{*rest}",
            build_api_router(),
        ))
        .layer(TowerLayer::new(CompressionLayer::new()).at("/api/v1"))
        .layer(TowerLayer::new(TraceLayer::new_for_http()).at("/api/v1"))
        .build()
}

#[page("/")]
async fn home() -> Result<impl View> {
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
