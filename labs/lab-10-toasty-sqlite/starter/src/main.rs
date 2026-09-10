use topcoat::asset::AssetBundle;

#[tokio::main]
async fn main() {
    let assets = AssetBundle::load().expect("run `topcoat dev` so the asset bundle exists");
    let url = std::env::var("SLIPWAY_DATABASE_URL")
        .unwrap_or_else(|_| lab10_starter::database::DEFAULT_DATABASE_URL.to_owned());
    let state = lab10_starter::auth::AppState::connect(&url)
        .await
        .expect("connect to the Slipway database");
    topcoat::start(lab10_starter::app::router(state, Some(assets)))
        .await
        .unwrap();
}
