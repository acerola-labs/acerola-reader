use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use acerola_p2p::api::{
    blobs::IrohBlobsConfig,
    guard::{TofuGuard, TrustedPeerStore},
    identity::{DefaultDeviceInfoProvider, DeviceInfoProvider},
    storage::P2PStorage,
    transport::IrohTransportBuilder,
    AcerolaP2p,
};
use tauri::{Emitter, Manager};

use crate::{
    bios::scopes::{read_device_alias_override, read_library_path, read_relay_settings},
    cmd::features::metadata::MetadataState,
    core::services::{
        network::{NetworkService, NetworkServiceApi},
        sync::{file_sync::FileSyncService, history_sync::HistorySyncService},
    },
    data::repositories::sync::SyncHistoryLogRepository,
    infra::{
        error::ComicError,
        security::{
            get_or_create_master_key,
            p2p_storage::{SecureP2pStorage, SharedP2pStorage},
            trusted_store::SecureTrustedStore,
            MasterKeySource,
        },
        sync::protocol::{
            comic_handler::{ComicSyncInbound, ComicSyncOutbound},
            comic_sync_registry::PendingComicSyncRegistry,
            cover_browse_handler::{CoverBrowseInbound, CoverBrowseOutbound},
            cover_request_registry::PendingCoverRequestRegistry,
            file_handler::{FileSyncInbound, FileSyncOutbound},
            file_session_guard::FileSyncSessionGuard,
            history_entry_handler::{HistoryEntrySyncInbound, HistoryEntrySyncOutbound},
            history_entry_registry::PendingHistoryEntryRegistry,
            history_handler::{HistorySyncInbound, HistorySyncOutbound},
            library_browse_handler::{LibraryBrowseInbound, LibraryBrowseOutbound},
            transfer::{BlobChapterTransfer, ChapterTransfer},
            COMIC_SYNC_ALPN, COVER_BROWSE_ALPN, FILE_SYNC_ALPN, HISTORY_ENTRY_SYNC_ALPN,
            HISTORY_SYNC_ALPN, LIBRARY_BROWSE_ALPN,
        },
    },
};

/// Name of the old plaintext seed file — only read once to migrate into
/// `identity.enc` (see [`migrate_plaintext_seed_if_present`]); it stops existing after that.
const LEGACY_PLAINTEXT_SEED_FILE_NAME: &str = "p2p-seed.key";

/// If `p2p-seed.key` (old plaintext format) still exists and `identity.enc` (new encrypted
/// format) does not exist yet, migrates the seed into secure storage and deletes the old
/// file. Without this, anyone who had already paired devices would lose their current P2P
/// identity on the first update and have to re-pair everything from scratch.
async fn migrate_plaintext_seed_if_present(
    app_data_directory: &Path, secure_storage: &SecureP2pStorage,
) -> Result<(), ComicError> {
    let legacy_path = app_data_directory.join(LEGACY_PLAINTEXT_SEED_FILE_NAME);
    if !legacy_path.exists() {
        return Ok(());
    }

    if secure_storage.load_identity().await.ok().flatten().is_some() {
        // Already migrated in a previous run (e.g. deleting the old file failed) — just clean up.
        std::fs::remove_file(&legacy_path).ok();
        return Ok(());
    }

    let legacy_seed = std::fs::read(&legacy_path).map_err(ComicError::Io)?;
    if legacy_seed.len() != 32 {
        tracing::warn!(
            "[Bios::Network] Legacy p2p-seed.key has unexpected length ({} bytes), ignoring",
            legacy_seed.len()
        );
        return Ok(());
    }

    secure_storage.save_identity(&legacy_seed).await.map_err(|err| {
        ComicError::SystemFailure(format!("failed to migrate legacy p2p seed: {err}"))
    })?;
    std::fs::remove_file(&legacy_path).ok();
    tracing::info!(
        "[Bios::Network] Migrated legacy plaintext p2p-seed.key into encrypted identity.enc"
    );

    Ok(())
}

