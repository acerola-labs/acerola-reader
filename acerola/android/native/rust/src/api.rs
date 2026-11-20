use acerola_p2p::api::AcerolaP2p;
use std::sync::{Arc, RwLock};
use tokio::runtime::Runtime;

#[cfg(target_os = "android")]
use acerola_p2p::api::{
    blobs::IrohBlobsConfig,
    guard::TofuGuard,
    identity::{DefaultDeviceInfoProvider, DeviceInfoProvider},
    network::NetworkMode,
    peer::{PeerAddr, PeerIdentity},
    storage::P2PStorage,
    transport::{IrohTransportBuilder, RelayModeConfig},
};

#[cfg(target_os = "android")]
use crate::{
    callbacks::{CoverBrowseProvider, FileSyncProvider, HistorySyncProvider, SecureBlobStore},
    mode::FfiNetworkMode,
    protocol::{
        cover_browse::{
            CoverBrowseInbound, CoverBrowseOutbound, PendingCoverRequestRegistry, COVER_BROWSE_ALPN,
        },
        files::{
            BlobChapterTransfer, ChapterTransfer, ComicSyncInbound, ComicSyncOutbound,
            FileSyncInbound, FileSyncOutbound, FileSyncSessionGuard, PendingComicScope,
            COMIC_SYNC_ALPN, FILE_SYNC_ALPN,
        },
        history::{
            HistoryEntrySyncInbound, HistoryEntrySyncOutbound, HistorySyncInbound,
            HistorySyncOutbound, PendingHistoryEntryScope, HISTORY_ENTRY_SYNC_ALPN,
            HISTORY_SYNC_ALPN,
        },
        library_browse::{LibraryBrowseInbound, LibraryBrowseOutbound, LIBRARY_BROWSE_ALPN},
    },
    relay_settings::{FfiRelaySettings, RelayTicketError},
    singleton::TOKIO_RUNTIME,
    storage::{SecureP2pStorage, SharedSecureP2pStorage},
    sync_direction::FfiSyncDirection,
    trust_store::SecureTrustedStore,
};

#[cfg(target_os = "android")]
use std::{collections::HashMap, sync::Mutex};

#[uniffi::export(with_foreign)]
pub trait P2PCallback: Send + Sync {
    fn on_event(&self, event: String, data: String);
}

#[derive(uniffi::Record)]
pub struct FfiPeerAddr {
    pub id: String,
    pub device_id: Option<String>,
    pub addrs: Vec<u8>,
}

/// Item of `get_paired_peers` — same as `FfiPeerAddr`, plus `device_name`. A separate
/// type (instead of adding the field to `FfiPeerAddr`) so every call site that builds an
/// `FfiPeerAddr` to dial (`connect`/`sync_comic`/`browse_library`/`browse_cover`) is not
/// forced to pass a `device_name` that makes no sense in those cases.
#[derive(uniffi::Record)]
pub struct FfiPairedPeer {
    pub id: String,
    pub device_id: Option<String>,
    pub addrs: Vec<u8>,
    /// Comes from `AcerolaP2p::known_peers()` (persists across restarts and survives the handshake
    /// closing), not from `connected_peers_with_info()` (only has data for the few seconds in which
    /// the handshake connection is actually open) — that wrong source was what left
    /// `device_name` almost always `None` in the UI, falling back to the raw peer id.
    pub device_name: Option<String>,
}

#[derive(uniffi::Record)]
pub struct FfiConnectedPeer {
    pub peer_id: String,
    pub alpns: Vec<Vec<u8>>,
    pub device_name: Option<String>,
}

