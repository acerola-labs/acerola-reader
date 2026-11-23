use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager, Runtime, State};

use crate::{
    bios::scopes::read_relay_settings,
    cmd::events::network::{DeviceInfoPayload, NetworkStatusPayload, PairedPeerPayload, RelayInfo},
    core::services::network::NetworkServiceApi,
    data::{models::sync::SyncHistoryLogEntry, repositories::sync::SyncHistoryLogRepository},
    infra::{
        security::MasterKeySource,
        sync::{
            messages::SyncDirection,
            protocol::{
                comic_sync_registry::PendingComicSyncRegistry,
                cover_request_registry::PendingCoverRequestRegistry,
                history_entry_registry::{HistoryEntryScope, PendingHistoryEntryRegistry},
                COMIC_SYNC_ALPN, COVER_BROWSE_ALPN, FILE_SYNC_ALPN, HISTORY_ENTRY_SYNC_ALPN,
                HISTORY_SYNC_ALPN, LIBRARY_BROWSE_ALPN,
            },
        },
    },
};

const SYNC_HISTORY_LOG_LIMIT: i64 = 200;

#[tauri::command]
pub async fn get_network_status<R: Runtime>(
    app: AppHandle<R>, service: State<'_, Arc<dyn NetworkServiceApi>>,
) -> Result<(), String> {
    let mode = service.mode().await?;
    let peers = service.connected_peers_with_info().await?;

    let _ = app.emit("network:status", NetworkStatusPayload::from(mode, peers));

    Ok(())
}

#[tauri::command]
pub async fn switch_to_local(service: State<'_, Arc<dyn NetworkServiceApi>>) -> Result<(), String> {
    service.switch_to_local().await?;
    Ok(())
}

#[tauri::command]
pub async fn switch_to_relay(service: State<'_, Arc<dyn NetworkServiceApi>>) -> Result<(), String> {
    service.switch_to_relay().await?;
    Ok(())
}

#[tauri::command]
pub async fn get_local_id(
    service: State<'_, Arc<dyn NetworkServiceApi>>,
) -> Result<String, String> {
    service.local_id()
}

/// Full address to generate the pairing code/QR (see [`NetworkServiceApi::local_addr`]).
#[tauri::command]
pub async fn get_local_addr(
    service: State<'_, Arc<dyn NetworkServiceApi>>,
) -> Result<acerola_p2p::api::peer::PeerAddr, String> {
    service.local_addr()
}

/// Name/OS/version of this device, to show something readable on the Network screen instead
/// of the raw peer id.
#[tauri::command]
pub async fn get_local_device_info(
    service: State<'_, Arc<dyn NetworkServiceApi>>,
) -> Result<DeviceInfoPayload, String> {
    Ok(DeviceInfoPayload::from(service.local_device_info().await?))
}

/// Sets a custom alias for the local device (LocalSend style, instead of the automatic
/// hostname) — applies from the next handshake on, without restarting the app. Persisting it
/// across restarts is the frontend's job (`settings.json`, key `device_alias`), which
/// `bios::network::setup_network` re-reads on the next boot.
#[tauri::command]
pub async fn set_local_device_name(
    service: State<'_, Arc<dyn NetworkServiceApi>>, name: String,
) -> Result<(), String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("device name cannot be empty".to_string());
    }

    service.set_local_device_name(trimmed.to_string()).await
}

/// Returns the current combinable relay configuration (Acerola relay / own /
/// Iroh / Iroh public network), read from `settings.json`.
#[tauri::command]
pub async fn get_relay_info<R: Runtime>(
    app: AppHandle<R>, service: State<'_, Arc<dyn NetworkServiceApi>>,
) -> Result<RelayInfo, String> {
    let app_data_directory = app.path().app_data_dir().map_err(|error| error.to_string())?;
    let has_ticket = service.has_iroh_services_ticket().await?;
    Ok(RelayInfo::new(read_relay_settings(&app_data_directory), has_ticket))
}