/// Every ingredient needed to build (or rebuild) the `AcerolaP2p` — extracted from the body of
/// `setup_network` so it can be reused between the initial boot and a `NetworkService::restart()`
/// (relay change, or the manual "Restart" button on the Network screen). Every field is cheap to
/// clone (`Arc` or `PathBuf`/types that were already `.clone()`d in the original flow) on purpose:
/// [`Self::build`] runs again on every restart, so nothing here can be "consumed" only once.
///
/// `relay_mode`/`device_information` are deliberately NOT fields — they are re-read from disk
/// (`settings.json` + vault) on every [`Self::build`] call, not captured once at boot.
/// This recomputation is what makes a relay/alias change actually take effect on a restart,
/// instead of requiring the whole app to be closed and reopened.
struct P2pNodeContext {
    app_data_directory: PathBuf,
    event_emitter: acerola_p2p::api::protocol::EventEmitter,
    trusted_store: Arc<SecureTrustedStore>,
    secure_p2p_storage: Arc<SecureP2pStorage>,
    history_sync_service: HistorySyncService,
    file_sync_service: FileSyncService,
    metadata_service: Arc<crate::core::services::metadata::MetadataService>,
    sync_log_repo: SyncHistoryLogRepository,
    file_sync_session_guard: Arc<FileSyncSessionGuard>,
    pending_comic_sync: Arc<PendingComicSyncRegistry>,
    pending_history_entry: Arc<PendingHistoryEntryRegistry>,
    pending_cover_request: Arc<PendingCoverRequestRegistry>,
    chapter_transfer: Arc<dyn ChapterTransfer>,
    remote_covers_dir: PathBuf,
    blob_context: Arc<crate::infra::sync::blob_context::BlobContext>,
}