/// Stable ingredients (identity, storage, handlers) to assemble an `AcerolaP2p` — extracted
/// from the body of `P2PNode::new` so it can be called again in `P2PNode::restart` without
/// duplicating the whole builder chain. No field is consumed in `build()` (all cloneable
/// `Arc`/`String`), so the same `P2pNodeContext` survives as many restarts as needed —
/// mirrors Desktop's `P2pNodeContext` (`bios/network.rs`).
#[cfg(target_os = "android")]
struct P2pNodeContext {
    emit: acerola_p2p::api::protocol::EventEmitter,
    trust_store: Arc<SecureTrustedStore>,
    storage: Arc<SecureP2pStorage>,
    history_provider: Arc<dyn HistorySyncProvider>,
    file_provider: Arc<dyn FileSyncProvider>,
    cover_provider: Arc<dyn CoverBrowseProvider>,
    file_sync_guard: Arc<FileSyncSessionGuard>,
    pending_comic_scope: PendingComicScope,
    pending_history_entry_scope: PendingHistoryEntryScope,
    pending_cover_scope: Arc<PendingCoverRequestRegistry>,
    chapter_transfer: Arc<dyn ChapterTransfer>,
    blob_context: Arc<crate::protocol::blob_context::BlobContext>,
    device_name: String,
    device_version: String,
}

/// Reads the Iroh Services ticket, logging any real backend failure — used at boot
/// (`P2PNode::new`), in `restart()` and in `apply_relay_settings()`. Without this, a vault read
/// failure (not "no ticket", but an actual error) silently became "no ticket"
/// in those last two, hiding exactly the kind of problem this robustness should
/// make visible — only the constructor logged it before.
#[cfg(target_os = "android")]
fn load_iroh_services_ticket_logged(storage: &SecureP2pStorage) -> Option<String> {
    storage.load_iroh_services_ticket().unwrap_or_else(|error| {
        tracing::warn!(layer = "relay", error = %error, "failed to load Iroh Services ticket");
        None
    })
}

#[cfg(target_os = "android")]
impl P2pNodeContext {
    async fn build(&self, relay_mode: RelayModeConfig) -> Arc<AcerolaP2p> {
        let transport = IrohTransportBuilder::default()
            .relay_mode(relay_mode)
            .blobs(IrohBlobsConfig::mem());

        let device =
            DefaultDeviceInfoProvider::new(self.device_name.clone(), self.device_version.clone())
                .provide()
                .expect("Failed to read device info");

        let acerola_node = AcerolaP2p::builder(Arc::clone(&self.emit), transport, device)
            .guard(
                TofuGuard::new(Arc::clone(&self.trust_store)
                    as Arc<dyn acerola_p2p::api::guard::TrustedPeerStore>)
                .into_validator(),
            )
            .storage(SharedSecureP2pStorage(Arc::clone(&self.storage)))
            .inbound(
                HISTORY_SYNC_ALPN,
                Arc::new(HistorySyncInbound::new(
                    Arc::clone(&self.emit),
                    Arc::clone(&self.history_provider),
                )),
            )
            .outbound(
                HISTORY_SYNC_ALPN,
                Arc::new(HistorySyncOutbound::new(
                    Arc::clone(&self.emit),
                    Arc::clone(&self.history_provider),
                )),
            )
            .inbound(
                FILE_SYNC_ALPN,
                Arc::new(FileSyncInbound::new(
                    Arc::clone(&self.emit),
                    Arc::clone(&self.file_provider),
                    Arc::clone(&self.file_sync_guard),
                    Arc::clone(&self.chapter_transfer),
                )),
            )
            .outbound(
                FILE_SYNC_ALPN,
                Arc::new(FileSyncOutbound::new(
                    Arc::clone(&self.emit),
                    Arc::clone(&self.file_provider),
                    Arc::clone(&self.file_sync_guard),
                    Arc::clone(&self.chapter_transfer),
                )),
            )
            // `acerola/sync-comic/1` reuses the SAME `file_sync_guard` as the
            // regular `sync-files` (see `protocol::files::run_and_report_scoped`) —
            // the two ALPNs never run simultaneous sessions for the same peer.
            .inbound(
                COMIC_SYNC_ALPN,
                Arc::new(ComicSyncInbound::new(
                    Arc::clone(&self.emit),
                    Arc::clone(&self.file_provider),
                    Arc::clone(&self.file_sync_guard),
                    Arc::clone(&self.chapter_transfer),
                )),
            )
            .outbound(
                COMIC_SYNC_ALPN,
                Arc::new(ComicSyncOutbound::new(
                    Arc::clone(&self.emit),
                    Arc::clone(&self.file_provider),
                    Arc::clone(&self.file_sync_guard),
                    Arc::clone(&self.chapter_transfer),
                    Arc::clone(&self.pending_comic_scope),
                )),
            )
            .inbound(
                HISTORY_ENTRY_SYNC_ALPN,
                Arc::new(HistoryEntrySyncInbound::new(
                    Arc::clone(&self.emit),
                    Arc::clone(&self.history_provider),
                )),
            )
            .outbound(
                HISTORY_ENTRY_SYNC_ALPN,
                Arc::new(HistoryEntrySyncOutbound::new(
                    Arc::clone(&self.emit),
                    Arc::clone(&self.history_provider),
                    Arc::clone(&self.pending_history_entry_scope),
                )),
            )
            .inbound(
                LIBRARY_BROWSE_ALPN,
                Arc::new(LibraryBrowseInbound::new(Arc::clone(&self.file_provider))),
            )
            .outbound(
                LIBRARY_BROWSE_ALPN,
                Arc::new(LibraryBrowseOutbound::new(Arc::clone(&self.emit))),
            )
            .inbound(
                COVER_BROWSE_ALPN,
                Arc::new(CoverBrowseInbound::new(
                    Arc::clone(&self.cover_provider),
                    Arc::clone(&self.chapter_transfer),
                )),
            )
            .outbound(
                COVER_BROWSE_ALPN,
                Arc::new(CoverBrowseOutbound::new(
                    Arc::clone(&self.emit),
                    Arc::clone(&self.cover_provider),
                    Arc::clone(&self.chapter_transfer),
                    Arc::clone(&self.pending_cover_scope),
                )),
            )
            .build()
            .await
            .expect("Failed to start P2P node");

        let node = Arc::new(acerola_node);
        self.blob_context.set_node(&node);
        node
    }
}

