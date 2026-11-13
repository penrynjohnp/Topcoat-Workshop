use serde::Serialize;
use topcoat::{
    Result,
    context::Cx,
    router::{content::Json, route},
};

#[derive(Serialize)]
pub(crate) struct Health {
    status: &'static str,
}

// ANCHOR: health-route
#[route(GET)]
async fn health(cx: &Cx) -> Result<Json<Health>> {
    tracing::info!(route = "/api/health", status = "ok", "health check");
    Ok(Json(Health { status: "ok" }))
}
