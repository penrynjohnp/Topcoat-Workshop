use serde::Serialize;
use topcoat::{
    Result,
    router::{content::Json, route},
};

#[derive(Serialize)]
pub(crate) struct Health {
    status: &'static str,
}

// ANCHOR: health-route
#[route(GET)]
async fn health() -> Result<Json<Health>> {
    tracing::info!(route = "/api/health", status = "ok", "health check");
    Ok(Json(Health { status: "ok" }))
}