#[derive(uniffi::Object)]
pub struct P2PNode {
    /// `RwLock` (not `Mutex`/plain field) because `restart()` needs to SWAP the live
    /// `Arc<AcerolaP2p>` for a new one without invalidating the UniFFI handle Kotlin already holds —
    /// the `P2PNode` object itself is never rebuilt on the Kotlin side, only the node underneath.
    /// `std::sync::RwLock`, not `tokio::sync::RwLock`: the methods that read this are synchronous
    /// (called straight from the FFI boundary), so a `tokio::RwLock` would require `blocking_read()` —
    /// a guaranteed panic inside `TOKIO_RUNTIME` (see `blocking_read` doc). Same choice as Desktop
    /// (`NetworkService.node`, `core/services/network/mod.rs`).
    node: RwLock<Arc<AcerolaP2p>>,
    runtime: Arc<Runtime>,
    /// Ingredients reused by `restart()` to rebuild the node from scratch — see
    /// `P2pNodeContext`.
    #[cfg(target_os = "android")]
    context: Arc<P2pNodeContext>,
    #[cfg(target_os = "android")]
    trust_store: Arc<SecureTrustedStore>,
    #[cfg(target_os = "android")]
    storage: Arc<SecureP2pStorage>,
    /// See `protocol::files::PendingComicScope` — pending state between the `sync_comic` FFI
    /// call and the outbound `Handler` of `acerola/sync-comic/1` it triggers.
    #[cfg(target_os = "android")]
    pending_comic_scope: PendingComicScope,
    /// See `protocol::history::PendingHistoryEntryScope` — same idea as `pending_comic_scope`,
    /// for the individual push of a history entry (`acerola/sync-history-entry/1`).
    #[cfg(target_os = "android")]
    pending_history_entry_scope: PendingHistoryEntryScope,
    /// See `protocol::cover_browse::PendingCoverRequestRegistry` — same idea as
    /// `pending_comic_scope`, but a per-peer FIFO queue instead of a single slot (the only one fired in
    /// bursts — `SyncViewModel::fetchCoversFor` calls `browse_cover` once per comic of the
    /// remote library, in quick succession, for the same peer).
    #[cfg(target_os = "android")]
    pending_cover_scope: Arc<PendingCoverRequestRegistry>,
}

