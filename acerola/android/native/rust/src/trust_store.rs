use std::{collections::HashSet, path::Path, sync::Arc};

use acerola_p2p::api::{guard::TrustedPeerStore, protocol::EventEmitter};
use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::callbacks::SecureBlobStore;

const PEER_TRUSTED_FIRST_TIME_EVENT: &str = "peer:trusted_first_time";
const TRUSTED_KEY: &str = "trusted";
const BLOCKED_KEY: &str = "blocked";

/// `TrustedPeerStore` implementation that persists via [`SecureBlobStore`] (Kotlin,
/// `EncryptedSharedPreferences` protected by the Android Keystore) instead of text files
/// (one id per line) — keeps TOFU trust permanent across process restarts and across
/// mode switches (local/relay), now protected at rest.
///
/// When registering a peer as trusted for the first time, emits the
/// `peer:trusted_first_time` event through the same `P2PCallback` channel, since the lib
/// (`TofuGuard`) does not expose its own hook for that moment.
pub(crate) struct SecureTrustedStore {
    store: Arc<dyn SecureBlobStore>,
    trusted: RwLock<HashSet<String>>,
    blocked: HashSet<String>,
    emit: EventEmitter,
}

impl SecureTrustedStore {
    /// `legacy_dir`, if given and still holding `trusted.txt`/`blocked.txt` (old format,
    /// plain text with one id per line), has its content migrated to the secure storage
    /// once — without this, the existing TOFU trust list would be lost on the first update and
    /// every already paired peer would ask for confirmation again.
    pub(crate) fn open(
        store: Arc<dyn SecureBlobStore>,
        emit: EventEmitter,
        legacy_dir: Option<&Path>,
    ) -> Self {
        if let Some(legacy_dir) = legacy_dir {
            Self::migrate_legacy_files(&store, legacy_dir);
        }

        let trusted = Self::read_set(&store, TRUSTED_KEY);
        let blocked = Self::read_set(&store, BLOCKED_KEY);

        Self {
            store,
            trusted: RwLock::new(trusted),
            blocked,
            emit,
        }
    }

    fn migrate_legacy_files(store: &Arc<dyn SecureBlobStore>, legacy_dir: &Path) {
        Self::migrate_legacy_file(store, &legacy_dir.join("trusted.txt"), TRUSTED_KEY);
        Self::migrate_legacy_file(store, &legacy_dir.join("blocked.txt"), BLOCKED_KEY);
    }

    /// Only migrates if the secure storage confirms (`Ok(None)`) that the key is genuinely
    /// empty — on a real backend failure (`Err`), it neither migrates nor deletes the legacy
    /// file, so as not to risk losing the only good data that still exists.
    fn migrate_legacy_file(store: &Arc<dyn SecureBlobStore>, path: &Path, key: &str) {
        match store.load_blob(key.to_string()) {
            Ok(None) => {
                if let Ok(Some(bytes)) = crate::fsutil::read_optional(path) {
                    let set: HashSet<String> = String::from_utf8_lossy(&bytes)
                        .lines()
                        .map(str::trim)
                        .filter(|line| !line.is_empty())
                        .map(str::to_string)
                        .collect();

                    match serde_json::to_vec(&set) {
                        Ok(json) => {
                            if let Err(err) = store.save_blob(key.to_string(), json) {
                                tracing::error!(layer = "storage", key, error = %err, "failed to migrate legacy file");
                                return;
                            }
                        }
                        Err(err) => {
                            tracing::error!(layer = "storage", key, error = ?err, "failed to encode migrated set");
                            return;
                        }
                    }
                }
                std::fs::remove_file(path).ok();
            }
            Ok(Some(_)) => {
                // Already migrated in a previous run — just clean up the old file.
                std::fs::remove_file(path).ok();
            }
            Err(err) => {
                tracing::error!(layer = "storage", key, error = %err, "secure store unavailable, skipping legacy migration");
            }
        }
    }

    fn read_set(store: &Arc<dyn SecureBlobStore>, key: &str) -> HashSet<String> {
        match store.load_blob(key.to_string()) {
            Ok(Some(bytes)) => serde_json::from_slice(&bytes).unwrap_or_default(),
            Ok(None) => HashSet::new(),
            Err(err) => {
                tracing::error!(layer = "storage", key, error = %err, "failed to load set, starting empty");
                HashSet::new()
            }
        }
    }