/// Saves the ticket of the user's `services.iroh.computer` account (pasted on the network
/// settings screen) in the encrypted vault — never in `settings.json`. Validates the format
/// before persisting. Does not apply the new config by itself — the frontend calls
/// [`apply_relay_settings`] right after, same flow as any other relay change.
#[tauri::command]
pub async fn set_iroh_services_ticket(
    service: State<'_, Arc<dyn NetworkServiceApi>>, ticket: String,
) -> Result<(), String> {
    service.set_iroh_services_ticket(ticket).await
}

/// Removes the saved ticket — used when the user turns the source off or replaces it with a new one.
#[tauri::command]
pub async fn clear_iroh_services_ticket(
    service: State<'_, Arc<dyn NetworkServiceApi>>,
) -> Result<(), String> {
    service.clear_iroh_services_ticket().await
}

/// Re-reads `settings.json` + the vault ticket and restarts the whole P2P module with the
/// resolved relay config (see `NetworkServiceApi::restart`), without restarting the app — the
/// frontend calls this after ANY change to the relay sources (Acerola/Iroh toggle,
/// add/remove own URL, save/remove ticket).
#[tauri::command]
pub async fn apply_relay_settings(
    service: State<'_, Arc<dyn NetworkServiceApi>>,
) -> Result<(), String> {
    service.apply_relay_settings().await
}

/// Fully restarts the P2P module (shuts down the current node, starts a new one with the same
/// identity/storage) on demand — the "Restart" button on the Network screen, LocalSend style.
/// Same operation [`apply_relay_settings`] already triggers automatically on a relay change;
/// exposed separately so the user can force a reset even without changing anything (e.g. a
/// connection stuck after a physical network change, without closing and reopening the app).
#[tauri::command]
pub async fn restart_p2p(service: State<'_, Arc<dyn NetworkServiceApi>>) -> Result<(), String> {
    service.restart().await
}

#[tauri::command]
pub async fn connect_to_peer(
    service: State<'_, Arc<dyn NetworkServiceApi>>, peer_id: String, addrs: Vec<u8>, alpn: String,
) -> Result<(), String> {
    use acerola_p2p::api::peer::{PeerAddr, PeerIdentity};

    let peer_identity = PeerIdentity { id: peer_id, device_id: None };
    let peer_addr = PeerAddr { id: peer_identity, addrs };
    service.connect(peer_addr, alpn.into_bytes()).await?;
    Ok(())
}

/// Every peer ever paired, with the last known address — survives restarts and does not
/// depend on an active connection right now (see [`NetworkServiceApi::paired_peers`]). It is
/// this list, not `get_network_status`, that must feed "start sync with X" in the UI.
#[tauri::command]
pub async fn get_paired_peers(
    service: State<'_, Arc<dyn NetworkServiceApi>>,
) -> Result<Vec<PairedPeerPayload>, String> {
    Ok(service.paired_peers().await?.into_iter().map(PairedPeerPayload::from).collect())
}

/// Unpairs a peer — it disappears from the paired list and from trust (TOFU). If it tries to
/// connect again later, it goes through the same confirmation flow as a never-seen device
/// (see [`NetworkServiceApi::remove_peer`]).
#[tauri::command]
pub async fn remove_paired_peer(
    service: State<'_, Arc<dyn NetworkServiceApi>>, peer_id: String,
) -> Result<(), String> {
    service.remove_peer(peer_id).await
}

/// Starts a history sync session with an already-paired peer. Progress and completion
/// reach the frontend via the `sync:history:*` events, emitted directly by the
/// protocol (see `infra::sync::protocol::history_handler`) as soon as the connection is accepted.
#[tauri::command]
pub async fn sync_history(
    service: State<'_, Arc<dyn NetworkServiceApi>>, peer_id: String, addrs: Vec<u8>,
) -> Result<(), String> {
    use acerola_p2p::api::peer::{PeerAddr, PeerIdentity};

    let peer_addr = PeerAddr { id: PeerIdentity { id: peer_id, device_id: None }, addrs };
    service.connect(peer_addr, HISTORY_SYNC_ALPN.to_vec()).await?;
    Ok(())
}

