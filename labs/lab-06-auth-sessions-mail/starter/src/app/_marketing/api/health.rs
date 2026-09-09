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
    Ok(Json(Health { status: "ok" }))
}
