use std::{
    collections::HashSet,
    future::Future,
    path::PathBuf,
    pin::Pin,
    sync::{Arc, RwLock},
};

use acerola_p2p::api::{
    guard::{InMemoryTrustedStore, TofuGuard, TrustedPeerStore},
    identity::DeviceInfo,
    network::NetworkMode,
    peer::{PeerAddr, PeerIdentity},
    storage::P2PStorage,
    AcerolaP2p,
};
use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::infra::security::{p2p_storage::SecureP2pStorage, trusted_store::SecureTrustedStore};

pub type ConnectedPeerInfo = (PeerIdentity, HashSet<Vec<u8>>, Option<DeviceInfo>);

/// Rebuilds an `AcerolaP2p` from scratch (same identity/storage/handlers, relay config
/// re-read on the spot) — provided by `bios::network::setup_network`, which holds all the
/// protocol-specific ingredients (`HistorySyncInbound`, `FileSyncInbound`, etc.) that
/// `NetworkService` does not need to know. `NetworkService` only knows how to "ask for a new
/// node", never how one is built — the same separation that already existed between
/// `bios::network` (builds) and `NetworkService` (operates what was built).
pub type NodeBuilder = Arc<
    dyn Fn() -> Pin<Box<dyn Future<Output = Result<Arc<AcerolaP2p>, String>> + Send>> + Send + Sync,
>;

// `#[async_trait]` generates `#[must_use]` on methods that already return a `Future`
// (`must_use` by itself) — clippy >= 1.99 flags this as `double_must_use` in the macro-expanded code.
#[allow(clippy::double_must_use)]
#[async_trait]
pub trait NetworkServiceApi: Send + Sync + 'static {
    fn local_id(&self) -> Result<String, String>;
    fn local_addr(&self) -> Result<PeerAddr, String>;
    async fn local_device_info(&self) -> Result<DeviceInfo, String>;
    /// Overrides the local device's display name (custom alias, LocalSend style). Applies
    /// from the next handshake on — no app restart needed (see
    /// `AcerolaP2p::set_local_device_name`). Persisting it across restarts is the caller's
    /// job (the frontend writes it to `settings.json`, `bios::network::setup_network` reapplies it
    /// on the next boot).
    async fn set_local_device_name(&self, name: String) -> Result<(), String>;
    async fn connected_peers_with_info(&self) -> Result<Vec<ConnectedPeerInfo>, String>;
    /// Every peer ever paired (TOFU), with the last known address — persists
    /// across restarts and does not depend on an active connection right now, unlike
    /// [`NetworkServiceApi::connected_peers_with_info`] (protocol session, lasts only
    /// seconds). This is the list that must feed "start sync with X" in the UI, since the peer
    /// is almost never connected at the exact moment the user clicks the button.
    ///
    /// `DeviceInfo` comes from `AcerolaP2p::known_peers()` (same persistent mechanism,
    /// survives the handshake closing) — not from `connected_peers_with_info`, which only has data
    /// during the few seconds the handshake session is actually open.
    async fn paired_peers(&self) -> Result<Vec<(PeerAddr, Option<DeviceInfo>)>, String>;
    /// Unpairs a peer: it disappears from trust (TOFU) and from the known address cache —
    /// the same pair of sources [`NetworkServiceApi::paired_peers`] reads. Does not drop an
    /// active connection nor block the peer; if it reconnects later, it goes through the same
    /// TOFU flow as a never-seen device (same behavior as the Android side, see
    /// `P2PNode::remove_paired_peer`).
    async fn remove_peer(&self, id: String) -> Result<(), String>;
    async fn switch_to_local(&self) -> Result<(), String>;
    async fn switch_to_relay(&self) -> Result<(), String>;
    async fn mode(&self) -> Result<NetworkMode, String>;
    async fn connect(&self, peer_addr: PeerAddr, alpn: Vec<u8>) -> Result<(), String>;
    async fn shutdown(&self) -> Result<(), String>;
    /// `true` if the user already pasted and saved a ticket of their own account on
    /// `services.iroh.computer` — never returns the value itself (it is a real credential).
    async fn has_iroh_services_ticket(&self) -> Result<bool, String>;
    /// Validates the format before persisting (rejects an incomplete/wrong paste right away,
    /// instead of only failing later). Only actually takes effect after the frontend calls
    /// [`Self::apply_relay_settings`] — which now restarts the node (see [`Self::restart`]), it no
    /// longer needs a fresh boot of the whole app.
    async fn set_iroh_services_ticket(&self, ticket: String) -> Result<(), String>;
    async fn clear_iroh_services_ticket(&self) -> Result<(), String>;
    /// Re-reads `settings.json` + the vault ticket, resolves the combinable relay config and
    /// applies it to the ALREADY LIVE node (`AcerolaP2p::apply_relay_mode`) — without this, changing
    /// the relay config in the UI only took effect on the app's NEXT boot. Called by the frontend
    /// after any change to the relay sources (Acerola/Iroh toggle, add/remove own URL, ticket).
    async fn apply_relay_settings(&self) -> Result<(), String>;
    /// Fully shuts down the P2P node (really closes the `Endpoint` — see `AcerolaP2p::shutdown`
    /// — not just dropping the command channel) and rebuilds it from scratch with the same
    /// identity/storage, re-reading the current relay config. LocalSend style: any relay change
    /// resets the whole module instead of mutating an already-live `Endpoint` — it was exactly
    /// that live mutation (`apply_relay_mode`) that left QUIC connections in a confused state
    /// after a big enough network change (e.g. own relay -> Iroh public network): "dropping
    /// unexpected packet" in the log, dialing repeatedly timing out without ever resolving.
    /// Also exposed as a command for the user to trigger manually ("Restart" on the Network
    /// screen), not only automatically in [`Self::apply_relay_settings`].
    async fn restart(&self) -> Result<(), String>;
}

