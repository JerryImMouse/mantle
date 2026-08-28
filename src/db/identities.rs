use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::MantleDb;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "identity_provider", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum IdentityProvider {
    Discord,
    SS14,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Identity {
    pub id: Uuid,
    pub mantle_user_id: Uuid,

    pub provider: IdentityProvider,
    pub provider_user_id: String,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct CreateIdentityReq<'a> {
    pub mantle_user_id: Uuid,
    pub provider: IdentityProvider,
    pub provider_user_id: &'a str,
}

#[tracing::instrument(skip(d))]
pub async fn create(d: &MantleDb, req: CreateIdentityReq<'_>) -> sqlx::Result<Identity> {
    Ok(sqlx::query_as(
        r#"
INSERT INTO identities (mantle_user_id, provider, provider_user_id)
    VALUES ($1, $2, $3) RETURNING *;
        "#,
    )
    .bind(req.mantle_user_id)
    .bind(req.provider)
    .bind(req.provider_user_id)
    .fetch_one(d)
    .await?)
}

pub async fn find_by_provider_id(
    d: &MantleDb,
    provider: IdentityProvider,
    provider_user_id: &str,
) -> sqlx::Result<Option<Identity>> {
    Ok(sqlx::query_as(
        r#"
            SELECT * FROM identities
                WHERE provider = $1
                AND provider_user_id = $2;
        "#,
    )
    .bind(provider)
    .bind(provider_user_id)
    .fetch_optional(d)
    .await?)
}

pub async fn all_for_user(d: &MantleDb, mantle_user_id: Uuid) -> sqlx::Result<Vec<Identity>> {
    Ok(
        sqlx::query_as("SELECT * FROM identities WHERE mantle_user_id = $1")
            .bind(mantle_user_id)
            .fetch_all(d)
            .await?,
    )
}

pub async fn find_linked_identity(
    d: &MantleDb,
    provider: IdentityProvider,
    provider_user_id: &str,
    target_provider: IdentityProvider,
) -> sqlx::Result<Option<Identity>> {
    sqlx::query_as(
        r#"
        SELECT target.*
        FROM identities source
        JOIN identities target
          ON target.mantle_user_id = source.mantle_user_id
         AND target.provider = $3
        WHERE source.provider = $1
          AND source.provider_user_id = $2
        "#,
    )
    .bind(provider)
    .bind(provider_user_id)
    .bind(target_provider)
    .fetch_optional(d)
    .await
}
