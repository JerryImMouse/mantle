use crate::db::{self, MantleDb};
use crate::error::{InternalError, Result};

/// Service which handles bulk operations over
/// `user_metadata`, e.g bulk_delete, bulk_update, etc.
#[derive(Debug, Clone)]
pub struct MetadataService {
    db: MantleDb,
}

impl MetadataService {
    pub fn new(db: MantleDb) -> Self {
        Self { db }
    }

    /// Bulk delete all metadata records with specific key
    pub async fn bulk_delete(&self, key: &str) -> Result<()> {
        db::metadata::delete_by_key(&self.db, key)
            .await
            .map_err(InternalError::from)
    }
}
