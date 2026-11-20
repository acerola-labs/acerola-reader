use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use acerola_p2p::api::{
    error::P2pError,
    identity::DeviceInfo,
    peer::{PeerAddr, PeerIdentity},
    storage::P2PStorage,
};
use async_trait::async_trait;
use tokio::sync::RwLock;

use super::{decrypt, encrypt};

/// Collapses entries with the same `id.id`, keeping the last one (same "most recent
/// wins" rule as the upsert in `save_peer`). Migration for `peers.enc` files written before the
/// upsert fix (which compared the whole `PeerId`, so `device_id: None` vs
/// `device_id: Some(..)` for the same peer became two entries) — without this, anyone who
/// already has a persisted duplicate would keep seeing `pairedPeers` break `{#each ... (peer.peerId)}`
/// in the frontend even after updating.
fn dedupe_by_id(peers: Vec<PeerAddr>) -> Vec<PeerAddr> {
    let mut deduped: Vec<PeerAddr> = Vec::with_capacity(peers.len());
    for peer in peers {
        match deduped.iter_mut().find(|cached| cached.id.id == peer.id.id) {
            Some(cached) => *cached = peer,
            None => deduped.push(peer),
        }
    }
    deduped
}

/// `P2PStorage` implementation (node identity + paired peers cache) with
/// encryption at rest — see `infra::security` for the scheme (master key in the
/// OS keyring, AES-256-GCM for the rest). Layout inside `base_dir`:
/// - `identity.enc` — the node's identity seed, encrypted.
/// - `peers.enc`    — list of known `PeerAddr`s (JSON before encryption).
pub struct SecureP2pStorage {
    base_dir: PathBuf,
    master_key: [u8; 32],
    peers_cache: RwLock<Vec<PeerAddr>>,
    device_info_cache: RwLock<Vec<(PeerIdentity, DeviceInfo)>>,
}

impl SecureP2pStorage {
    pub fn open(base_dir: impl Into<PathBuf>, master_key: [u8; 32]) -> std::io::Result<Self> {
        let base_dir = base_dir.into();
        std::fs::create_dir_all(&base_dir)?;

        let peers_cache = Self::read_peers(&base_dir, &master_key)?;
        let device_info_cache = Self::read_device_info(&base_dir, &master_key)?;

        Ok(Self {
            base_dir,
            master_key,
            peers_cache: RwLock::new(peers_cache),
            device_info_cache: RwLock::new(device_info_cache),
        })
    }

    fn identity_path(base_dir: &Path) -> PathBuf {
        base_dir.join("identity.enc")
    }

    fn peers_path(base_dir: &Path) -> PathBuf {
        base_dir.join("peers.enc")
    }

    fn device_info_path(base_dir: &Path) -> PathBuf {
        base_dir.join("device_info.enc")
    }

    fn iroh_services_ticket_path(base_dir: &Path) -> PathBuf {
        base_dir.join("iroh_services_ticket.enc")
    }

    /// Reads and decrypts `peers.enc`. Unlike the previous version, a real decryption
    /// failure (wrong/corrupted master key) becomes `Err` from here — it must not
    /// silently become an "empty list", because `save_peer` rewrites the whole file
    /// with whatever is in memory: a mistakenly empty cache wipes every previously paired
    /// peer as soon as the next sync saves something.
    fn read_peers(base_dir: &Path, master_key: &[u8; 32]) -> std::io::Result<Vec<PeerAddr>> {
        let peers = match std::fs::read(Self::peers_path(base_dir)) {
            Ok(encrypted) => {
                let json = decrypt(master_key, &encrypted).map_err(|err| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("failed to decrypt peers.enc (wrong master key or corrupted file): {err}"),
                    )
                })?;
                serde_json::from_slice(&json).map_err(|err| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("failed to parse peers.enc contents: {err}"),
                    )
                })?
            },
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(err) => return Err(err),
        };

        Ok(dedupe_by_id(peers))
    }

    /// Reads and decrypts `device_info.enc` — same logic as `read_peers`, but for the device
    /// name exchanged in the handshake. Before this persistence, that data only existed in
    /// memory (`NetworkState`) and disappeared from the UI (falling back to the raw ID) until the
    /// peer reconnected and redid the handshake in this session.
    fn read_device_info(
        base_dir: &Path, master_key: &[u8; 32],
    ) -> std::io::Result<Vec<(PeerIdentity, DeviceInfo)>> {
        match std::fs::read(Self::device_info_path(base_dir)) {
            Ok(encrypted) => {
                let json = decrypt(master_key, &encrypted).map_err(|err| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "failed to decrypt device_info.enc (wrong master key or corrupted file): {err}"
                        ),
                    )
                })?;
                serde_json::from_slice(&json).map_err(|err| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("failed to parse device_info.enc contents: {err}"),
                    )
                })
            },
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(err) => Err(err),
        }
    }
}

