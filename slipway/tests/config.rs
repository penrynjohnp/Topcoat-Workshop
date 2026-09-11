use base64::{Engine as _, engine::general_purpose::STANDARD};
use slipway_capstone::config::{AppConfig, DatabaseConfig};

fn pairs(values: &[(&str, &str)]) -> Vec<(String, String)> {
    values
        .iter()
        .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
        .collect()
}

#[test]
fn local_configuration_defaults_to_sqlite() {
    let config = AppConfig::from_pairs(pairs(&[])).unwrap();

    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 3000);
    assert_eq!(
        config.database,
        DatabaseConfig::Sqlite {
            url: "sqlite:./slipway.db".to_owned()
        }
    );
    assert!(!config.production);
}

#[test]
fn production_requires_managed_postgres_and_a_stable_cookie_key() {
    let error = AppConfig::from_pairs(pairs(&[("SLIPWAY_ENV", "production")]))
        .err()
        .unwrap();

    assert_eq!(
        error.to_string(),
        "SLIPWAY_DATABASE_HOST is required in production unless SLIPWAY_DATABASE_URL is explicitly set"
    );
}

#[test]
fn production_can_use_an_explicit_sqlite_volume_off_azure() {
    let cookie_key = STANDARD.encode([5_u8; 64]);
    let config = AppConfig::from_pairs(pairs(&[
        ("SLIPWAY_ENV", "production"),
        ("SLIPWAY_DATABASE_URL", "sqlite:/data/slipway.db"),
        ("SLIPWAY_COOKIE_KEY", &cookie_key),
    ]))
    .unwrap();

    assert_eq!(
        config.database,
        DatabaseConfig::Sqlite {
            url: "sqlite:/data/slipway.db".to_owned()
        }
    );
    assert!(config.production);
}

#[test]
fn production_configuration_uses_identity_inputs() {
    let cookie_key = STANDARD.encode([7_u8; 64]);
    let config = AppConfig::from_pairs(pairs(&[
        ("SLIPWAY_ENV", "production"),
        ("HOST", "0.0.0.0"),
        ("PORT", "8080"),
        (
            "SLIPWAY_DATABASE_HOST",
            "slipway.postgres.database.azure.com",
        ),
        ("SLIPWAY_DATABASE_NAME", "slipway"),
        ("SLIPWAY_DATABASE_USER", "mi-slipway"),
        ("AZURE_CLIENT_ID", "00000000-0000-0000-0000-000000000000"),
        ("SLIPWAY_COOKIE_KEY", &cookie_key),
    ]))
    .unwrap();

    assert!(config.production);
    assert_eq!(config.port, 8080);
    assert_eq!(
        config.database,
        DatabaseConfig::ManagedPostgres {
            host: "slipway.postgres.database.azure.com".to_owned(),
            database: "slipway".to_owned(),
            user: "mi-slipway".to_owned(),
            client_id: "00000000-0000-0000-0000-000000000000".to_owned(),
        }
    );
}

#[tokio::test]
async fn production_state_marks_session_cookies_secure() {
    let cookie_key = STANDARD.encode([9_u8; 64]);
    let config = AppConfig::from_pairs(pairs(&[
        ("SLIPWAY_ENV", "production"),
        ("SLIPWAY_DATABASE_HOST", "db.example"),
        ("SLIPWAY_DATABASE_NAME", "slipway"),
        ("SLIPWAY_DATABASE_USER", "mi-slipway"),
        ("AZURE_CLIENT_ID", "00000000-0000-0000-0000-000000000000"),
        ("SLIPWAY_COOKIE_KEY", &cookie_key),
    ]))
    .unwrap();
    let db = slipway_capstone::database::test_database().await.unwrap();
    let state = slipway_capstone::auth::AppState::from_configured_db(db, config.cookie_key, true);

    assert!(state.secure_cookies);
}

#[test]
fn cookie_key_must_decode_to_at_least_64_bytes() {
    let error = AppConfig::from_pairs(pairs(&[(
        "SLIPWAY_COOKIE_KEY",
        &STANDARD.encode([1_u8; 32]),
    )]))
    .err()
    .unwrap();

    assert_eq!(
        error.to_string(),
        "SLIPWAY_COOKIE_KEY must decode to at least 64 random bytes"
    );
}
