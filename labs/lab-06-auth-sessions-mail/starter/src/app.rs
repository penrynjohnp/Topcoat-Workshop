// ANCHOR: app-router
mod _marketing;

// TODO(lab-06): accept AppState and register it once with `.app_context(state)`.
// TODO(lab-06): install `.cookies()`, `.sessions(...)`, and `.mail(...)` before build.
pub fn router() -> topcoat::router::Router {
    topcoat::router::module_router!().build()
}
// ANCHOR_END: app-router
