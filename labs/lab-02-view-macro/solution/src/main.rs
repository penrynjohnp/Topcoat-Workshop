use topcoat::router::{Router, RouterBuilderDiscoverExt};

#[tokio::main]
async fn main() {
    // The pages live in the library so `tests/` can build the same router.
    // Referencing the library links it in, which is what puts its pages in
    // front of `discover()`.
    assert!(!lab02_solution::berths().is_empty());

    topcoat::start(Router::builder().discover().build())
        .await
        .unwrap();
}
