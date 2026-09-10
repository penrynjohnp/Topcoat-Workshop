use slipway::{
    auth::AppState,
    config::{AppConfig, DatabaseConfig},
    production,
};
use topcoat::asset::AssetBundle;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    production::init_tracing();
    let config = AppConfig::from_env()?;
    let database_kind = match &config.database {
        DatabaseConfig::Sqlite { .. } => "sqlite",
        DatabaseConfig::ManagedPostgres { .. } => "postgresql-managed-identity",
    };
    tracing::info!(
        host = %config.host,
        port = config.port,
        database_kind,
        production = config.production,
        "starting Slipway"
    );

    let assets = AssetBundle::load().expect("run `topcoat dev` so the asset bundle exists");
    let db = match &config.database {
        DatabaseConfig::Sqlite { url } => slipway::database::connect(url).await?,
        postgres @ DatabaseConfig::ManagedPostgres { .. } => {
            production::connect_managed_postgres(postgres).await?
        }
    };
    let state = AppState::from_configured_db(db, config.cookie_key, config.production);
    topcoat::start(slipway::app::router(state, Some(assets))).await?;
    Ok(())
}
