#[tokio::main]
async fn main() {
    topcoat::start(lab05_solution::app::router(
        lab05_solution::shared::AppState::new(),
    ))
    .await
    .unwrap();
}