// Outside the `#[uniffi::export]`-annotated `impl` on purpose: the macro scans EVERY method
// inside an exported block (public or not) and tries to generate bindings for it — `Arc<AcerolaP2p>`
// has no `FfiConverter` (it should never cross the FFI boundary raw), so a private `fn node(&self)`
// inside the exported block breaks the macro at compile time.
#[cfg(target_os = "android")]
impl P2PNode {
    fn node(&self) -> Arc<AcerolaP2p> {
        Arc::clone(
            &self
                .node
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner()),
        )
    }
}

#[uniffi::export]
#[cfg(target_os = "android")]
impl P2PNode {
    #[uniffi::constructor]
    pub fn new(
        callback: Arc<dyn P2PCallback>,
        legacy_data_dir: Option<String>,
        blobs_dir: String,
        relay_settings: FfiRelaySettings,
        device_name: String,
        device_version: String,
        secure_store: Arc<dyn SecureBlobStore>,
        history_provider: Arc<dyn HistorySyncProvider>,
        file_provider: Arc<dyn FileSyncProvider>,
        cover_provider: Arc<dyn CoverBrowseProvider>,
    ) -> Self {
        let runtime = TOKIO_RUNTIME.clone();

        let cb_clone = Arc::clone(&callback);
        let emit: acerola_p2p::api::protocol::EventEmitter = Arc::new(move |event, data| {
            cb_clone.on_event(event.to_string(), data);
        });

        // Only used to migrate `identity.seed`/`peers.json`/`trusted.txt`/`blocked.txt` (old
        // plain-text format, from before `SecureBlobStore`) — see `storage.rs`/`trust_store.rs`.
        let legacy_dir = legacy_data_dir.as_deref().map(std::path::Path::new);

        let trust_store = Arc::new(SecureTrustedStore::open(
            Arc::clone(&secure_store),
            Arc::clone(&emit),
            legacy_dir,
        ));

        let storage = Arc::new(SecureP2pStorage::open(
            Arc::clone(&secure_store),
            legacy_dir,
        ));

        // Ticket of the user's account on `services.iroh.computer` (pasted by them on the network
        // settings screen) — comes from the SAME vault as the identity, not from DataStore. A real
        // backend failure should not block initializing the rest of the P2P network just
        // because of Iroh's public network, so it also becomes "no ticket" here, with a log.
        let iroh_services_ticket = load_iroh_services_ticket_logged(&storage);

        let pending_comic_scope: PendingComicScope = Arc::new(Mutex::new(HashMap::new()));
        let pending_history_entry_scope: PendingHistoryEntryScope =
            Arc::new(Mutex::new(HashMap::new()));
        let pending_cover_scope = PendingCoverRequestRegistry::new();

        // `sync-files`/`sync-comic` handlers are registered in the builder BEFORE the node
        // exists, but they need `node.blobs()`/`node.known_peers()` to publish/fetch
        // blobs — `BlobContext` holds a `Weak<AcerolaP2p>` filled only after each
        // `.build()` (see `protocol::blob_context`).
        let blob_context = crate::protocol::blob_context::BlobContext::new();
        let chapter_transfer: Arc<dyn ChapterTransfer> =
            Arc::new(BlobChapterTransfer::new(Arc::clone(&blob_context)));
        let file_sync_guard = Arc::new(FileSyncSessionGuard::default());

        // TEMPORARY MITIGATION: `IrohBlobsConfig::fs(blobs_dir)` hangs inside
        // `FsStore::load_with_opts` (iroh-blobs) — never tested with an on-disk store in the lib, only
        // in memory. It blocked `runtime.block_on` on Android's MAIN THREAD (ANR). `.mem()`
        // does not persist blobs across restarts, but unblocks the app immediately while the FsStore
        // hang is isolated/fixed. `blobs_dir` stays unused until this mitigation is removed.
        let _ = &blobs_dir;

        let context = Arc::new(P2pNodeContext {
            emit: Arc::clone(&emit),
            trust_store: Arc::clone(&trust_store),
            storage: Arc::clone(&storage),
            history_provider,
            file_provider,
            cover_provider,
            file_sync_guard,
            pending_comic_scope: Arc::clone(&pending_comic_scope),
            pending_history_entry_scope: Arc::clone(&pending_history_entry_scope),
            pending_cover_scope: Arc::clone(&pending_cover_scope),
            chapter_transfer,
            blob_context,
            device_name,
            device_version,
        });

        // Log redacted on purpose: `IrohDefault` carries the user's account ticket (a
        // real credential) as variant data — it must never go to the enum's raw Debug. Only the
        // resolved source name matters here for diagnostics (parity with Desktop's
        // `[Bios::Network] Using relay mode`).
        let resolved_relay_mode = relay_settings.resolve(iroh_services_ticket.as_deref());
        let relay_mode_label = match &resolved_relay_mode {
            RelayModeConfig::MdnsOnly => "mdns_only",
            RelayModeConfig::Custom(_) => "custom",
            RelayModeConfig::AcerolaOwn => "acerola_own",
            RelayModeConfig::IrohDefault(_) => "iroh_default",
        };
        tracing::info!(
            layer = "relay",
            mode = relay_mode_label,
            "resolved relay mode for P2P transport"
        );

        let node = {
            let context = Arc::clone(&context);
            runtime.block_on(async move { context.build(resolved_relay_mode).await })
        };

        Self {
            node: RwLock::new(node),
            runtime,
            context,
            trust_store,
            storage,
            pending_comic_scope,
            pending_history_entry_scope,
            pending_cover_scope,
        }
    }

