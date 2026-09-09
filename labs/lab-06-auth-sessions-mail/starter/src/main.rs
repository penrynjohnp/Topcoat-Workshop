#[tokio::main]
async fn main() {
    // TODO(lab-06): construct the auth/session AppState once and pass it to app::router.
    topcoat::start(lab06_starter::app::router()).await.unwrap();
}
