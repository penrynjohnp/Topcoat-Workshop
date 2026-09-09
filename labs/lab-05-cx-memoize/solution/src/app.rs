use crate::shared::AppState;

mod _marketing;

// ANCHOR: app-context-router
pub fn router(state: AppState) -> topcoat::router::Router {
    topcoat::router::module_router!().app_context(state).build()
}
// ANCHOR_END: app-context-router
