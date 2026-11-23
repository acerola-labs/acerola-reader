package br.acerola.comic.module.main.sync.state

import br.acerola.comic.error.message.SyncProtocolError
import br.acerola.comic.service.NetworkMode
import br.acerola.comic.service.network.ComicSummary

data class SyncUiState(
    val localId: String = "",
    val localDeviceName: String = "",
    val pairingCode: String? = null,
    val mode: NetworkMode = NetworkMode.LOCAL,
    val relaySettings: RelaySettingsUiState = RelaySettingsUiState(),
    /** `true` right after `SyncAction.SetIrohServicesTicket` fails (invalid format) —
     *  [RelaySettingsUiState.hasIrohServicesTicket] itself doesn't change in that case. */
    val irohServicesTicketError: Boolean = false,
    val pairedPeers: List<PairedPeer> = emptyList(),
    val pendingConnect: PendingConnect? = null,
    val connecting: Boolean = false,
    val connectError: ConnectError? = null,
    val trustedPeerDialogPeerId: String? = null,
    val transferLog: List<TransferLogEntry> = emptyList(),
    /**
     * `kind` -> id of the [transferLog] row currently in progress for that protocol
     * (spinning spinner). Lets [TransferLogEntry] transition in place (started -> progress
     * -> complete/error) instead of stacking a new row per event, which left the
     * "started" row stuck forever as if it were still running.
     */
    val inFlightLogEntryByKind: Map<String, Long> = emptyMap(),
    /**
     * `"$peerId:$kind"` keys (kind = "history"/"files") with a sync session in flight —
     * disables that peer/protocol's buttons to avoid firing a second concurrent session for
     * the same (peer, ALPN) pair. Granularity is per kind, not just per peer: "Sync All"
     * fires history AND files for the same peer, and both need to run at once without
     * blocking each other.
     */
    val syncingKeys: Set<String> = emptySet(),
    /** `peerId` -> timestamp (epoch ms) of the last successfully completed session. */
    val lastSyncedByPeer: Map<String, Long> = emptyMap(),
    /** `peerId`s currently reachable in this session (subset of [pairedPeers]) — presence, not pairing. */
    val connectedPeerIds: Set<String> = emptySet(),
    /** `peerId` -> outcome of the most recent sync attempt (either kind), for inline feedback on [PeerRow]. */
    val lastSyncResultByPeer: Map<String, SyncResult> = emptyMap(),
    /** `peerId` of the peer whose [RemoteLibrarySheet] is currently open, or `null` if none.
     *  Also drives the loading state of that sheet while [remoteLibrary] hasn't arrived yet. */
    val browsingPeerId: String? = null,
    /** Result of the last `BrowseLibrary` request for [browsingPeerId] — cleared whenever a
     *  new browse starts or the sheet is dismissed. */
    val remoteLibrary: List<ComicSummary> = emptyList(),
    /** Distinguishes "still waiting for [P2pEvent.LibraryBrowseResult]" from "peer really has
     *  an empty library" — [remoteLibrary] alone can't tell those apart once it's empty. */
    val remoteLibraryLoaded: Boolean = false,
    /** Raw — only used as a fallback/template argument when [browseLibraryErrorType] is
     *  `null` (see `RemoteLibrarySheet`, same pattern as `TransferLogEntry`/`SyncResult`). */
    val browseLibraryError: String? = null,
    /** Recognized cause (`SyncProtocolError`), when there is one. */
    val browseLibraryErrorType: SyncProtocolError? = null,
    /** `"$peerId:$comicName"` -> local path (`file://...`) of the cover already downloaded via
     *  `acerola/browse-cover/1`, ready for Coil to load. In-memory cache — never downloads the
     *  same version twice within the same app session (see
     *  [br.acerola.comic.module.main.sync.SyncViewModel]). */
    val remoteCoverPaths: Map<String, String> = emptyMap(),
)

/**
 * Combinable relay configuration shown/edited in [br.acerola.comic.module.main.sync.RelaySettingsCard]
 * — mirrors `RelayPreference.RelaySettings` (the persisted source), with its own name here for the
 * same reason as [PairedPeer]/[SyncResult]: `SyncUiState` shouldn't carry another layer's types
 * for convenience, the same shape is just reused on purpose.
 */
data class RelaySettingsUiState(
    val useAcerolaRelay: Boolean = true,
    val useIrohPublicNetwork: Boolean = false,
    val customRelayUrls: List<String> = emptyList(),
    /** Only indicates WHETHER a ticket for the user's `services.iroh.computer` account was already
     *  pasted and saved — the value itself is never exposed in the UI (it's a real credential, kept
     *  in the node's encrypted vault, not in DataStore alongside the other relay preferences). */
    val hasIrohServicesTicket: Boolean = false,
    /** `true` while `SyncAction.RestartP2p` is in progress — disables the "Restart"
     *  button to avoid firing two simultaneous restarts (the second would shut down a node
     *  the first one just started). */
    val restarting: Boolean = false,
    /** `true` right after a `RestartP2p` fails — inline message on the card, same pattern as
     *  [SyncUiState.irohServicesTicketError]. */
    val restartError: Boolean = false,
)

data class SyncResult(
    val kind: String,
    val state: LogState,
    /** Raw/in English — never shown directly, only embedded in an already translated template
     *  (see `SyncScreen::PeerRow`). Prefer [errorType] when present: it's the recognized cause
     *  (`SyncProtocolError`), with its own message, more specific than the generic fallback
     *  template. */
    val message: String?,
    val timestamp: Long,
    val errorType: SyncProtocolError? = null,
)

data class PairedPeer(
    val peerId: String,
    val deviceName: String?,
    /** Local nickname (set by the user on this device, see `PeerNicknamePreference`) —
     *  takes priority over [deviceName] for display, without ever overwriting it. */
    val nickname: String? = null,
)

data class PendingConnect(
    val peerId: String,
    val deviceId: String?,
    val addrs: ByteArray,
)

enum class ConnectError {
    INVALID_CODE,
    CONNECTION_FAILED,
}

enum class LogState {
    IN_PROGRESS,
    SUCCESS,
    ERROR,
}

/**
 * Raw data for one activity-log row — deliberately no pre-rendered display string here.
 * [br.acerola.comic.module.main.sync.SyncScreen] is the one that turns this into text via
 * `stringResource`, same as it already does for [ConnectError] in `LaunchedEffect`. Keeps
 * string-resource resolution (and thus any [android.content.Context] dependency) out of the
 * ViewModel entirely.
 */
data class TransferLogEntry(
    val id: Long,
    /** `"history"` or `"files"`. */
    val kind: String,
    /** `"started"`, `"progress"`, `"complete"`, `"error"`, or `"chapterFailed"` (files only). */
    val status: String,
    val state: LogState,
    val comicName: String? = null,
    val chapter: String? = null,
    val message: String? = null,
    val timestamp: Long = System.currentTimeMillis(),
)
