use topcoat::asset::AssetBundle;

#[tokio::main]
async fn main() {
    let assets = AssetBundle::load().expect("run `topcoat dev` so the asset bundle exists");
    topcoat::start(lab07_starter::app::router(
        lab07_starter::auth::AppState::new(),
        Some(assets),
    ))
    .await
    .unwrap();
}
