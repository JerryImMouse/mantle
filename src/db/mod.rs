use std::path::Path;

use sqlx::{Pool, Postgres, migrate::Migrator};

use crate::config::SharedConfig;
pub type MantleDb = Pool<Postgres>;

pub mod identities;
pub mod identity_cache;
pub mod oauth_tokens;
pub mod users;

#[tracing::instrument(skip(c))]
pub async fn setup(c: &SharedConfig) -> sqlx::Result<MantleDb> {
    tracing::info!("setting up sqlite database");
    let pool = Pool::<Postgres>::connect(&c.database.url).await?;

    tracing::info!("running sqlite migrations...");
    let migrator = Migrator::new(Path::new("migrations")).await?;

    migrator.run(&pool).await?;

    Ok(pool)
}
