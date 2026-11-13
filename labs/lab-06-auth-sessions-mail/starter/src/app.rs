// ANCHOR: app-router
mod _marketing;

// TODO(lab-06): accept AppState and register it once with `.with(state)`.
// TODO(lab-06): add `.layer(CookieLayer::new())`, `.with(SessionLifetime(...))`, and
// `.with(Mailer::new(...))` before build.
pub fn router() -> topcoat::router::Router {
    topcoat::router::module_router!().build()
}
// ANCHOR_END: app-router