/// Pushes the selected chapter(s) (progress + "read" marker) of ONE single comic to an
/// already-paired peer — lighter than `sync_history` (which exchanges the whole library in
/// both directions). Used both for sending a single chapter and several selected at once.
/// Registers the [`HistoryEntryScope`] in the `PendingHistoryEntryRegistry` before
/// connecting, same technique as `sync_comic`. Progress via the
/// `sync:history-entry:*` events.
#[tauri::command]
pub async fn sync_history_entry(
    service: State<'_, Arc<dyn NetworkServiceApi>>,
    registry: State<'_, Arc<PendingHistoryEntryRegistry>>, peer_id: String, addrs: Vec<u8>,
    comic_name: String, chapter_ids: Vec<String>,
) -> Result<(), String> {
    use acerola_p2p::api::peer::{PeerAddr, PeerIdentity};

    // Same pattern as `history_mark_chapters_read_batch`: the frontend treats chapter IDs
    // as strings (`ChapterId` in `use-chapter-selection.svelte.ts`), the parse to `i64` (what
    // `HistorySyncService` actually expects) happens here.
    let chapter_ids = chapter_ids
        .into_iter()
        .map(|id| id.parse::<i64>().map_err(|error| error.to_string()))
        .collect::<Result<Vec<_>, _>>()?;

    registry.set(peer_id.clone(), HistoryEntryScope { comic_name, chapter_ids });

    let peer_addr = PeerAddr { id: PeerIdentity { id: peer_id, device_id: None }, addrs };
    service.connect(peer_addr, HISTORY_ENTRY_SYNC_ALPN.to_vec()).await?;
    Ok(())
}

/// Starts a file sync session with an already-paired peer. Progress via the
/// `sync:files:*` events.
#[tauri::command]
pub async fn sync_files(
    service: State<'_, Arc<dyn NetworkServiceApi>>, peer_id: String, addrs: Vec<u8>,
) -> Result<(), String> {
    use acerola_p2p::api::peer::{PeerAddr, PeerIdentity};

    let peer_addr = PeerAddr { id: PeerIdentity { id: peer_id, device_id: None }, addrs };
    service.connect(peer_addr, FILE_SYNC_ALPN.to_vec()).await?;
    Ok(())
}

/// Runs history and files in sequence against the same peer.
#[tauri::command]
pub async fn sync_all(
    service: State<'_, Arc<dyn NetworkServiceApi>>, peer_id: String, addrs: Vec<u8>,
) -> Result<(), String> {
    use acerola_p2p::api::peer::{PeerAddr, PeerIdentity};

    let peer_identity = PeerIdentity { id: peer_id, device_id: None };

    let history_addr = PeerAddr { id: peer_identity.clone(), addrs: addrs.clone() };
    service.connect(history_addr, HISTORY_SYNC_ALPN.to_vec()).await?;

    let files_addr = PeerAddr { id: peer_identity, addrs };
    service.connect(files_addr, FILE_SYNC_ALPN.to_vec()).await?;

    Ok(())
}

/// Starts an individual sync session of ONE comic with an already-paired peer — works both
/// to "push" (I have it, the peer doesn't) and to "pull" (the peer has it, I don't), see the
/// design comment in `infra::sync::protocol::comic_handler`. Registers the `comic_name` in
/// `PendingComicSyncRegistry` before connecting, because the `Handler` (`ComicSyncOutbound`) is a
/// boot-time singleton and does not receive this parameter per `connect()` call. Progress via
/// the `sync:comic:*` events.
#[tauri::command]
pub async fn sync_comic(
    service: State<'_, Arc<dyn NetworkServiceApi>>,
    registry: State<'_, Arc<PendingComicSyncRegistry>>, peer_id: String, addrs: Vec<u8>,
    comic_name: String, direction: SyncDirection, chapter_ids: Vec<String>,
) -> Result<(), String> {
    use acerola_p2p::api::peer::{PeerAddr, PeerIdentity};

    // Same pattern as `sync_history_entry`: the frontend treats chapter IDs as strings
    // (`ChapterId` in `use-chapter-selection.svelte.ts`), the parse to `i64`
    // (`chapter_archive.id`) happens here. Empty list = whole comic (the
    // "Sync with device" button), not scoped to any specific chapter.
    let chapter_ids = chapter_ids
        .into_iter()
        .map(|id| id.parse::<i64>().map_err(|error| error.to_string()))
        .collect::<Result<Vec<_>, _>>()?;

    registry.set(peer_id.clone(), comic_name, direction, chapter_ids);

    let peer_addr = PeerAddr { id: PeerIdentity { id: peer_id, device_id: None }, addrs };
    service.connect(peer_addr, COMIC_SYNC_ALPN.to_vec()).await?;
    Ok(())
}