pub struct NetworkService {
    node: RwLock<Option<Arc<AcerolaP2p>>>,
    /// Same storage passed to the builder (`.storage(...)`) — cloned beforehand by
    /// `bios::network::setup_network` to survive here as the source of persisted "paired
    /// peers", since the lib does not hand the storage back after `build()`. Concrete type
    /// (not `Arc<dyn P2PStorage>`) because [`Self::remove_peer`] needs
    /// `SecureP2pStorage::remove_peer`, which is specific to this implementation — it is not part
    /// of the trait shared with the Android side.
    storage: Arc<SecureP2pStorage>,
    /// Same reason as the field above, for the trust (TOFU) side — `SecureTrustedStore::remove`
    /// is not part of `TrustedPeerStore` either.
    trust_store: Arc<SecureTrustedStore>,
    /// Only so [`Self::apply_relay_settings`] can re-read `settings.json` on demand — same
    /// folder already used by `bios::network::setup_network` at startup.
    app_data_directory: PathBuf,
    /// Provided by `bios::network::setup_network` — see the docs of [`NodeBuilder`]. Only called
    /// by [`Self::restart`].
    rebuild_node: NodeBuilder,
    /// Serializes [`Self::restart`] — without it, two overlapping calls (e.g. two relay toggles
    /// in a row on the Network screen, or the toggle and the manual "Restart" button almost
    /// together; Tauri does not queue `invoke`s, each runs in its own task) build TWO
    /// nodes in parallel with the SAME persisted identity — both try to register at the
    /// same time on the same relay, which accepts only one and silently drops the other
    /// ("Another endpoint connected with the same endpoint id", seen live). `tokio::sync`
    /// (not `std::sync`) because the lock must stay held safely across several `.await`s
    /// (old node shutdown + async rebuild), not just a synchronous critical section
    /// like [`Self::node`].
    restart_lock: Mutex<()>,
}

impl NetworkService {
    pub fn new(
        node: Arc<AcerolaP2p>, storage: Arc<SecureP2pStorage>,
        trust_store: Arc<SecureTrustedStore>, app_data_directory: PathBuf,
        rebuild_node: NodeBuilder,
    ) -> Self {
        Self {
            node: RwLock::new(Some(node)),
            storage,
            trust_store,
            app_data_directory,
            rebuild_node,
            restart_lock: Mutex::new(()),
        }
    }

    pub fn new_uninitialized(
        storage: Arc<SecureP2pStorage>, trust_store: Arc<SecureTrustedStore>,
        app_data_directory: PathBuf, rebuild_node: NodeBuilder,
    ) -> Self {
        Self {
            node: RwLock::new(None),
            storage,
            trust_store,
            app_data_directory,
            rebuild_node,
            restart_lock: Mutex::new(()),
        }
    }

    pub fn set_node(&self, node: Arc<AcerolaP2p>) {
        let mut write = self.node.write().unwrap_or_else(|poisoned| poisoned.into_inner());
        *write = Some(node);
    }