    /// Removes a peer from the trusted list — used when the user unpairs a
    /// device from the UI. Does not touch `blocked`, only undoes the granted trust: if that
    /// peer tries to connect again later, it goes through the same TOFU flow as a new peer
    /// (see `TofuGuard::into_validator` in acerola-p2p — it recognizes and re-trusts
    /// automatically, exactly as it would with a never-seen device).
    pub(crate) async fn remove(&self, id: &str) {
        let mut trusted = self.trusted.write().await;
        trusted.remove(id);
        self.write_set(TRUSTED_KEY, &trusted);
    }

    fn write_set(&self, key: &str, set: &HashSet<String>) {
        match serde_json::to_vec(set) {
            Ok(bytes) => {
                if let Err(err) = self.store.save_blob(key.to_string(), bytes) {
                    tracing::error!(layer = "storage", key, error = %err, "failed to persist blob");
                }
            }
            Err(err) => {
                tracing::error!(layer = "storage", key, error = ?err, "failed to encode set");
            }
        }
    }
}

#[async_trait]
impl TrustedPeerStore for SecureTrustedStore {
    async fn insert(&self, id: &str) {
        let mut trusted = self.trusted.write().await;
        if trusted.contains(id) {
            return;
        }

        trusted.insert(id.to_string());
        self.write_set(TRUSTED_KEY, &trusted);
        drop(trusted);

        (self.emit)(PEER_TRUSTED_FIRST_TIME_EVENT, id.to_string());
    }

    async fn contains(&self, id: &str) -> bool {
        self.trusted.read().await.contains(id)
    }

    async fn is_blocked(&self, id: &str) -> bool {
        self.blocked.contains(id)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Mutex as StdMutex};

    use super::*;
    use crate::callbacks::{SecureBlobStore, SecureBlobStoreError};

    struct FakeBlobStore {
        data: StdMutex<HashMap<String, Vec<u8>>>,
    }

    impl FakeBlobStore {
        fn new() -> Self {
            Self {
                data: StdMutex::new(HashMap::new()),
            }
        }
    }

    impl SecureBlobStore for FakeBlobStore {
        fn save_blob(&self, key: String, value: Vec<u8>) -> Result<(), SecureBlobStoreError> {
            self.data.lock().unwrap().insert(key, value);
            Ok(())
        }

        fn load_blob(&self, key: String) -> Result<Option<Vec<u8>>, SecureBlobStoreError> {
            Ok(self.data.lock().unwrap().get(&key).cloned())
        }

        fn clear_blob(&self, key: String) -> Result<(), SecureBlobStoreError> {
            self.data.lock().unwrap().remove(&key);
            Ok(())
        }
    }

    fn noop_emitter() -> EventEmitter {
        Arc::new(|_event, _data| {})
    }

    #[tokio::test]
    async fn remove_forgets_a_trusted_peer() {
        let store = SecureTrustedStore::open(Arc::new(FakeBlobStore::new()), noop_emitter(), None);
        store.insert("peer-a").await;
        assert!(store.contains("peer-a").await);

        store.remove("peer-a").await;

        assert!(!store.contains("peer-a").await);
    }

    #[tokio::test]
    async fn remove_only_affects_the_given_peer() {
        let store = SecureTrustedStore::open(Arc::new(FakeBlobStore::new()), noop_emitter(), None);
        store.insert("peer-a").await;
        store.insert("peer-b").await;

        store.remove("peer-a").await;

        assert!(!store.contains("peer-a").await);
        assert!(store.contains("peer-b").await);
    }

    #[tokio::test]
    async fn remove_persists_across_reopen() {
        let blob_store: Arc<dyn SecureBlobStore> = Arc::new(FakeBlobStore::new());

        let store = SecureTrustedStore::open(Arc::clone(&blob_store), noop_emitter(), None);
        store.insert("peer-a").await;
        store.insert("peer-b").await;
        store.remove("peer-a").await;

        let reopened = SecureTrustedStore::open(blob_store, noop_emitter(), None);
        assert!(!reopened.contains("peer-a").await);
        assert!(reopened.contains("peer-b").await);
    }

    #[tokio::test]
    async fn removing_an_unknown_peer_is_a_no_op() {
        let store = SecureTrustedStore::open(Arc::new(FakeBlobStore::new()), noop_emitter(), None);

        store.remove("never-existed").await;

        assert!(!store.contains("never-existed").await);
    }
}
