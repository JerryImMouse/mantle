use chrono::{DateTime, Utc};
use serde::Serialize;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::db::MantleDb;
use crate::error::Result;

#[derive(Debug, sqlx::FromRow)]
pub struct IdentityCache {
    pub identity_id: Uuid,
    pub key: String,
    pub value: serde_json::Value,
    pub expires_at: DateTime<Utc>,
}

#[tracing::instrument(skip(d, value))]
pub async fn set<T: Serialize + std::fmt::Debug>(
    d: &MantleDb,
    identity_id: Uuid,
    key: &str,
    value: &T,
    expires_at: DateTime<Utc>,
) -> Result<IdentityCache> {
    Ok(sqlx::query_as(
        r#"
        INSERT INTO identity_cache (identity_id, key, value, expires_at)
        VALUES ($1, $2, $3, $4) 
        ON CONFLICT (identity_id, key) DO UPDATE
            SET value = excluded.value,
                expires_at = excluded.expires_at
        RETURNING *;
    "#,
    )
    .bind(identity_id)
    .bind(key)
    .bind(serde_json::to_value(value)?)
    .bind(expires_at)
    .fetch_one(d)
    .await?)
}

#[tracing::instrument(skip(d))]
pub async fn get<T: DeserializeOwned>(
    d: &MantleDb,
    identity_id: Uuid,
    key: &str,
) -> Result<Option<T>> {
    // select only non-expired records
    sqlx::query_as::<_, IdentityCache>(
        r#"
        SELECT * FROM identity_cache
            WHERE identity_id = $1
            AND key = $2
            AND expires_at >= NOW()
    "#,
    )
    .bind(identity_id)
    .bind(key)
    .fetch_optional(d)
    .await?
    .map(|data| serde_json::from_value(data.value))
    .transpose()
    .map_err(Into::into)
}
