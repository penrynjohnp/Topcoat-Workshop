#[tokio::main]
async fn main() {
    topcoat::start(lab04_solution::app::router()).await.unwrap();
}
