// ANCHOR: app-router
mod _marketing;

pub fn router() -> topcoat::router::Router {
    topcoat::router::module_router!().build()
}
// ANCHOR_END: app-router
