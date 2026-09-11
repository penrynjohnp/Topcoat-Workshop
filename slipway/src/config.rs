use std::{collections::HashMap, error::Error, fmt, sync::Arc};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use topcoat::cookie::Key;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DatabaseConfig {
    Sqlite {
        url: String,
    },
    ManagedPostgres {
        host: String,
        database: String,
        user: String,
        client_id: String,
    },
}

#[derive(Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database: DatabaseConfig,
    pub cookie_key: Arc<Key>,
    pub production: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ConfigError(String);

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for ConfigError {}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_pairs(std::env::vars())
    }

    pub fn from_pairs(
        pairs: impl IntoIterator<Item = (String, String)>,
    ) -> Result<Self, ConfigError> {
        let vars = pairs.into_iter().collect::<HashMap<_, _>>();
        let production = vars
            .get("SLIPWAY_ENV")
            .is_some_and(|value| value == "production");
        let host = vars
            .get("HOST")
            .cloned()
            .unwrap_or_else(|| "127.0.0.1".to_owned());
        let port = vars
            .get("PORT")
            .map(|value| {
                value
                    .parse::<u16>()
                    .map_err(|_| ConfigError("PORT must be a valid TCP port".to_owned()))
            })
            .transpose()?
            .unwrap_or(3000);

        let database = match vars.get("SLIPWAY_DATABASE_HOST") {
            Some(host) => DatabaseConfig::ManagedPostgres {
                host: host.clone(),
                database: required(&vars, "SLIPWAY_DATABASE_NAME")?,
                user: required(&vars, "SLIPWAY_DATABASE_USER")?,
                client_id: required(&vars, "AZURE_CLIENT_ID")?,
            },
            None if vars.contains_key("SLIPWAY_DATABASE_URL") => DatabaseConfig::Sqlite {
                url: required(&vars, "SLIPWAY_DATABASE_URL")?,
            },
            None if production => {
                return Err(ConfigError(
                    "SLIPWAY_DATABASE_HOST is required in production unless SLIPWAY_DATABASE_URL is explicitly set"
                        .to_owned(),
                ));
            }
            None => DatabaseConfig::Sqlite {
                url: crate::database::DEFAULT_DATABASE_URL.to_owned(),
            },
        };

        let cookie_key = match vars.get("SLIPWAY_COOKIE_KEY") {
            Some(encoded) => Arc::new(decode_cookie_key(encoded)?),
            None if production => {
                return Err(ConfigError(
                    "SLIPWAY_COOKIE_KEY is required in production".to_owned(),
                ));
            }
            None => Arc::new(Key::generate()),
        };

        Ok(Self {
            host,
            port,
            database,
            cookie_key,
            production,
        })
    }
}

fn required(vars: &HashMap<String, String>, name: &str) -> Result<String, ConfigError> {
    vars.get(name)
        .filter(|value| !value.is_empty())
        .cloned()
        .ok_or_else(|| ConfigError(format!("{name} must not be empty")))
}

fn decode_cookie_key(encoded: &str) -> Result<Key, ConfigError> {
    let bytes = STANDARD
        .decode(encoded.trim())
        .map_err(|_| ConfigError("SLIPWAY_COOKIE_KEY must be valid base64".to_owned()))?;
    Key::try_from(bytes.as_slice()).map_err(|_| {
        ConfigError("SLIPWAY_COOKIE_KEY must decode to at least 64 random bytes".to_owned())
    })
}
