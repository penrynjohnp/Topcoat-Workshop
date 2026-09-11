use azure_core::credentials::TokenCredential;
use azure_identity::{ManagedIdentityCredential, ManagedIdentityCredentialOptions, UserAssignedId};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};

use crate::config::DatabaseConfig;

const POSTGRES_SCOPE: &str = "https://ossrdbms-aad.database.windows.net/.default";

// ANCHOR: managed-postgres-token
pub async fn connect_managed_postgres(
    config: &DatabaseConfig,
) -> Result<toasty::Db, Box<dyn std::error::Error>> {
    let DatabaseConfig::ManagedPostgres {
        host,
        database,
        user,
        client_id,
    } = config
    else {
        return Err("managed PostgreSQL configuration is required".into());
    };

    let credential = ManagedIdentityCredential::new(Some(ManagedIdentityCredentialOptions {
        user_assigned_id: Some(UserAssignedId::ClientId(client_id.clone())),
        ..Default::default()
    }))?;
    let access_token = credential.get_token(&[POSTGRES_SCOPE], None).await?;
    let url = postgres_connection_url(host, database, user, access_token.token.secret());

    let mut db = toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .connect(&url)
        .await?;
    if !postgres_schema_exists(&mut db).await? {
        db.push_schema().await?;
        crate::database::ensure_seeded(&mut db).await?;
    }
    Ok(db)
}
// ANCHOR_END: managed-postgres-token

async fn postgres_schema_exists(db: &mut toasty::Db) -> Result<bool, Box<dyn std::error::Error>> {
    let rows = toasty::sql::query("SELECT to_regclass('public.berths') IS NOT NULL")
        .exec(db)
        .await?;
    schema_probe_result(&rows)
}

fn schema_probe_result(rows: &[toasty::stmt::Value]) -> Result<bool, Box<dyn std::error::Error>> {
    match rows {
        [toasty::stmt::Value::Record(row)] => match row.first() {
            Some(toasty::stmt::Value::Bool(exists)) => Ok(*exists),
            _ => Err("PostgreSQL schema probe returned a non-boolean value".into()),
        },
        _ => Err("PostgreSQL schema probe returned an unexpected row count".into()),
    }
}

fn postgres_connection_url(host: &str, database: &str, user: &str, token: &str) -> String {
    let encoded_user = utf8_percent_encode(user, NON_ALPHANUMERIC);
    let encoded_token = utf8_percent_encode(token, NON_ALPHANUMERIC);
    format!(
        "postgresql://{encoded_user}:{encoded_token}@{host}:5432/{database}?sslmode=verify-full&sslrootcert=system&application_name=slipway"
    )
}

// ANCHOR: structured-tracing
pub fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .json()
        .with_current_span(false)
        .try_init();
}
// ANCHOR_END: structured-tracing

#[cfg(test)]
mod tests {
    use super::{postgres_connection_url, schema_probe_result};
    use toasty::stmt::Value;

    #[test]
    fn credentials_are_percent_encoded_and_tls_is_verified() {
        let url = postgres_connection_url("db.example", "slipway", "mi name", "a/b+c=");

        assert!(url.contains("mi%20name:a%2Fb%2Bc%3D@db.example"));
        assert!(url.contains("sslmode=verify-full"));
        assert!(url.contains("sslrootcert=system"));
        assert!(!url.contains("a/b+c="));
    }

    #[test]
    fn schema_probe_distinguishes_fresh_and_existing_databases() {
        let fresh = [Value::record_from_vec(vec![Value::Bool(false)])];
        let existing = [Value::record_from_vec(vec![Value::Bool(true)])];

        assert!(!schema_probe_result(&fresh).unwrap());
        assert!(schema_probe_result(&existing).unwrap());
    }
}