    /// Fully restarts the node: shuts down the current instance and brings up a new one with the same
    /// identity/storage/handlers (reused via `context`), resolving a fresh `relay_settings`
    /// — Kotlin re-reads the combinable config from DataStore (`RelayPreference`) before
    /// calling, same pattern as `apply_relay_settings`. LocalSend style: called both after
    /// any relay source change and by the manual "Restart" button on the Network screen.
    /// Mirrors Desktop's `NetworkServiceApi::restart` (`core/services/network/mod.rs`). Same
    /// boot risk: if the rebuild fails (`P2pNodeContext::build` calls `.expect(...)`), the call
    /// panics instead of returning a handleable error — same as `P2PNode::new` today.
    pub fn restart(&self, relay_settings: FfiRelaySettings) {
        let old_node = self.node();
        self.runtime.block_on(async move {
            if let Err(error) = old_node.shutdown().await {
                tracing::warn!(?error, "failed to shut down old p2p node before restart");
            }
        });

        let iroh_services_ticket = load_iroh_services_ticket_logged(&self.storage);
        let resolved_relay_mode = relay_settings.resolve(iroh_services_ticket.as_deref());

        let context = Arc::clone(&self.context);
        let storage = Arc::clone(&self.storage);
        let fresh_node = self.runtime.block_on(async move {
            let fresh_node = context.build(resolved_relay_mode).await;

            // Without this, a restart returned the node in a clean state — identity and storage
            // preserved, but ZERO active connections — and nothing tried to talk again to any
            // paired peer until the user triggered some manual action (browse library, sync,
            // etc). Same bug already fixed on Desktop (`NetworkService::restart`,
            // `core/services/network/mod.rs`); `reconnect_known_peers` only QUEUES the attempt
            // (does not wait for confirmation) and is best-effort: a failure loading `paired_peers`
            // must not make the restart itself (already completed successfully above) look like it
            // failed.
            match storage.load_peers().await {
                Ok(known_peers) => {
                    acerola_p2p::api::network::reconnect_known_peers(&fresh_node, known_peers)
                        .await;
                },
                Err(error) => {
                    tracing::warn!(
                        ?error,
                        "failed to load paired peers for post-restart reconnect"
                    );
                },
            }

            fresh_node
        });

        *self
            .node
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = fresh_node;

        tracing::info!("[P2PNode] restarted");
    }

