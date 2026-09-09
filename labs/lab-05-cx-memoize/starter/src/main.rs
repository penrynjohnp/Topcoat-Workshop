#[tokio::main]
async fn main() {
    // TODO(lab-05): construct AppState once and pass it to app::router.
    topcoat::start(lab05_starter::app::router()).await.unwrap();
}