impl P2pNodeContext {
    /// Builds an `AcerolaP2p` from scratch: re-reads relay/alias from disk, registers the same
    /// handlers as always (the instances themselves are reused via `Arc`/`.clone()`, only the
    /// underlying node is new) and re-points the shared `BlobContext` to the newly created node
    /// before returning — without this, the sync-files/sync-comic handlers (which exist since
    /// the first boot and are never recreated) would stay stuck pointing to the OLD node.
    async fn build(&self) -> Result<Arc<AcerolaP2p>, ComicError> {
        let iroh_services_ticket =
            self.secure_p2p_storage.load_iroh_services_ticket().await.unwrap_or_else(|error| {
                tracing::warn!("[Bios::Network] Failed to load Iroh Services ticket: {}", error);
                None
            });
        let relay_mode =
            read_relay_settings(&self.app_data_directory).resolve(iroh_services_ticket.as_deref());
        tracing::info!("[Bios::Network] Using relay mode: {:?}", relay_mode);

        // TEMPORARY MITIGATION: `IrohBlobsConfig::fs(...)` hangs inside
        // `FsStore::load_with_opts` (iroh-blobs) — see the historical comment in
        // `acerola-p2p/src/core/blobs/iroh/mod.rs`. `.mem()` does not persist blobs across
        // module (or app) restarts, but unblocks things while the FsStore hang is not
        // fixed.
        let transport_builder =
            IrohTransportBuilder::default().relay_mode(relay_mode).blobs(IrohBlobsConfig::mem());

        // No `.seed(...)` — the builder resolves the identity on its own from the `.storage(...)`
        // below (it loads the seed saved in `identity.enc`), so the identity survives a module
        // restart normally, without repeating pairing.
        let mut device_information =
            DefaultDeviceInfoProvider::new("0.0.1-beta").provide().map_err(|device_error| {
                ComicError::SystemFailure(format!("Failed to read device info: {:?}", device_error))
            })?;
        if let Some(alias) = read_device_alias_override(&self.app_data_directory) {
            device_information.name = alias;
        }

        let node = match tokio::time::timeout(
            std::time::Duration::from_secs(10),
            AcerolaP2p::builder(
                Arc::clone(&self.event_emitter),
                transport_builder,
                device_information,
            )
            .guard(
                TofuGuard::new(Arc::clone(&self.trusted_store) as Arc<dyn TrustedPeerStore>)
                    .into_validator(),
            )
            .storage(SharedP2pStorage(Arc::clone(&self.secure_p2p_storage)))
            .inbound(
                HISTORY_SYNC_ALPN,
                Arc::new(HistorySyncInbound::new(
                    Arc::clone(&self.event_emitter),
                    self.history_sync_service.clone(),
                    self.sync_log_repo.clone(),
                )),
            )
            .outbound(
                HISTORY_SYNC_ALPN,
                Arc::new(HistorySyncOutbound::new(
                    Arc::clone(&self.event_emitter),
                    self.history_sync_service.clone(),
                    self.sync_log_repo.clone(),
                )),
            )
            .inbound(
                FILE_SYNC_ALPN,
                Arc::new(FileSyncInbound::new(
                    Arc::clone(&self.event_emitter),
                    self.file_sync_service.clone(),
                    Arc::clone(&self.metadata_service),
                    self.sync_log_repo.clone(),
                    Arc::clone(&self.file_sync_session_guard),
                    Arc::clone(&self.chapter_transfer),
                )),
            )
            .outbound(
                FILE_SYNC_ALPN,
                Arc::new(FileSyncOutbound::new(
                    Arc::clone(&self.event_emitter),
                    self.file_sync_service.clone(),
                    Arc::clone(&self.metadata_service),
                    self.sync_log_repo.clone(),
                    Arc::clone(&self.file_sync_session_guard),
                    Arc::clone(&self.chapter_transfer),
                )),
            )
            .inbound(
                COMIC_SYNC_ALPN,
                Arc::new(ComicSyncInbound::new(
                    Arc::clone(&self.event_emitter),
                    self.file_sync_service.clone(),
                    Arc::clone(&self.metadata_service),
                    self.sync_log_repo.clone(),
                    Arc::clone(&self.file_sync_session_guard),
                    Arc::clone(&self.chapter_transfer),
                )),
            )
            .outbound(
                COMIC_SYNC_ALPN,
                Arc::new(ComicSyncOutbound::new(
                    Arc::clone(&self.event_emitter),
                    self.file_sync_service.clone(),
                    Arc::clone(&self.metadata_service),
                    self.sync_log_repo.clone(),
                    Arc::clone(&self.file_sync_session_guard),
                    Arc::clone(&self.pending_comic_sync),
                    Arc::clone(&self.chapter_transfer),
                )),
            )
            .inbound(
                HISTORY_ENTRY_SYNC_ALPN,
                Arc::new(HistoryEntrySyncInbound::new(
                    Arc::clone(&self.event_emitter),
                    self.history_sync_service.clone(),
                )),
            )
            .outbound(
                HISTORY_ENTRY_SYNC_ALPN,
                Arc::new(HistoryEntrySyncOutbound::new(
                    Arc::clone(&self.event_emitter),
                    self.history_sync_service.clone(),
                    Arc::clone(&self.pending_history_entry),
                )),
            )
            .inbound(
                LIBRARY_BROWSE_ALPN,
                Arc::new(LibraryBrowseInbound::new(self.file_sync_service.clone())),
            )
            .outbound(
                LIBRARY_BROWSE_ALPN,
                Arc::new(LibraryBrowseOutbound::new(Arc::clone(&self.event_emitter))),
            )
            .inbound(
                COVER_BROWSE_ALPN,
                Arc::new(CoverBrowseInbound::new(
                    self.file_sync_service.clone(),
                    Arc::clone(&self.chapter_transfer),
                )),
            )
            .outbound(
                COVER_BROWSE_ALPN,
                Arc::new(CoverBrowseOutbound::new(
                    Arc::clone(&self.event_emitter),
                    Arc::clone(&self.chapter_transfer),
                    Arc::clone(&self.pending_cover_request),
                    self.remote_covers_dir.clone(),
                )),
            )
            .build(),
        )
        .await
        {
            Ok(Ok(node_instance)) => node_instance,
            Ok(Err(start_error)) => {
                tracing::error!("[Bios::Network] Failed to start P2P node: {:?}", start_error);
                return Err(ComicError::SystemFailure(format!(
                    "Failed to start p2p node: {:?}",
                    start_error
                )));
            },
            Err(timeout_error) => {
                tracing::error!(
                    "[Bios::Network] Timeout waiting for AcerolaP2p::build(): {:?}",
                    timeout_error
                );
                return Err(ComicError::SystemFailure(
                    "TIMEOUT waiting for AcerolaP2p::build()!".to_string(),
                ));
            },
        };

        let node = Arc::new(node);
        self.blob_context.set_node(&node);
        Ok(node)
    }
}