#[async_trait]
impl P2PStorage for SecureP2pStorage {
    async fn save_identity(&self, secret: &[u8]) -> Result<(), P2pError> {
        let encrypted = encrypt(&self.master_key, secret);
        std::fs::write(Self::identity_path(&self.base_dir), encrypted)
            .map_err(|err| P2pError::StreamFailed(format!("failed to save identity: {err}")))
    }

    async fn load_identity(&self) -> Result<Option<Vec<u8>>, P2pError> {
        let encrypted = match std::fs::read(Self::identity_path(&self.base_dir)) {
            Ok(bytes) => bytes,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(err) => {
                return Err(P2pError::StreamFailed(format!("failed to load identity: {err}")))
            },
        };

        decrypt(&self.master_key, &encrypted)
            .map(Some)
            .map_err(|err| P2pError::StreamFailed(format!("failed to decrypt identity: {err}")))
    }

    async fn save_peer(&self, peer: &PeerAddr) -> Result<(), P2pError> {
        let mut peers = self.peers_cache.write().await;
        // Compare only `id.id` (the identity string), not the whole `PeerId` — `PeerId` derives
        // `PartialEq` including `device_id`, which the caller does not always know (e.g.
        // frontend commands build `PeerIdentity { id, device_id: None }` because they only have the
        // string). Comparing the whole struct makes a reconnection without `device_id` duplicate the
        // already-paired peer (which has `device_id: Some(..)` from the original handshake) instead of
        // updating it — and the paired list becomes the `pairedPeers` the UI uses as the `{#each}` key,
        // so the duplicate breaks rendering (`each_key_duplicate`) instead of just being ugly data.
        match peers.iter_mut().find(|cached| cached.id.id == peer.id.id) {
            Some(cached) => *cached = peer.clone(),
            None => peers.push(peer.clone()),
        }

        let json = serde_json::to_vec(&*peers)
            .map_err(|err| P2pError::StreamFailed(format!("failed to encode peer cache: {err}")))?;
        let encrypted = encrypt(&self.master_key, &json);
        std::fs::write(Self::peers_path(&self.base_dir), encrypted)
            .map_err(|err| P2pError::StreamFailed(format!("failed to save peer cache: {err}")))
    }

    async fn load_peers(&self) -> Result<Vec<PeerAddr>, P2pError> {
        Ok(self.peers_cache.read().await.clone())
    }

    async fn save_device_info(
        &self, peer: &PeerIdentity, info: &DeviceInfo,
    ) -> Result<(), P2pError> {
        let mut device_info = self.device_info_cache.write().await;
        match device_info.iter_mut().find(|(cached_id, _)| cached_id.id == peer.id) {
            Some(entry) => entry.1 = info.clone(),
            None => device_info.push((peer.clone(), info.clone())),
        }

        let json = serde_json::to_vec(&*device_info).map_err(|err| {
            P2pError::StreamFailed(format!("failed to encode device info cache: {err}"))
        })?;
        let encrypted = encrypt(&self.master_key, &json);
        std::fs::write(Self::device_info_path(&self.base_dir), encrypted).map_err(|err| {
            P2pError::StreamFailed(format!("failed to save device info cache: {err}"))
        })
    }

