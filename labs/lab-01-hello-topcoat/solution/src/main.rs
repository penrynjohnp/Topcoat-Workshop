use topcoat::router::{Router, RouterBuilderDiscoverExt};

#[tokio::main]
async fn main() {
    assert!(!lab01_solution::NAME.is_empty());

    topcoat::start(Router::builder().discover().build())
        .await
        .unwrap();
}
