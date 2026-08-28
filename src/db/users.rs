use super::MantleDb;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(sqlx::FromRow, Debug)]
pub struct MantleUser {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[tracing::instrument(skip(d))]
pub async fn create(d: &MantleDb) -> sqlx::Result<MantleUser> {
    sqlx::query_as::<_, MantleUser>("INSERT INTO mantle_users DEFAULT VALUES RETURNING *;")
        .fetch_one(d)
        .await
}