    /// Clones the current `Arc` and releases the lock right away — each caller operates on the
    /// node that was alive at the moment of the call, without holding the lock for the whole
    /// duration of a potentially long network operation (and without risking a deadlock if that
    /// operation, directly or indirectly, ends up calling back into a method of this service).
    ///
    /// `std::sync::RwLock` (not `tokio::sync::RwLock`) on purpose: the critical section here
    /// is just a `clone()`, it never crosses an `.await` — using the synchronous version avoids
    /// `blocking_read()` entirely (which can panic inside the tokio runtime, exactly where
    /// [`Self::local_id`]/[`Self::local_addr`] — the trait's two SYNCHRONOUS methods — are
    /// called) without having to make this function `async` just because of the other methods.
    fn node(&self) -> Result<Arc<AcerolaP2p>, String> {
        self.node
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .cloned()
            .ok_or_else(|| "P2P network service is initializing".to_string())
    }
}

#[async_trait]
impl NetworkServiceApi for NetworkService {
    fn local_id(&self) -> Result<String, String> {
        Ok(self.node()?.local_id().to_string())
    }

    /// Full address (id + addressing bytes) used to generate the pairing code/QR —
    /// it is what the other device needs to reach us via `connect()`.
    fn local_addr(&self) -> Result<PeerAddr, String> {
        self.node()?.local_addr().map_err(|err| err.to_string())
    }

    /// Name/OS/version of this device — used on the Network screen to show something more
    /// readable than the raw peer id (e.g. "Vinicius's Notebook" instead of a 64-char hex).
    async fn local_device_info(&self) -> Result<DeviceInfo, String> {
        Ok(self.node()?.local_device_info().await)
    }

    async fn set_local_device_name(&self, name: String) -> Result<(), String> {
        self.node()?.set_local_device_name(name).await;
        Ok(())
    }

    async fn connected_peers_with_info(&self) -> Result<Vec<ConnectedPeerInfo>, String> {
        match self.node() {
            Ok(node) => Ok(node.connected_peers_with_info().await),
            Err(_) => Ok(vec![]),
        }
    }

    async fn paired_peers(&self) -> Result<Vec<(PeerAddr, Option<DeviceInfo>)>, String> {
        use std::collections::HashMap;

        let peers = self.storage.load_peers().await.map_err(|err| err.to_string())?;

        let device_info_by_peer: HashMap<String, DeviceInfo> = match self.node() {
            Ok(node) => node
                .known_peers()
                .await
                .into_iter()
                .filter_map(|(peer, _, info)| info.map(|device| (peer.id, device)))
                .collect(),
            Err(_) => HashMap::new(),
        };

        Ok(peers
            .into_iter()
            .map(|addr| {
                let device = device_info_by_peer.get(&addr.id.id).cloned();
                (addr, device)
            })
            .collect())
    }

    async fn remove_peer(&self, id: String) -> Result<(), String> {
        self.trust_store.remove(&id).await.map_err(|err| err.to_string())?;
        self.storage.remove_peer(&id).await.map_err(|err| err.to_string())
    }

    async fn switch_to_local(&self) -> Result<(), String> {
        let store = Arc::new(InMemoryTrustedStore::new());
        let guard = TofuGuard::new(store as Arc<dyn TrustedPeerStore>).into_validator();
        self.node()?.switch_guard(guard, NetworkMode::Local).await.map_err(|err| err.to_string())
    }

    async fn switch_to_relay(&self) -> Result<(), String> {
        let store = Arc::new(InMemoryTrustedStore::new());
        let guard = TofuGuard::new(store as Arc<dyn TrustedPeerStore>).into_validator();
        self.node()?.switch_guard(guard, NetworkMode::Relay).await.map_err(|err| err.to_string())
    }

    async fn mode(&self) -> Result<NetworkMode, String> {
        match self.node() {
            Ok(node) => Ok(node.mode().await),
            Err(_) => Ok(NetworkMode::Local),
        }
    }

    async fn connect(&self, peer_addr: PeerAddr, alpn: Vec<u8>) -> Result<(), String> {
        self.node()?.connect(peer_addr, &alpn).await.map_err(|err| err.to_string())
    }

    async fn shutdown(&self) -> Result<(), String> {
        if let Ok(node) = self.node() {
            node.shutdown().await.map_err(|err| err.to_string())?;
        }
        Ok(())
    }