    async fn load_device_info(&self) -> Result<Vec<(PeerIdentity, DeviceInfo)>, P2pError> {
        Ok(self.device_info_cache.read().await.clone())
    }
}

impl SecureP2pStorage {
    /// Removes a peer from the paired list — used by "unpair". Idempotent: an
    /// unknown id is not an error, it just does not rewrite the file needlessly. Not part of
    /// `P2PStorage` (trait shared with the Android side) on purpose — the lib never
    /// needs to remove a peer on its own, only the "unpair" UI calls this.
    pub async fn remove_peer(&self, id: &str) -> std::io::Result<()> {
        let mut peers = self.peers_cache.write().await;
        let before = peers.len();
        peers.retain(|peer| peer.id.id != id);
        if peers.len() != before {
            let json =
                serde_json::to_vec(&*peers).expect("Vec<PeerAddr> serialization cannot fail");
            std::fs::write(Self::peers_path(&self.base_dir), encrypt(&self.master_key, &json))?;
        }

        // Also clears the persisted name — without this, re-pairing the same `id` would make the
        // old name "come back to life" before the next handshake really updates it.
        let mut device_info = self.device_info_cache.write().await;
        let before = device_info.len();
        device_info.retain(|(cached_id, _)| cached_id.id != id);
        if device_info.len() != before {
            let json = serde_json::to_vec(&*device_info)
                .expect("Vec<(PeerIdentity, DeviceInfo)> serialization cannot fail");
            std::fs::write(
                Self::device_info_path(&self.base_dir),
                encrypt(&self.master_key, &json),
            )?;
        }

        Ok(())
    }

    /// Persists the ticket of the user's `services.iroh.computer` account (pasted on the
    /// network settings screen) — not part of `P2PStorage` (trait shared with
    /// Android) because it is specific to the Desktop relay UI, not something the lib itself needs
    /// to read/write on its own. Only takes effect on the next app start (same rule as the other
    /// relay sources in `RelaySettings::resolve`), because the lib does not support changing relay at
    /// runtime.
    pub async fn save_iroh_services_ticket(&self, ticket: &str) -> std::io::Result<()> {
        std::fs::write(
            Self::iroh_services_ticket_path(&self.base_dir),
            encrypt(&self.master_key, ticket.as_bytes()),
        )
    }

    /// `None` when the user never pasted a ticket, or when the file exists but does not
    /// decrypt (master key changed) — in that case it is treated as "not configured" instead
    /// of propagating an error, since the Iroh public network being unavailable should not block
    /// the startup of the rest of the P2P network (peers/history keep working without it).
    pub async fn load_iroh_services_ticket(&self) -> std::io::Result<Option<String>> {
        let encrypted = match std::fs::read(Self::iroh_services_ticket_path(&self.base_dir)) {
            Ok(bytes) => bytes,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(err) => return Err(err),
        };

        let ticket = decrypt(&self.master_key, &encrypted)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok());
        Ok(ticket)
    }

    /// Removes the saved ticket — used when the user unchecks the source or pastes a new value
    /// that replaces the old one.
    pub async fn clear_iroh_services_ticket(&self) -> std::io::Result<()> {
        match std::fs::remove_file(Self::iroh_services_ticket_path(&self.base_dir)) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err),
        }
    }
}

/// Delegates to an `Arc<SecureP2pStorage>` — makes the same storage passed to the builder
/// (`AcerolaP2p::builder().storage(...)`, which requires ownership) also reachable outside
/// of it. Without this, `paired_peers()` (`NetworkServiceApi`) would have no way to read the
/// persisted peers: the lib does not hand the storage back after `build()`.
pub struct SharedP2pStorage(pub Arc<SecureP2pStorage>);

