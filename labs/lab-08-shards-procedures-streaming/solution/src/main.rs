use topcoat::asset::AssetBundle;

#[tokio::main]
async fn main() {
    let assets = AssetBundle::load().expect("run `topcoat dev` so the asset bundle exists");
    topcoat::start(lab08_solution::app::router(
        lab08_solution::auth::AppState::new(),
        Some(assets),
    ))
    .await
    .unwrap();
}
