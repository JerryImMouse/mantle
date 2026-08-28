use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::db::MantleDb;

#[derive(Debug, sqlx::FromRow)]
pub struct UserMetadata {
    pub mantle_user_id: Uuid,
    pub key: String,
    pub value: serde_json::Value,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn set(
    d: &MantleDb,
    mantle_user_id: Uuid,
    key: &str,
    value: &serde_json::Value,
) -> sqlx::Result<UserMetadata> {
    sqlx::query_as(
        r#"
        INSERT INTO user_metadata (mantle_user_id, key, value)
        VALUES ($1, $2, $3) 
        ON CONFLICT (mantle_user_id, key) DO UPDATE
            SET value = excluded.value,
                updated_at = NOW()
        RETURNING *;
        "#,
    )
    .bind(mantle_user_id)
    .bind(key)
    .bind(value)
    .fetch_one(d)
    .await
}

pub async fn get(
    d: &MantleDb,
    mantle_user_id: Uuid,
    key: &str,
) -> sqlx::Result<Option<UserMetadata>> {
    sqlx::query_as(
        r#"
        SELECT * FROM user_metadata 
        WHERE mantle_user_id = $1 
            AND key = $2
        "#,
    )
    .bind(mantle_user_id)
    .bind(key)
    .fetch_optional(d)
    .await
}

pub async fn delete(
    d: &MantleDb,
    mantle_user_id: Uuid,
    key: &str,
) -> sqlx::Result<Option<UserMetadata>> {
    sqlx::query_as("DELETE FROM user_metadata WHERE mantle_user_id = $1 AND key = $2 RETURNING *")
        .bind(mantle_user_id)
        .bind(key)
        .fetch_optional(d)
        .await
}

pub async fn delete_by_key(d: &MantleDb, key: &str) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM user_metadata WHERE key = $1")
        .bind(key)
        .execute(d)
        .await?;
    Ok(())
}

pub async fn update_by_key(d: &MantleDb, key: &str, value: &serde_json::Value) -> sqlx::Result<()> {
    sqlx::query("UPDATE user_metadata SET value = $1 WHERE key = $2")
        .bind(value)
        .bind(key)
        .execute(d)
        .await?;
    Ok(())
}