    pub fn get_local_id(&self) -> String {
        self.node().local_id().to_string()
    }

    pub fn get_local_addr(&self) -> FfiPeerAddr {
        // Recomputed live by the transport on every call (see `AcerolaP2p::local_addr`
        // docs) — only fails if serializing the `EndpointAddr` breaks,
        // which does not happen in practice (no custom fields, it is all standard iroh serde).
        let addr = self
            .node()
            .local_addr()
            .expect("local_addr should always serialize");
        FfiPeerAddr {
            id: addr.id.id.clone(),
            device_id: addr.id.device_id.clone(),
            addrs: addr.addrs.clone(),
        }
    }

    /// Forwards to the transport a notice that the OS network may have changed (Wi-Fi/mobile data
    /// switch, leaving Doze, etc). Exists because Android does not expose network interface
    /// monitoring to native code — only to Java/Kotlin via `ConnectivityManager.NetworkCallback`
    /// — so without this bridge the QUIC transport never learns the network path changed and
    /// keeps reusing silently dead connections until the app is restarted. Kotlin
    /// must call this on every `onAvailable`/`onLost`/`onCapabilitiesChanged` of the
    /// `NetworkCallback` (see `P2pService`).
    pub fn notify_network_change(&self) {
        let node = self.node();
        self.runtime.spawn(async move {
            node.network_change().await;
        });
    }

    /// Overrides this device's display name (LocalSend-style custom nickname, instead of the
    /// default `Build.MODEL`) — applies from the next handshake on, without restarting the
    /// app, since `AcerolaP2p::set_local_device_name` writes to the same shared `DeviceInfo`
    /// read by the handshake handlers on every identity exchange (see
    /// `lib/p2p/src/api/acerola_p2p.rs`). Persistence across restarts is the responsibility of the
    /// Kotlin side (DataStore), which re-reads the saved nickname to pass as `device_name` to the
    /// next `P2PNode::new`.
    pub fn set_local_device_name(&self, name: String) {
        let node = self.node();
        self.runtime.spawn(async move {
            node.set_local_device_name(name).await;
        });
    }

    pub fn connect(&self, peer_addr: FfiPeerAddr, alpn: Vec<u8>) {
        let node = self.node();
        let addr = PeerAddr {
            id: PeerIdentity {
                id: peer_addr.id,
                device_id: peer_addr.device_id,
            },
            addrs: peer_addr.addrs,
        };
        self.runtime.spawn(async move {
            let _ = node.connect(addr, &alpn).await;
        });
    }

    /// Syncs a single comic (`comic_name`) with `peer_addr`, in the explicit `direction`
    /// chosen by the user (`Push` = send to the peer; `Pull` = pull from it) — see
    /// `protocol::files::model::SyncDirection`. Stores `(comic_name, direction, chapters)` in the
    /// pending registry BEFORE connecting — the only way for this choice (which only exists here,
    /// on the calling side) to reach `ComicSyncOutbound::handle`, since `connect()` carries no
    /// payload (see `protocol::files::COMIC_SYNC_ALPN`).
    ///
    /// `chapters` (chapter labels, same key as `FileChapterInfo.chapter` — empty =
    /// whole comic) scopes the session to a subset of chapters, used by the
    /// "Send" button of the multi-selection/three-dot menu: Kotlin resolves the selected
    /// `chapterSort`s to the matching labels before calling this method
    /// (see `ComicViewModel`), since the manifest exchanged on the wire uses labels, not `chapter_sort`.
    pub fn sync_comic(
        &self,
        peer_addr: FfiPeerAddr,
        comic_name: String,
        direction: FfiSyncDirection,
        chapters: Vec<String>,
    ) {
        self.pending_comic_scope
            .lock()
            .expect("pending comic scope mutex poisoned")
            .insert(peer_addr.id.clone(), (comic_name, direction.into(), chapters));
        self.connect(peer_addr, COMIC_SYNC_ALPN.to_vec());
    }

