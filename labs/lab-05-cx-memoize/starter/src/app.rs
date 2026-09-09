// ANCHOR: app-router
mod _marketing;

// TODO(lab-05): accept AppState and register it once with `.app_context(state)`.
pub fn router() -> topcoat::router::Router {
    topcoat::router::module_router!().build()
}
// ANCHOR_END: app-router