pub async fn setup_network_services(
    app_handle: &tauri::AppHandle,
) -> Result<InitializedNetworkServices, ComicError> {
    let app_handle_clone = app_handle.clone();

    let event_emitter: acerola_p2p::api::protocol::EventEmitter =
        Arc::new(move |event_name, event_data| {
            app_handle_clone.emit(event_name, event_data).ok();
        });

    let app_data_directory =
        app_handle.path().app_data_dir().unwrap_or_else(|_| PathBuf::from("."));

    let (master_key, master_key_source) = get_or_create_master_key(&app_data_directory)?;
    // Kept as managed state (not just emitted as an event) because `setup_network` runs in a
    // separate async task (`bios/mod.rs::setup_runtime`) that may finish before or after the
    // frontend mounts the Network screen — a plain event would be lost if emitted before any
    // listener exists. `get_security_status` (command) queries this on demand; the event below
    // covers the case where the frontend is already listening.
    app_handle.manage(master_key_source);
    if master_key_source == MasterKeySource::FallbackFile {
        // No real keyring available (e.g. Linux/Hyprland without gnome-keyring/kwallet
        // running) — never fail silently, same principle as VS Code: warn the user that the
        // master key fell back to disk without the extra OS protection.
        app_handle.emit("security:keyring_unavailable", ()).ok();
    }

    // Registered BEFORE opening trust/peer storage on purpose: `get_sync_history_log` is a
    // feature independent of P2P identity/trust (it is just the local log of past sync
    // sessions) — if it came later, a real decrypt failure in `trusted.enc`/`peers.enc`
    // (`?` below) would take this command down too as collateral damage, even though it has
    // nothing to do with the actual cause of the failure.
    let database_pool = app_handle.state::<sqlx::SqlitePool>().inner().clone();
    let sync_log_repo = SyncHistoryLogRepository::new(database_pool.clone());
    app_handle.manage(sync_log_repo.clone());

    let trusted_store = Arc::new(
        SecureTrustedStore::open(&app_data_directory, master_key).map_err(ComicError::Io)?,
    );
    let secure_p2p_storage =
        Arc::new(SecureP2pStorage::open(&app_data_directory, master_key).map_err(ComicError::Io)?);
    migrate_plaintext_seed_if_present(&app_data_directory, &secure_p2p_storage).await?;

    // `database_pool`/`sync_log_repo` already resolved above, before opening the
    // trust/peer storage.
    //
    // `library_root` (inside `file_sync_service`) re-evaluates the resolver on every call, not
    // once here — changing `library_path` in `settings.json` keeps working without having to
    // restart anything, P2P included.
    let remote_covers_dir = app_data_directory.join("remote_covers");

    let history_sync_service = HistorySyncService::new(database_pool.clone());
    let file_sync_service = FileSyncService::new(database_pool, {
        let app_data_directory = app_data_directory.clone();
        move || {
            read_library_path(&app_data_directory)
                .unwrap_or_else(|| app_data_directory.join("library"))
        }
    });
    // Reuses the same instance already managed by `bios::db::setup_database` (which runs via
    // `block_on` BEFORE this `setup_network` is even triggered — see `bios/mod.rs`) instead of
    // building a second `MetadataService` just for this: avoids duplicating the internal
    // `reqwest::Client` and keeps a single source of truth for reprocessing `ComicInfo.xml`
    // received via sync, whether triggered by the manual button or by P2P.
    let metadata_service = Arc::clone(&app_handle.state::<MetadataState>().service);
    // Shared between inbound and outbound: ensures only one sync-files session per peer runs
    // at a time, in both directions (see `file_session_guard.rs`). Also shared with
    // `COMIC_SYNC_ALPN` (same resource — file transfer — so a whole-library session and a
    // single-comic session for the same peer cannot run at the same time). A single instance
    // for the whole process lifetime — survives a node `restart()`, it is not recreated on every
    // rebuild (`P2pNodeContext::build` only reuses it, via `Arc::clone`).
    let file_sync_session_guard = FileSyncSessionGuard::new();

    // Side channel for the Tauri command `sync_comic` to tell which `comic_name` the
    // `ComicSyncOutbound` must use in the next session it starts with a given peer — see
    // `comic_sync_registry.rs` for why this is needed (the `Handler` is a singleton, it does not
    // receive per-call parameters from `connect()`). Registered as managed state (not recreated
    // on every rebuild) because Tauri commands already hold a fixed reference to it.
    let pending_comic_sync = PendingComicSyncRegistry::new();
    app_handle.manage(Arc::clone(&pending_comic_sync));

    // Same reason as `pending_comic_sync`: `sync_history_entry` (Tauri command) writes the
    // `comic_name` here before `connect()`, and `HistoryEntrySyncOutbound` consumes it at the
    // start of the session (see `history_entry_registry.rs`).
    let pending_history_entry = PendingHistoryEntryRegistry::new();
    app_handle.manage(Arc::clone(&pending_history_entry));

    let pending_cover_request = PendingCoverRequestRegistry::new();
    app_handle.manage(Arc::clone(&pending_cover_request));

    // `sync-files`/`sync-comic` handlers are registered on the builder BEFORE the node exists,
    // but need `node.blobs()`/`node.known_peers()` to publish/fetch blobs — `BlobContext`
    // holds a `Weak<AcerolaP2p>` re-pointed on every `.build()` (see
    // `infra::sync::blob_context`), including on a restart — the handlers themselves (and the
    // `chapter_transfer` that references them) are created only once here and survive.
    let blob_context = crate::infra::sync::blob_context::BlobContext::new();
    let chapter_transfer: Arc<dyn ChapterTransfer> =
        Arc::new(BlobChapterTransfer::new(Arc::clone(&blob_context)));

    let context = Arc::new(P2pNodeContext {
        app_data_directory: app_data_directory.clone(),
        event_emitter,
        trusted_store,
        secure_p2p_storage,
        history_sync_service,
        file_sync_service,
        metadata_service,
        sync_log_repo,
        file_sync_session_guard,
        pending_comic_sync,
        pending_history_entry,
        pending_cover_request,
        chapter_transfer,
        remote_covers_dir,
        blob_context,
    });

    // `NetworkService::restart()` (relay change, or the manual "Restart" button) calls this
    // again to build a fresh node with the same identity/storage/handlers — see the docs of
    // `NodeBuilder`/`P2pNodeContext`.
    let rebuild_context = Arc::clone(&context);
    let rebuild_node: crate::core::services::network::NodeBuilder = Arc::new(move || {
        let context = Arc::clone(&rebuild_context);
        Box::pin(async move { context.build().await.map_err(|err| err.to_string()) })
    });

    let network_service = Arc::new(NetworkService::new_uninitialized(
        Arc::clone(&context.secure_p2p_storage),
        Arc::clone(&context.trusted_store),
        app_data_directory,
        rebuild_node,
    ));
    app_handle.manage(Arc::clone(&network_service) as Arc<dyn NetworkServiceApi>);

    Ok(InitializedNetworkServices { context, network_service })
}

pub struct InitializedNetworkServices {
    context: Arc<P2pNodeContext>,
    network_service: Arc<NetworkService>,
}

impl InitializedNetworkServices {
    pub async fn start_node(self) -> Result<(), ComicError> {
        let p2p_node = self.context.build().await?;
        self.network_service.set_node(p2p_node);
        tracing::info!("[Bios::Network] P2P network service initialized successfully");
        Ok(())
    }
}

pub async fn setup_network(app_handle: &tauri::AppHandle) -> Result<(), ComicError> {
    let services = setup_network_services(app_handle).await?;
    services.start_node().await?;
    Ok(())
}