    /// Pushes the progress + "read" markers of the selected chapter(s)
    /// (`chapter_sorts`) of ONE single comic (`comic_name`) to `peer_addr` — lighter than
    /// a full `acerola/sync-history/1` session (whole library in both directions).
    /// Same technique as `sync_comic`: stores the pending scope BEFORE connecting.
    pub fn sync_history_entry(
        &self,
        peer_addr: FfiPeerAddr,
        comic_name: String,
        chapter_sorts: Vec<String>,
    ) {
        self.pending_history_entry_scope
            .lock()
            .expect("pending history entry scope mutex poisoned")
            .insert(peer_addr.id.clone(), (comic_name, chapter_sorts));
        self.connect(peer_addr, HISTORY_ENTRY_SYNC_ALPN.to_vec());
    }

    /// Requests the list of comics (name + chapter count) from `peer_addr`'s library,
    /// without syncing anything — fire-and-forget like `connect()`; the result arrives later via
    /// `browse:library:result`/`browse:library:error` (see `protocol::library_browse`).
    pub fn browse_library(&self, peer_addr: FfiPeerAddr) {
        self.connect(peer_addr, LIBRARY_BROWSE_ALPN.to_vec());
    }

    /// Fetches the cover (thumbnail) of `comic_name` from `peer_addr`'s library — `known_version`
    /// is the version already cached locally (`(peer_id, comic_name, cover_version)`), `None` if
    /// this cover was never fetched before. Same fire-and-forget pattern as `sync_comic`: stores the
    /// pending scope before connecting, the result arrives via
    /// `browse:cover:result`/`browse:cover:error` (see `protocol::cover_browse`).
    pub fn browse_cover(
        &self,
        peer_addr: FfiPeerAddr,
        comic_name: String,
        known_version: Option<i64>,
    ) {
        self.pending_cover_scope
            .push(peer_addr.id.clone(), comic_name, known_version);
        self.connect(peer_addr, COVER_BROWSE_ALPN.to_vec());
    }

    pub fn shutdown(&self) {
        let node = self.node();
        self.runtime.block_on(async move {
            let _ = node.shutdown().await;
        });
    }

    pub fn switch_to_local(&self) {
        let node = self.node();
        let trust_store = Arc::clone(&self.trust_store);
        self.runtime.spawn(async move {
            let guard =
                TofuGuard::new(trust_store as Arc<dyn acerola_p2p::api::guard::TrustedPeerStore>)
                    .into_validator();
            let _ = node.switch_guard(guard, NetworkMode::Local).await;
        });
    }

    pub fn switch_to_relay(&self) {
        let node = self.node();
        let trust_store = Arc::clone(&self.trust_store);
        self.runtime.spawn(async move {
            let guard =
                TofuGuard::new(trust_store as Arc<dyn acerola_p2p::api::guard::TrustedPeerStore>)
                    .into_validator();
            let _ = node.switch_guard(guard, NetworkMode::Relay).await;
        });
    }

    pub fn get_mode(&self) -> FfiNetworkMode {
        let node = self.node();
        self.runtime
            .block_on(async move { node.mode().await.into() })
    }

    pub fn get_connected_peers(&self) -> HashMap<String, Vec<Vec<u8>>> {
        let node = self.node();
        self.runtime.block_on(async move {
            node.connected_peers()
                .await
                .into_iter()
                .map(|(peer, alpns)| (peer.id.clone(), alpns.into_iter().collect()))
                .collect()
        })
    }

