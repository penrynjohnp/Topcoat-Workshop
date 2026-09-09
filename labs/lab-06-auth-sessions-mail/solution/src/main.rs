#[tokio::main]
async fn main() {
    topcoat::start(lab06_solution::app::router(
        lab06_solution::auth::AppState::new(),
    ))
    .await
    .unwrap();
}
