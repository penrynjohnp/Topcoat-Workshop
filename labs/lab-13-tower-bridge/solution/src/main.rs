#[tokio::main]
async fn main() {
    topcoat::start(lab13_solution::build_router())
        .await
        .unwrap();
}