    async fn has_iroh_services_ticket(&self) -> Result<bool, String> {
        Ok(self.storage.load_iroh_services_ticket().await.map_err(|err| err.to_string())?.is_some())
    }

    async fn set_iroh_services_ticket(&self, ticket: String) -> Result<(), String> {
        let trimmed = ticket.trim();
        acerola_p2p::api::transport::validate_iroh_services_ticket(trimmed)
            .map_err(|err| err.to_string())?;
        self.storage.save_iroh_services_ticket(trimmed).await.map_err(|err| err.to_string())
    }

    async fn clear_iroh_services_ticket(&self) -> Result<(), String> {
        self.storage.clear_iroh_services_ticket().await.map_err(|err| err.to_string())
    }

    /// It used to mutate the already-live `Endpoint` (`apply_relay_mode`) — now it delegates to
    /// [`Self::restart`], which resets the whole module. See the docs of [`NetworkServiceApi::restart`]
    /// for why: a big enough relay change (e.g. own relay -> Iroh public
    /// network) left QUIC connections in a confused state when only reconfigured live.
    async fn apply_relay_settings(&self) -> Result<(), String> {
        self.restart().await
    }

    async fn restart(&self) -> Result<(), String> {
        // Holds the lock for the WHOLE operation (not just the pointer swap at the end) — a
        // second concurrent call waits for this one to really finish (old node shut down +
        // new node already registered on the relay) before starting its own. See the docs of
        // `Self::restart_lock` for the real bug this prevents.
        let _restart_guard = self.restart_lock.lock().await;

        if let Ok(old_node) = self.node() {
            // Best-effort: even if shutting down the old node fails/partially hangs, it is still
            // worth trying to bring up a new node instead of leaving the user with no network at all —
            // log it but do not abort the restart because of it.
            if let Err(error) = old_node.shutdown().await {
                tracing::warn!(
                    ?error,
                    "[NetworkService] failed to cleanly shut down old p2p node before restart"
                );
            }
        }

        let fresh_node = (self.rebuild_node)().await?;
        *self.node.write().unwrap_or_else(|poisoned| poisoned.into_inner()) =
            Some(Arc::clone(&fresh_node));

        tracing::info!("[NetworkService] P2P node restarted");

        // Without this, a restart returned the node in a clean state — identity and storage
        // preserved, but ZERO active connections — and nothing tried to talk to any paired peer
        // again until the user triggered some manual action (browse library, sync, etc). Found
        // live: switching to a relay a paired peer does not share cuts relay reachability to it,
        // and the user had NO signal of that other than trying something manually
        // and seeing it fail. `reconnect_known_peers` only QUEUES the attempt (it does not wait for
        // confirmation — see the docs there); best-effort: failing to read `paired_peers` must not
        // make the restart itself (already successfully completed above) look like it failed.
        //
        // No dedicated test for the `Err` branch below: `SecureP2pStorage::load_peers` only reads an
        // in-memory cache (`peers_cache.read().await.clone()`) and currently has no way to
        // really fail — the `Result` exists because it is the `P2PStorage` trait contract
        // (future implementations, or another platform, may genuinely fail here), not
        // because this path is reachable with the current concrete storage. Handled anyway
        // (no `.unwrap()`/`.expect()`, per the CONTRIBUTING.md rule) to respect the type's
        // contract, not because test coverage is possible right now.
        match self.storage.load_peers().await {
            Ok(known_peers) => {
                acerola_p2p::api::network::reconnect_known_peers(&fresh_node, known_peers).await;
            },
            Err(error) => {
                tracing::warn!(
                    ?error,
                    "[NetworkService] failed to load paired peers for post-restart reconnect"
                );
            },
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use acerola_p2p::api::{identity::DeviceInfo, transport::IrohTransportBuilder};

    use super::*;

    fn no_op_emitter() -> acerola_p2p::api::protocol::EventEmitter {
        Arc::new(|_event: &str, _payload: String| {})
    }

    fn test_device_info() -> DeviceInfo {
        DeviceInfo {
            name: "test-device".to_string(),
            os: "test-os".to_string(),
            version: "0.0.0".to_string(),
        }
    }

    /// Real node (actual Iroh, without `.storage(...)`) — identity generated from scratch on every
    /// call, which is exactly the property the tests below rely on to prove that
    /// `restart()` really swapped the node, and did not just rerun something on top of the same one.
    async fn build_test_node() -> Arc<AcerolaP2p> {
        Arc::new(
            AcerolaP2p::builder(
                no_op_emitter(),
                IrohTransportBuilder::default(),
                test_device_info(),
            )
            .build()
            .await
            .unwrap(),
        )
    }

    fn open_storage() -> (Arc<SecureP2pStorage>, Arc<SecureTrustedStore>, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let key = [7u8; 32];
        let storage = Arc::new(SecureP2pStorage::open(dir.path(), key).unwrap());
        let trust = Arc::new(SecureTrustedStore::open(dir.path(), key).unwrap());
        (storage, trust, dir)
    }

    /// Core regression of this change: `restart()` must (1) really shut down the old node
    /// (`AcerolaP2p::shutdown`, not just drop the command channel — see the fix in
    /// `lib/p2p`), (2) call the rebuild closure provided by `bios::network::setup_network`
    /// exactly once, and (3) make any following call (`local_id()`) reflect the NEW node,
    /// not the old one. Two real Iroh nodes (without `.storage(...)`, always a new identity)
    /// make "the id changed" direct proof that the swap really happened.
    #[tokio::test]
    async fn restart_shuts_down_old_node_and_swaps_in_a_freshly_built_one() {
        let (storage, trust, _dir) = open_storage();
        let old_node = build_test_node().await;
        let old_local_id = old_node.local_id().to_string();

        let rebuild_calls = Arc::new(AtomicUsize::new(0));
        let rebuild_calls_clone = Arc::clone(&rebuild_calls);
        let rebuild_node: NodeBuilder = Arc::new(move || {
            let rebuild_calls = Arc::clone(&rebuild_calls_clone);
            Box::pin(async move {
                rebuild_calls.fetch_add(1, Ordering::SeqCst);
                Ok(build_test_node().await)
            })
        });

        let service =
            NetworkService::new(old_node, storage, trust, std::env::temp_dir(), rebuild_node);

        service.restart().await.expect("restart should succeed");

        assert_eq!(
            rebuild_calls.load(Ordering::SeqCst),
            1,
            "the rebuild closure should have been called exactly once"
        );
        let new_local_id = service.local_id().unwrap();
        assert_ne!(
            old_local_id, new_local_id,
            "restart should have swapped to a new node (different identity)"
        );
    }

    /// If the rebuild closure fails, `restart()` propagates the error instead of masking a broken
    /// state — the old node was already shut down at that point, so the user NEEDS to know the
    /// restart did not complete, not see "success" and only find out when the network stops
    /// responding.
    #[tokio::test]
    async fn restart_propagates_rebuild_failure() {
        let (storage, trust, _dir) = open_storage();
        let old_node = build_test_node().await;

        let rebuild_node: NodeBuilder = Arc::new(|| Box::pin(async { Err("boom".to_string()) }));

        let service =
            NetworkService::new(old_node, storage, trust, std::env::temp_dir(), rebuild_node);

        let result = service.restart().await;
        assert_eq!(result, Err("boom".to_string()));
    }

    /// Regression for the live bug: two relay changes triggered in a row (two toggles,
    /// or the toggle and the manual "Restart" button) each call `restart()` in their own task —
    /// Tauri does not queue `invoke`s. Without mutual exclusion, both rebuilds ran in
    /// parallel with the SAME persisted identity, and the relay saw two concurrent endpoints
    /// trying to register with the same id ("Another endpoint connected with the same
    /// endpoint id" — the real message seen in production). Proves `restart()` serializes: the
    /// rebuild closure never observes more than 1 concurrent run, even with two calls
    /// triggered at the same time via `tokio::join!`.
    #[tokio::test]
    async fn concurrent_restarts_are_serialized_instead_of_racing() {
        let (storage, trust, _dir) = open_storage();
        let old_node = build_test_node().await;

        let concurrent = Arc::new(AtomicUsize::new(0));
        let max_concurrent = Arc::new(AtomicUsize::new(0));
        let rebuild_calls = Arc::new(AtomicUsize::new(0));

        let concurrent_clone = Arc::clone(&concurrent);
        let max_concurrent_clone = Arc::clone(&max_concurrent);
        let rebuild_calls_clone = Arc::clone(&rebuild_calls);
        let rebuild_node: NodeBuilder = Arc::new(move || {
            let concurrent = Arc::clone(&concurrent_clone);
            let max_concurrent = Arc::clone(&max_concurrent_clone);
            let rebuild_calls = Arc::clone(&rebuild_calls_clone);
            Box::pin(async move {
                rebuild_calls.fetch_add(1, Ordering::SeqCst);
                let now_in_flight = concurrent.fetch_add(1, Ordering::SeqCst) + 1;
                max_concurrent.fetch_max(now_in_flight, Ordering::SeqCst);
                // Enough room for a second concurrent call to ENTER `restart()`
                // while this one is still building — if the lock is not protecting the whole
                // operation, this is where both rebuilds overlap.
                tokio::time::sleep(std::time::Duration::from_millis(30)).await;
                concurrent.fetch_sub(1, Ordering::SeqCst);
                Ok(build_test_node().await)
            })
        });

        let service = Arc::new(NetworkService::new(
            old_node,
            storage,
            trust,
            std::env::temp_dir(),
            rebuild_node,
        ));

        let service_a = Arc::clone(&service);
        let service_b = Arc::clone(&service);
        let (result_a, result_b) =
            tokio::join!(async move { service_a.restart().await }, async move {
                service_b.restart().await
            });

        result_a.expect("first restart call should succeed");
        result_b.expect("second restart call should succeed, not be discarded");

        assert_eq!(
            rebuild_calls.load(Ordering::SeqCst),
            2,
            "both calls should have rebuilt the node, none was ignored"
        );
        assert_eq!(
            max_concurrent.load(Ordering::SeqCst),
            1,
            "both rebuilds ran at the same time — restart() is not serializing"
        );
    }

    /// Tests that bring up REAL Iroh nodes (actual mDNS/QUIC) and wait for reconnection within a
    /// fixed timeout (50 attempts * 50ms = 2.5s) — running in parallel with the rest of the suite (or
    /// with each other), port/mDNS contention under load can push a legitimate reconnection past
    /// that timeout, failing the test with no real bug involved (seen live in CI: 2-3
    /// of these tests failing alternately between runs). `[test-groups]
    /// run-in-isolation` in `.config/nextest.toml` already existed for this (filter
    /// `test(::run_in_isolation::)`), but no test used the module yet — moving the 3 tests
    /// that depend on timed reconnection here serializes them (`max-threads = 1`), removing the
    /// contention that caused the flakiness. Module nested inside `tests` (not a sibling) to
    /// reuse the private helpers (`build_test_node`/`open_storage`) via `use super::*`.
    mod run_in_isolation {
        use super::*;

        /// End-to-end regression for the live bug: a restart (relay change, or the manual button)
        /// returned a node with identity/storage preserved but ZERO active connections — nothing
        /// tried to talk to already-paired peers again until the user triggered some manual action
        /// (browse library, sync, etc). Proves `restart()` now reconnects ON ITS OWN with a real
        /// already-paired peer, with no action other than the `restart()` call itself.
        #[tokio::test]
        async fn restart_reconnects_to_previously_paired_peers_without_further_action() {
            let (storage, trust, _dir) = open_storage();
            let old_node = build_test_node().await;

            let peer_node = build_test_node().await;
            let peer_addr = peer_node.local_addr().unwrap();
            let peer_id = PeerIdentity {
                id: peer_node.local_id().to_string(),
                device_id: peer_node.local_device_id().map(|s| s.to_string()),
            };
            storage.save_peer(&peer_addr).await.expect("seeding paired peer should succeed");

            let rebuild_node: NodeBuilder =
                Arc::new(|| Box::pin(async { Ok(build_test_node().await) }));

            let service = NetworkService::new(
                old_node,
                Arc::clone(&storage),
                trust,
                std::env::temp_dir(),
                rebuild_node,
            );

            service.restart().await.expect("restart should succeed");

            let mut reconnected = false;
            for _ in 0..50 {
                let paired = service.paired_peers().await.expect("paired_peers should succeed");
                if paired.iter().any(|(addr, info)| addr.id == peer_id && info.is_some()) {
                    reconnected = true;
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }

            assert!(
                reconnected,
                "restart() should have reconnected on its own to the already-paired peer, without \
             manual user action"
            );
        }

        /// Sad path: a paired peer may have left the network between sessions (closed the app,
        /// lost connection) — that must not make `restart()` fail nor prevent reconnecting to
        /// ANOTHER paired peer that is still reachable. `reconnect_known_peers` is best-effort by
        /// design (see the docs in `lib/p2p`), but only an integration-level test (real storage +
        /// two real peers, one of them shut down) proves the composition of both sides does not break
        /// that contract.
        #[tokio::test]
        async fn restart_still_succeeds_and_reconnects_reachable_peer_when_another_paired_peer_is_offline(
        ) {
            let (storage, trust, _dir) = open_storage();
            let old_node = build_test_node().await;

            let reachable_peer = build_test_node().await;
            let reachable_addr = reachable_peer.local_addr().unwrap();
            let reachable_id = PeerIdentity {
                id: reachable_peer.local_id().to_string(),
                device_id: reachable_peer.local_device_id().map(|s| s.to_string()),
            };

            let offline_peer = build_test_node().await;
            let offline_addr = offline_peer.local_addr().unwrap();
            offline_peer.shutdown().await.expect("shutdown should succeed");

            storage
                .save_peer(&reachable_addr)
                .await
                .expect("seeding reachable peer should succeed");
            storage.save_peer(&offline_addr).await.expect("seeding offline peer should succeed");

            let rebuild_node: NodeBuilder =
                Arc::new(|| Box::pin(async { Ok(build_test_node().await) }));

            let service = NetworkService::new(
                old_node,
                Arc::clone(&storage),
                trust,
                std::env::temp_dir(),
                rebuild_node,
            );

            service
                .restart()
                .await
                .expect("restart should succeed even with an offline paired peer");

            let mut reconnected = false;
            for _ in 0..50 {
                let paired = service.paired_peers().await.expect("paired_peers should succeed");
                if paired.iter().any(|(addr, info)| addr.id == reachable_id && info.is_some()) {
                    reconnected = true;
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }

            assert!(
                reconnected,
                "an offline paired peer in the same restart must not prevent reconnecting to the \
             peer that is still reachable"
            );
        }

        /// Interaction between the two recent fixes (mutex serializing `restart()` +
        /// `reconnect_known_peers` triggered at its end): the reconnection the FIRST call
        /// triggers runs against the node IT built, but if a SECOND concurrent call
        /// comes in right after (waiting for the first to release the lock), the first's node was already
        /// shut down by that second call before any background reconnection retry
        /// finishes. That must not panic nor leave the paired peer without ANY reconnection
        /// attempt in the end — the reconnection triggered by the call that effectively won
        /// last (the most recent to finish) is the one that must have really happened.
        #[tokio::test]
        async fn concurrent_restarts_with_a_paired_peer_do_not_panic_and_still_reconnect() {
            let (storage, trust, _dir) = open_storage();
            let old_node = build_test_node().await;

            let peer_node = build_test_node().await;
            let peer_addr = peer_node.local_addr().unwrap();
            let peer_id = PeerIdentity {
                id: peer_node.local_id().to_string(),
                device_id: peer_node.local_device_id().map(|s| s.to_string()),
            };
            storage.save_peer(&peer_addr).await.expect("seeding paired peer should succeed");

            let rebuild_node: NodeBuilder = Arc::new(|| {
                Box::pin(async {
                    // Same spacing as `concurrent_restarts_are_serialized_instead_of_racing`
                    // — ensures the second call really waits for the first to be in the middle of its
                    // rebuild before trying to enter, exercising the node transition BEFORE the
                    // first call's reconnection has any chance to resolve.
                    tokio::time::sleep(std::time::Duration::from_millis(30)).await;
                    Ok(build_test_node().await)
                })
            });

            let service = Arc::new(NetworkService::new(
                old_node,
                Arc::clone(&storage),
                trust,
                std::env::temp_dir(),
                rebuild_node,
            ));

            let service_a = Arc::clone(&service);
            let service_b = Arc::clone(&service);
            let (result_a, result_b) =
                tokio::join!(async move { service_a.restart().await }, async move {
                    service_b.restart().await
                });

            result_a.expect("first restart call should succeed");
            result_b.expect("second restart call should succeed");

            let mut reconnected = false;
            for _ in 0..50 {
                let paired = service.paired_peers().await.expect("paired_peers should succeed");
                if paired.iter().any(|(addr, info)| addr.id == peer_id && info.is_some()) {
                    reconnected = true;
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }

            assert!(
                reconnected,
                "after two concurrent restarts, the final node should still have reconnected \
             to the paired peer — no panic nor silent loss of the reconnection attempt"
            );
        }
    }
}
