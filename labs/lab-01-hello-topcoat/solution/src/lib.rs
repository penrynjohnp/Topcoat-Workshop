//! The Lab 01 solution. The pages live here rather than in `main.rs` so the
//! integration tests in `tests/` can build the same router.

use topcoat::{
    Result,
    router::page,
    view::{View, component, view},
};

/// Linking marker: referencing the library from a binary or a test binary is
/// what puts its inventory-registered pages in front of `discover()`.
pub const NAME: &str = "Topcoat";

#[page("/")]
async fn home() -> Result<impl View> {
    Ok(view! {
        <!DOCTYPE html>
        <html>
            <head>
                <title>"Hello world"</title>
                topcoat::dev::script()
            </head>
            <body>hello(name: NAME)</body>
        </html>
    })
}

#[component]
async fn hello(name: &str) -> Result<impl View> {
    Ok(view! {
        <h1>
            "Hello, "
            (name)
            "!"
        </h1>
    })
}