#[async_trait]
impl P2PStorage for SharedP2pStorage {
    async fn save_identity(&self, secret: &[u8]) -> Result<(), P2pError> {
        self.0.save_identity(secret).await
    }

    async fn load_identity(&self) -> Result<Option<Vec<u8>>, P2pError> {
        self.0.load_identity().await
    }

    async fn save_peer(&self, peer: &PeerAddr) -> Result<(), P2pError> {
        self.0.save_peer(peer).await
    }

    async fn load_peers(&self) -> Result<Vec<PeerAddr>, P2pError> {
        self.0.load_peers().await
    }

    async fn save_device_info(
        &self, peer: &PeerIdentity, info: &DeviceInfo,
    ) -> Result<(), P2pError> {
        self.0.save_device_info(peer, info).await
    }

    async fn load_device_info(&self) -> Result<Vec<(PeerIdentity, DeviceInfo)>, P2pError> {
        self.0.load_device_info().await
    }
}

#[cfg(test)]
mod tests {
    use acerola_p2p::api::peer::PeerIdentity;

    use super::*;

    fn peer(id: &str) -> PeerAddr {
        PeerAddr { id: PeerIdentity { id: id.to_string(), device_id: None }, addrs: vec![1, 2, 3] }
    }

    #[tokio::test]
    async fn identity_roundtrip_through_encrypted_file() {
        let dir = tempfile::tempdir().unwrap();
        let storage = SecureP2pStorage::open(dir.path(), [5u8; 32]).unwrap();

        assert_eq!(storage.load_identity().await.unwrap(), None);

        storage.save_identity(b"my secret seed").await.unwrap();
        assert_eq!(storage.load_identity().await.unwrap(), Some(b"my secret seed".to_vec()));

        // The file on disk must not contain the secret in plain text.
        let raw = std::fs::read(dir.path().join("identity.enc")).unwrap();
        assert!(!raw.windows(b"my secret seed".len()).any(|window| window == b"my secret seed"));
    }

    #[tokio::test]
    async fn save_peer_upserts_by_id_and_persists_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let key = [9u8; 32];

        {
            let storage = SecureP2pStorage::open(dir.path(), key).unwrap();
            storage.save_peer(&peer("peer-a")).await.unwrap();
            storage.save_peer(&peer("peer-a")).await.unwrap(); // upsert, does not duplicate
            storage.save_peer(&peer("peer-b")).await.unwrap();
            assert_eq!(storage.load_peers().await.unwrap().len(), 2);
        }

