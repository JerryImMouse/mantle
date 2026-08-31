use std::{
    collections::{HashMap, hash_map::Entry},
    sync::{Arc, Mutex},
};

use tokio::sync::Notify;
use uuid::Uuid;

use crate::db::identities::Identity;

// TODO: it should be tested properly, but currently im too lazy to do this
#[derive(Debug, Default, Clone)]
pub struct RefreshLock {
    refreshes: Arc<Mutex<HashMap<Uuid, Arc<Notify>>>>,
}

pub struct RefreshGuard {
    identity_id: Uuid,
    refreshes: Arc<Mutex<HashMap<Uuid, Arc<Notify>>>>,
}

impl Drop for RefreshGuard {
    fn drop(&mut self) {
        let identity_id = self.identity_id;
        let refreshes = self.refreshes.clone();

        let mut refreshes = refreshes.lock().unwrap();
        if let Some(current) = refreshes.remove(&identity_id) {
            current.notify_waiters();
        }
    }
}

impl RefreshLock {
    pub fn try_lock(&self, identity: &Identity) -> Result<RefreshGuard, Arc<Notify>> {
        let mut refreshes = self.refreshes.lock().unwrap();

        match refreshes.entry(identity.id) {
            Entry::Occupied(entry) => Err(entry.get().clone()),
            Entry::Vacant(entry) => {
                entry.insert(Arc::new(Notify::new()));
                Ok(RefreshGuard {
                    identity_id: identity.id,
                    refreshes: self.refreshes.clone(),
                })
            }
        }
    }
}