    /// Returns every peer ever paired (TOFU), with the last known
    /// address — they persist across app restarts and do not depend on a currently active connection,
    /// since the handshake connection itself lasts only a few seconds (exchanges PING/PONG/DeviceInfo
    /// and closes). It is this list, not `get_connected_peers*`, that should feed "paired
    /// devices" in the UI.
    pub fn get_paired_peers(&self) -> Vec<FfiPairedPeer> {
        let storage = Arc::clone(&self.storage);
        let node = self.node();
        self.runtime.block_on(async move {
            let device_names: HashMap<String, String> = node
                .known_peers()
                .await
                .into_iter()
                .filter_map(|(peer, _, info)| info.map(|device| (peer.id.clone(), device.name)))
                .collect();

            storage
                .load_peers()
                .await
                .unwrap_or_default()
                .into_iter()
                .map(|addr| FfiPairedPeer {
                    device_name: device_names.get(&addr.id.id).cloned(),
                    id: addr.id.id,
                    device_id: addr.id.device_id,
                    addrs: addr.addrs,
                })
                .collect()
        })
    }

    /// Unpairs a peer: removes it from trust (`trust_store`) and from the known address
    /// cache (`storage`, the same source as `get_paired_peers`). Does not drop an active
    /// connection nor block the peer — if it tries to connect again later, it goes through the same
    /// TOFU flow as a never-seen device (see `trust_store::SecureTrustedStore::remove`).
    pub fn remove_paired_peer(&self, id: String) {
        let trust_store = Arc::clone(&self.trust_store);
        let storage = Arc::clone(&self.storage);
        self.runtime.block_on(async move {
            trust_store.remove(&id).await;
            let _ = storage.remove_peer(&id).await;
        });
    }

    pub fn get_connected_peers_with_info(&self) -> Vec<FfiConnectedPeer> {
        let node = self.node();
        self.runtime.block_on(async move {
            node.connected_peers_with_info()
                .await
                .into_iter()
                .map(|(peer, alpns, device)| FfiConnectedPeer {
                    peer_id: peer.id.clone(),
                    alpns: alpns.into_iter().collect(),
                    device_name: device.map(|info| info.name),
                })
                .collect()
        })
    }

    /// `true` if the user already pasted and saved a ticket of their own account on
    /// `services.iroh.computer` — never returns the value itself (it is a real credential).
    pub fn has_iroh_services_ticket(&self) -> bool {
        self.storage
            .load_iroh_services_ticket()
            .ok()
            .flatten()
            .is_some()
    }

    /// Validates the format before persisting. Only takes effect on the next app start — the lib
    /// does not support switching relay at runtime.
    pub fn set_iroh_services_ticket(&self, ticket: String) -> Result<(), RelayTicketError> {
        let trimmed = ticket.trim();
        acerola_p2p::api::transport::validate_iroh_services_ticket(trimmed).map_err(|error| {
            RelayTicketError::Invalid {
                reason: error.to_string(),
            }
        })?;

        self.storage
            .save_iroh_services_ticket(trimmed)
            .map_err(|error| RelayTicketError::Invalid {
                reason: error.to_string(),
            })
    }

    /// Removes the saved ticket — used when the user turns the source off or replaces it with a new one.
    pub fn clear_iroh_services_ticket(&self) {
        let _ = self.storage.clear_iroh_services_ticket();
    }

    /// Re-reads the combinable relay config (coming from Kotlin, `RelayPreference`) + the vault
    /// ticket, resolves and applies it to the ALREADY LIVE node (`AcerolaP2p::apply_relay_mode`) — without
    /// restarting the app. Called by Kotlin after ANY change in the relay sources
    /// (Acerola/Iroh toggle, add/remove of a custom URL, save/remove ticket). Mirrors
    /// Desktop's `NetworkServiceApi::apply_relay_settings`.
    pub fn apply_relay_settings(
        &self,
        relay_settings: FfiRelaySettings,
    ) -> Result<(), RelayTicketError> {
        let iroh_services_ticket = load_iroh_services_ticket_logged(&self.storage);
        let config = relay_settings.resolve(iroh_services_ticket.as_deref());
        let node = self.node();

        self.runtime
            .block_on(async move { node.apply_relay_mode(config).await })
            .map_err(|error| RelayTicketError::Invalid {
                reason: error.to_string(),
            })
    }
}