        // Reopen (simulates an app restart) — the cache must come from the encrypted file.
        let reopened = SecureP2pStorage::open(dir.path(), key).unwrap();
        let peers = reopened.load_peers().await.unwrap();
        assert_eq!(peers.len(), 2);
        assert!(peers.iter().any(|p| p.id.id == "peer-a"));
        assert!(peers.iter().any(|p| p.id.id == "peer-b"));
    }

    /// Regression: `sync_comic`/`query_remote_library`/etc. build `PeerIdentity { id, device_id:
    /// None }` because the frontend only sends the peer string — but the initial pairing (handshake)
    /// saves with `device_id: Some(uuid)`. If the upsert compared the whole `PeerId` (as it did
    /// before), these two calls for the SAME peer would become two list entries, and the UI
    /// (`{#each pairedPeers as peer (peer.peerId)}`) breaks with a duplicate key.
    #[tokio::test]
    async fn save_peer_upserts_by_id_string_even_when_device_id_differs() {
        let dir = tempfile::tempdir().unwrap();
        let storage = SecureP2pStorage::open(dir.path(), [7u8; 32]).unwrap();

        let paired_via_handshake = PeerAddr {
            id: PeerIdentity { id: "peer-a".to_string(), device_id: Some("uuid-1".to_string()) },
            addrs: vec![1, 2, 3],
        };
        let reconnected_via_action = PeerAddr {
            id: PeerIdentity { id: "peer-a".to_string(), device_id: None },
            addrs: vec![4, 5, 6],
        };

        storage.save_peer(&paired_via_handshake).await.unwrap();
        storage.save_peer(&reconnected_via_action).await.unwrap();

        let peers = storage.load_peers().await.unwrap();
        assert_eq!(peers.len(), 1, "the same peer.id.id should update, not duplicate");
        assert_eq!(peers[0].addrs, vec![4, 5, 6]);
    }

    /// Migration: a `peers.enc` written BEFORE the upsert fix may have duplicates
    /// persisted on disk (same `id.id`, different `device_id`). `open()` must clean
    /// that up on read — without this, anyone who already hit the bug would keep seeing `pairedPeers`
    /// break the UI even after updating the app, because the duplicate was saved before the fix.
    #[tokio::test]
    async fn open_deduplicates_peers_already_persisted_before_the_fix() {
        let dir = tempfile::tempdir().unwrap();
        let key = [11u8; 32];

        let duplicated_on_disk = vec![
            PeerAddr {
                id: PeerIdentity {
                    id: "peer-a".to_string(),
                    device_id: Some("uuid-1".to_string()),
                },
                addrs: vec![1, 2, 3],
            },
            PeerAddr {
                id: PeerIdentity { id: "peer-a".to_string(), device_id: None },
                addrs: vec![4, 5, 6],
            },
        ];
        let json = serde_json::to_vec(&duplicated_on_disk).unwrap();
        std::fs::write(dir.path().join("peers.enc"), encrypt(&key, &json)).unwrap();

        let storage = SecureP2pStorage::open(dir.path(), key).unwrap();
        let peers = storage.load_peers().await.unwrap();
        assert_eq!(peers.len(), 1, "a pre-existing duplicate should be collapsed on read");
        assert_eq!(peers[0].addrs, vec![4, 5, 6], "keeps the last entry, same rule as the upsert");
    }

    #[tokio::test]
    async fn remove_peer_persists_and_survives_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let key = [12u8; 32];

        {
            let storage = SecureP2pStorage::open(dir.path(), key).unwrap();
            storage.save_peer(&peer("peer-a")).await.unwrap();
            storage.save_peer(&peer("peer-b")).await.unwrap();

            storage.remove_peer("peer-a").await.unwrap();

            let peers = storage.load_peers().await.unwrap();
            assert_eq!(peers.len(), 1);
            assert_eq!(peers[0].id.id, "peer-b");
        }

        let reopened = SecureP2pStorage::open(dir.path(), key).unwrap();
        let peers = reopened.load_peers().await.unwrap();
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].id.id, "peer-b");
    }

    fn device_info(name: &str) -> DeviceInfo {
        DeviceInfo {
            name: name.to_string(),
            os: "windows".to_string(),
            version: "1.0.0".to_string(),
        }
    }

    #[tokio::test]
    async fn device_info_persists_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let key = [21u8; 32];

        {
            let storage = SecureP2pStorage::open(dir.path(), key).unwrap();
            storage
                .save_device_info(&peer("peer-a").id, &device_info("PC do Vinicius"))
                .await
                .unwrap();
        }

        // Reopen (simulates an app restart) — the name must come from the encrypted file, not
        // only exist while the process was running.
        let reopened = SecureP2pStorage::open(dir.path(), key).unwrap();
        let loaded = reopened.load_device_info().await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].0.id, "peer-a");
        assert_eq!(loaded[0].1.name, "PC do Vinicius");
    }

    #[tokio::test]
    async fn save_device_info_upserts_by_id_instead_of_duplicating() {
        let dir = tempfile::tempdir().unwrap();
        let storage = SecureP2pStorage::open(dir.path(), [22u8; 32]).unwrap();

        storage.save_device_info(&peer("peer-a").id, &device_info("old-name")).await.unwrap();
        storage.save_device_info(&peer("peer-a").id, &device_info("new-name")).await.unwrap();

        let loaded = storage.load_device_info().await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].1.name, "new-name");
    }

    #[tokio::test]
    async fn remove_peer_also_forgets_its_persisted_device_name() {
        let dir = tempfile::tempdir().unwrap();
        let storage = SecureP2pStorage::open(dir.path(), [23u8; 32]).unwrap();

        storage.save_peer(&peer("peer-a")).await.unwrap();
        storage.save_device_info(&peer("peer-a").id, &device_info("PC do Vinicius")).await.unwrap();

        storage.remove_peer("peer-a").await.unwrap();

        assert!(storage.load_device_info().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn remove_peer_of_unknown_id_is_a_no_op() {
        let dir = tempfile::tempdir().unwrap();
        let storage = SecureP2pStorage::open(dir.path(), [13u8; 32]).unwrap();
        storage.save_peer(&peer("peer-a")).await.unwrap();

        storage.remove_peer("never-paired-peer").await.unwrap();

        assert_eq!(storage.load_peers().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn iroh_services_ticket_roundtrip_through_encrypted_file() {
        let dir = tempfile::tempdir().unwrap();
        let storage = SecureP2pStorage::open(dir.path(), [6u8; 32]).unwrap();

        assert_eq!(storage.load_iroh_services_ticket().await.unwrap(), None);

        storage.save_iroh_services_ticket("services-fake-ticket").await.unwrap();
        assert_eq!(
            storage.load_iroh_services_ticket().await.unwrap(),
            Some("services-fake-ticket".to_string())
        );

        // The file on disk must not contain the ticket in plain text.
        let raw = std::fs::read(dir.path().join("iroh_services_ticket.enc")).unwrap();
        assert!(!raw
            .windows("services-fake-ticket".len())
            .any(|window| window == b"services-fake-ticket"));
    }

    #[tokio::test]
    async fn iroh_services_ticket_persists_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let key = [14u8; 32];

        {
            let storage = SecureP2pStorage::open(dir.path(), key).unwrap();
            storage.save_iroh_services_ticket("services-fake-ticket").await.unwrap();
        }

        let reopened = SecureP2pStorage::open(dir.path(), key).unwrap();
        assert_eq!(
            reopened.load_iroh_services_ticket().await.unwrap(),
            Some("services-fake-ticket".to_string())
        );
    }

    #[tokio::test]
    async fn clear_iroh_services_ticket_removes_it_and_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let storage = SecureP2pStorage::open(dir.path(), [15u8; 32]).unwrap();

        storage.save_iroh_services_ticket("services-fake-ticket").await.unwrap();
        storage.clear_iroh_services_ticket().await.unwrap();
        assert_eq!(storage.load_iroh_services_ticket().await.unwrap(), None);

        // Calling again with nothing saved is not an error.
        storage.clear_iroh_services_ticket().await.unwrap();
    }

    #[tokio::test]
    async fn reopen_with_wrong_key_fails_loudly_instead_of_silently_dropping_peers() {
        let dir = tempfile::tempdir().unwrap();
        let storage = SecureP2pStorage::open(dir.path(), [1u8; 32]).unwrap();
        storage.save_peer(&peer("secret-peer")).await.unwrap();

        // Wrong key (e.g. the keyring resolved another key in this run) — decrypt fails.
        // Regression: this used to silently degrade into an empty list, and the next
        // `save_peer` rewrote `peers.enc` from scratch, wiping forever what was
        // saved with the old key. Now it must fail loudly in `open()`, not open with
        // an empty cache as if nothing had happened.
        let reopen_result = SecureP2pStorage::open(dir.path(), [2u8; 32]);
        assert!(
            reopen_result.is_err(),
            "opening with the wrong key should fail, not silently degrade"
        );

        // And the original file stays intact on disk — nothing was overwritten.
        let reopened_with_correct_key = SecureP2pStorage::open(dir.path(), [1u8; 32]).unwrap();
        assert_eq!(reopened_with_correct_key.load_peers().await.unwrap().len(), 1);
    }
}
