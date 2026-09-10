use topcoat::asset::AssetBundle;

#[tokio::main]
async fn main() {
    let assets = AssetBundle::load().expect("run `topcoat dev` so the asset bundle exists");
    topcoat::start(lab08_starter::app::router(
        lab08_starter::auth::AppState::new(),
        Some(assets),
    ))
    .await
    .unwrap();
}
