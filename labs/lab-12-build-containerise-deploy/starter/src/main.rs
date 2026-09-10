use topcoat::asset::AssetBundle;

#[tokio::main]
async fn main() {
    // TODO(lab-12): Initialize JSON `tracing` before startup work begins.
    let assets = AssetBundle::load().expect("run `topcoat dev` so the asset bundle exists");
    // TODO(lab-12): Replace the local SQLite URL with typed production configuration
    // and a managed-identity PostgreSQL token; never log the completed URL.
    let url = std::env::var("SLIPWAY_DATABASE_URL")
        .unwrap_or_else(|_| lab12_starter::database::DEFAULT_DATABASE_URL.to_owned());
    // TODO(lab-12): Build AppState with the stable Key Vault-backed cookie key.
    let state = lab12_starter::auth::AppState::connect(&url)
        .await
        .expect("connect to the Slipway database");
    topcoat::start(lab12_starter::app::router(state, Some(assets)))
        .await
        .unwrap();
}
