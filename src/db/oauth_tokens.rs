use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::db::MantleDb;

#[derive(Debug, sqlx::FromRow)]
pub struct OAuthTokens {
    pub identity_id: Uuid,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn create(
    d: &MantleDb,
    identity_id: Uuid,
    access_token: String,
    refresh_token: String,
    expires_at: DateTime<Utc>,
) -> sqlx::Result<OAuthTokens> {
    Ok(sqlx::query_as(
        r#"
        INSERT INTO oauth_tokens (identity_id, access_token, refresh_token, expires_at)
        VALUES ($1, $2, $3, $4) RETURNING *;
    "#,
    )
    .bind(identity_id)
    .bind(access_token)
    .bind(refresh_token)
    .bind(expires_at)
    .fetch_one(d)
    .await?)
}

pub async fn find_by_id(d: &MantleDb, identity_id: Uuid) -> sqlx::Result<Option<OAuthTokens>> {
    Ok(
        sqlx::query_as("SELECT * FROM oauth_tokens WHERE identity_id = $1")
            .bind(identity_id)
            .fetch_optional(d)
            .await?,
    )
}

pub async fn update(
    d: &MantleDb, 
    identity_id: Uuid, 
    access_token: &str,
    refresh_token: &str,
    expires_at: DateTime<Utc>,
) -> sqlx::Result<()> {
    sqlx::query("UPDATE oauth_tokens SET access_token = $1, refresh_token = $2, expires_at = $3, updated_at = $4 WHERE identity_id = $5")
        .bind(access_token)
        .bind(refresh_token)
        .bind(expires_at)
        .bind(Utc::now())
        .bind(identity_id)
        .execute(d)
        .await?;
    Ok(())
}