/// Queries the remote library of an already-paired peer (titles + chapter count only, no
/// transfer) — prerequisite for choosing a comic to pull (`sync_comic`). The result reaches
/// the frontend via the `library:query:result` event.
#[tauri::command]
pub async fn query_remote_library(
    service: State<'_, Arc<dyn NetworkServiceApi>>, peer_id: String, addrs: Vec<u8>,
) -> Result<(), String> {
    use acerola_p2p::api::peer::{PeerAddr, PeerIdentity};

    let peer_addr = PeerAddr { id: PeerIdentity { id: peer_id, device_id: None }, addrs };
    service.connect(peer_addr, LIBRARY_BROWSE_ALPN.to_vec()).await?;
    Ok(())
}

/// Fetches the cover (thumbnail) of ONE remote comic — `known_version` is the version already
/// cached locally (`None` if this cover was never fetched). Same fire-and-forget pattern as
/// `sync_comic`: enqueues `(comic_name, known_version)` in `PendingCoverRequestRegistry` before
/// connecting, because the `Handler` (`CoverBrowseOutbound`) is a boot-time singleton and does
/// not receive this parameter per `connect()` call. `use-remote-library.svelte.ts::fetchCoversFor`
/// calls this command in parallel (one per comic in the list) for the same peer — that is why
/// the registry is a per-peer queue (`push`/`take` FIFO), not a single slot where one call would
/// overwrite the other (bug reported/fixed on 2026-08-22, see the registry docs).
/// Result via `browse:cover:result`/`browse:cover:error`.
#[tauri::command]
pub async fn query_remote_cover(
    service: State<'_, Arc<dyn NetworkServiceApi>>,
    registry: State<'_, Arc<PendingCoverRequestRegistry>>, peer_id: String, addrs: Vec<u8>,
    comic_name: String, known_version: Option<i64>,
) -> Result<(), String> {
    use acerola_p2p::api::peer::{PeerAddr, PeerIdentity};

    registry.push(peer_id.clone(), comic_name, known_version);

    let peer_addr = PeerAddr { id: PeerIdentity { id: peer_id, device_id: None }, addrs };
    service.connect(peer_addr, COVER_BROWSE_ALPN.to_vec()).await?;
    Ok(())
}

/// Last persisted sync sessions (history and files) — survives restarts, unlike the
/// frontend's in-memory live log (`use-network-sync.svelte.ts`), which only has the events
/// of the app's current session.
#[tauri::command]
pub async fn get_sync_history_log(
    repo: State<'_, SyncHistoryLogRepository>,
) -> Result<Vec<SyncHistoryLogEntry>, String> {
    repo.find_recent(SYNC_HISTORY_LOG_LIMIT).await.map_err(|error| error.to_string())
}

/// Deletes the whole persisted sync history — the "Clear" button on the Network screen.
/// Irreversible, the UI already confirms with the user before calling.
#[tauri::command]
pub async fn clear_sync_history_log(
    repo: State<'_, SyncHistoryLogRepository>,
) -> Result<(), String> {
    repo.delete_all().await.map_err(|error| error.to_string())
}

/// If `true`, the master key that encrypts identity/peers/trust fell back to a local file
/// because no usable OS keyring was available — see
/// `infra::security::get_or_create_master_key`. Queried on demand (instead of relying only on
/// the `security:keyring_unavailable` event) because `setup_network` runs in a separate task
/// that may finish before the frontend mounts and starts listening for events.
#[tauri::command]
pub async fn get_security_status(source: State<'_, MasterKeySource>) -> Result<bool, String> {
    Ok(*source == MasterKeySource::FallbackFile)
}
