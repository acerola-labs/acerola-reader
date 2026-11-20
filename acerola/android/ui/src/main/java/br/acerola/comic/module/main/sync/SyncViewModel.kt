package br.acerola.comic.module.main.sync

import android.content.Context
import android.os.Build
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import br.acerola.comic.config.preference.DeviceAliasPreference
import br.acerola.comic.config.preference.PeerNicknamePreference
import br.acerola.comic.config.preference.RelayPreference
import br.acerola.comic.error.message.SyncProtocolError
import br.acerola.comic.logging.AcerolaLogger
import br.acerola.comic.logging.LogSource
import br.acerola.comic.module.main.sync.state.ConnectError
import br.acerola.comic.module.main.sync.state.LogState
import br.acerola.comic.module.main.sync.state.PairedPeer
import br.acerola.comic.module.main.sync.state.PendingConnect
import br.acerola.comic.module.main.sync.state.SyncAction
import br.acerola.comic.module.main.sync.state.SyncResult
import br.acerola.comic.module.main.sync.state.SyncUiState
import br.acerola.comic.module.main.sync.state.TransferLogEntry
import br.acerola.comic.service.PeerAddress
import br.acerola.comic.service.RelaySettings
import br.acerola.comic.service.SyncDirection
import br.acerola.comic.service.network.ComicSummary
import br.acerola.comic.service.network.P2pEvent
import br.acerola.comic.service.network.P2pEventBus
import br.acerola.comic.usecase.network.P2pUseCase
import br.acerola.comic.usecase.network.SyncHistoryLogUseCase
import br.acerola.comic.util.p2p.PairingCode
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicLong
import javax.inject.Inject

private const val HANDSHAKE_ALPN = "acerola/handshake/1"
private const val HISTORY_SYNC_ALPN = "acerola/sync-history/1"
private const val FILE_SYNC_ALPN = "acerola/sync-files/1"
private const val CONNECT_TIMEOUT_MS = 15_000L
private const val MAX_LOG_ENTRIES = 50

// A real network may never deliver the completion/error event (peer vanished mid-session) —
// without this, that peer/protocol's buttons would stay disabled forever.
private const val SYNC_IN_FLIGHT_TIMEOUT_MS = 60_000L
internal const val SYNC_KIND_HISTORY = "history"
internal const val SYNC_KIND_FILES = "files"

/** Shared with [SyncScreen] to decide whether a sync button should be disabled. */
internal fun syncKey(
    peerId: String,
    kind: String,
) = "$peerId:$kind"

/** Shared with [RemoteLibrarySheet] to look up [SyncUiState.remoteCoverPaths] per item. */
internal fun coverKey(
    peerId: String,
    comicName: String,
) = "$peerId:$comicName"

@HiltViewModel
class SyncViewModel
    @Inject
    constructor(
        private val p2pUseCase: P2pUseCase,
        private val p2pEventBus: P2pEventBus,
        private val syncHistoryLogUseCase: SyncHistoryLogUseCase,
        @param:ApplicationContext private val context: Context,
    ) : ViewModel() {
        private val _uiState = MutableStateFlow(SyncUiState())
        val uiState: StateFlow<SyncUiState> = _uiState.asStateFlow()

        private val nextLogId = AtomicLong(0)

        /** Latest snapshot of [PeerNicknamePreference.nicknamesFlow] — read by
         *  [refreshLocalInfo] so it doesn't wipe the already-applied nicknames while rebuilding
         *  [SyncUiState.pairedPeers] from scratch from the native side (which knows nothing about
         *  local nicknames). */
        @Volatile
        private var currentNicknames: Map<String, String> = emptyMap()

        /** `"$peerId:$comicName"` -> latest confirmed cover version (cached or already known
         *  to be current) — used as `known_version` in `browseCover`, so the same version is never
         *  downloaded twice in the same app session. Kept out of `SyncUiState` on purpose: it
         *  doesn't need to trigger recomposition on its own, only [SyncUiState.remoteCoverPaths]
         *  matters to the UI. */
        private val knownCoverVersions = ConcurrentHashMap<String, Long>()

        init {
            viewModelScope.launch { refreshLocalInfo() }
            viewModelScope.launch { loadPersistedLog() }

            viewModelScope.launch {
                RelayPreference.relaySettingsFlow(context).collect { settings ->
                    // `.copy(...)` on top of the current `relaySettings`, not a brand-new
                    // `RelaySettingsUiState` — `hasIrohServicesTicket` comes from another source (the
                    // node's native vault, see below), not from DataStore, and must not be reset on
                    // every emission of this Flow.
                    _uiState.update {
                        it.copy(
                            relaySettings =
                                it.relaySettings.copy(
                                    useAcerolaRelay = settings.useAcerolaRelay,
                                    useIrohPublicNetwork = settings.useIrohPublicNetwork,
                                    customRelayUrls = settings.customRelayUrls,
                                ),
                        )
                    }

                    // Applies to the ALREADY RUNNING node — without this, changing any relay source
                    // on screen only took effect on the NEXT app boot. Redundant (but harmless, the
                    // diff becomes a no-op) on the first emission of this Flow, which just repeats
                    // the config the node was already built with.
                    withContext(Dispatchers.IO) { applyCurrentRelaySettingsLive() }
                }
            }

            viewModelScope.launch(Dispatchers.IO) {
                val hasTicket = p2pUseCase.hasIrohServicesTicket()
                _uiState.update {
                    it.copy(relaySettings = it.relaySettings.copy(hasIrohServicesTicket = hasTicket))
                }
            }

            // The saved alias was already read to build the node (`NetworkCaseModule`) — this
            // second read only mirrors the same value here in the UI, since the node doesn't
            // return the `DeviceInfo` it got at boot. After that, `localDeviceName` only
            // changes via `renameDevice` — never via `refreshLocalInfo`, which would overwrite
            // what was just renamed on every network event.
            viewModelScope.launch {
                val alias = DeviceAliasPreference.deviceAliasFlow(context).first()
                _uiState.update { it.copy(localDeviceName = alias ?: Build.MODEL) }
            }

            // Local nicknames per peer — reapplied on top of `pairedPeers` on every change
            // (rename or unpair), without needing a new `refreshLocalInfo()` (which hits
            // the native side via JNI) just for that.
            viewModelScope.launch {
                PeerNicknamePreference.nicknamesFlow(context).collect { nicknames ->
                    currentNicknames = nicknames
                    _uiState.update { state ->
                        state.copy(
                            pairedPeers = state.pairedPeers.map { peer -> peer.copy(nickname = nicknames[peer.peerId]) },
                        )
                    }
                }
            }

            // Only for THIS screen's UI state (in-memory log, button spinners) — persistence/
            // notification/foreground service of a P2P session moved from here to
            // `P2pSyncCoordinator` (application-scoped singleton, see KDoc there): since this
            // ViewModel is destroyed when leaving the screen (`hiltViewModel()` scoped to the
            // `NavBackStackEntry`), it isn't the right place for a result that needs to
            // survive navigation.
            viewModelScope.launch {
                p2pEventBus.events.collect(::handleEvent)
            }
        }

        /**
         * Loads the persisted sessions (most recent first) when the screen first opens —
         * only to feed the summary shown in [ActivityLogCard]'s hero button; the full list
         * lives on its own screen
         * ([br.acerola.comic.module.main.transferlog.TransferLogActivity]/
         * [br.acerola.comic.module.main.transferlog.TransferLogViewModel], which
         * fetches on its own). Only replaces the part of the log that came from there (`id < 0`) —
         * keeps any LIVE entry still at `id >= 0` (`started`/`progress` session
         * of this app run). Negative IDs so they never collide with the live IDs
         * generated by [nextLogId] (which only grows from 0).
         */
        private suspend fun loadPersistedLog() {
            val rows = syncHistoryLogUseCase.findRecent(MAX_LOG_ENTRIES)

            val entries =
                rows.map { row ->
                    TransferLogEntry(
                        id = -row.id,
                        kind = row.kind,
                        status = row.status,
                        state = if (row.status == "complete") LogState.SUCCESS else LogState.ERROR,
                        message = row.message,
                        timestamp = row.createdAt,
                    )
                }

            val lastSynced =
                rows
                    .filter { it.status == "complete" }
                    .groupBy { it.peerId }
                    .mapValues { (_, entriesForPeer) -> entriesForPeer.maxOf { it.createdAt } }

            _uiState.update { current ->
                val liveEntries = current.transferLog.filter { it.id >= 0 }
                current.copy(transferLog = liveEntries + entries, lastSyncedByPeer = lastSynced)
            }
        }


        /** Records a session's terminal result only in THIS screen's UI state — updates
         *  "last sync" only on success, but [SyncUiState.lastSyncResultByPeer]
         *  always, so [SyncScreen] can show a failure inline on the peer's row without
         *  digging through the aggregated log. Persisting to disk is now the job of
         *  [P2pSyncCoordinator] (survives this ViewModel being destroyed), not this function —
         *  writing here too would duplicate the row in [SyncHistoryLogUseCase] whenever both
         *  are alive at the same time. */
        private fun recordSyncResult(
            peerId: String,
            kind: String,
            status: String,
            message: String?,
            errorType: SyncProtocolError? = null,
        ) {
            val now = System.currentTimeMillis()
            _uiState.update {
                it.copy(
                    lastSyncedByPeer = if (status == "complete") it.lastSyncedByPeer + (peerId to now) else it.lastSyncedByPeer,
                    lastSyncResultByPeer =
                        it.lastSyncResultByPeer +
                            (
                                peerId to
                                    SyncResult(
                                        kind = kind,
                                        state = if (status == "complete") LogState.SUCCESS else LogState.ERROR,
                                        message = message,
                                        timestamp = now,
                                        errorType = errorType,
                                    )
                            ),
                )
            }
        }

        fun onAction(action: SyncAction) {
            when (action) {
                is SyncAction.RenameDevice -> renameDevice(action.name)
                is SyncAction.ProposeConnect -> proposeConnect(action.code)
                SyncAction.ConfirmConnect -> confirmConnect()
                SyncAction.CancelConnect -> _uiState.update { it.copy(pendingConnect = null) }
                is SyncAction.SyncHistory -> triggerSync(action.peerId, HISTORY_SYNC_ALPN, SYNC_KIND_HISTORY)
                is SyncAction.SyncFiles -> triggerSync(action.peerId, FILE_SYNC_ALPN, SYNC_KIND_FILES)
                is SyncAction.SyncAll -> {
                    triggerSync(action.peerId, HISTORY_SYNC_ALPN, SYNC_KIND_HISTORY)
                    triggerSync(action.peerId, FILE_SYNC_ALPN, SYNC_KIND_FILES)
                }

                SyncAction.DismissTrustDialog -> _uiState.update { it.copy(trustedPeerDialogPeerId = null) }
                SyncAction.DismissConnectError -> _uiState.update { it.copy(connectError = null) }
                is SyncAction.RemovePeer -> removePeer(action.peerId)
                is SyncAction.RenamePeer -> renamePeer(action.peerId, action.nickname)

                is SyncAction.BrowseLibrary -> browseLibrary(action.peerId)
                SyncAction.DismissLibraryBrowse ->
                    _uiState.update {
                        it.copy(
                            browsingPeerId = null,
                            remoteLibrary = emptyList(),
                            remoteLibraryLoaded = false,
                            browseLibraryError = null,
                            browseLibraryErrorType = null,
                        )
                    }
                is SyncAction.SyncComic -> syncComic(action.peerId, action.comicName)

                is SyncAction.ToggleUseAcerolaRelay ->
                    viewModelScope.launch { RelayPreference.setUseAcerolaRelay(context, action.value) }
                is SyncAction.ToggleUseIrohPublicNetwork ->
                    viewModelScope.launch { RelayPreference.setUseIrohPublicNetwork(context, action.value) }
                is SyncAction.AddCustomRelayUrl ->
                    viewModelScope.launch { RelayPreference.addCustomRelayUrl(context, action.url) }
                is SyncAction.RemoveCustomRelayUrl ->
                    viewModelScope.launch { RelayPreference.removeCustomRelayUrl(context, action.url) }
                is SyncAction.SetIrohServicesTicket -> setIrohServicesTicket(action.ticket)
                SyncAction.ClearIrohServicesTicket -> clearIrohServicesTicket()
                SyncAction.DismissIrohServicesTicketError ->
                    _uiState.update { it.copy(irohServicesTicketError = false) }

                SyncAction.RestartP2p -> restartP2p()
            }
        }

        /** Validates the format on the native side before persisting — `p2pUseCase.setIrohServicesTicket`
         *  throws `IllegalArgumentException` if malformed, which becomes `irohServicesTicketError`. */
        private fun setIrohServicesTicket(ticket: String) {
            viewModelScope.launch(Dispatchers.IO) {
                try {
                    p2pUseCase.setIrohServicesTicket(ticket)
                    _uiState.update {
                        it.copy(
                            relaySettings = it.relaySettings.copy(hasIrohServicesTicket = true),
                            irohServicesTicketError = false,
                        )
                    }
                    applyCurrentRelaySettingsLive()
                } catch (error: IllegalArgumentException) {
                    AcerolaLogger.w("SyncViewModel", "Invalid Iroh Services ticket: ${error.message}", LogSource.UI)
                    _uiState.update { it.copy(irohServicesTicketError = true) }
                }
            }
        }

        private fun clearIrohServicesTicket() {
            viewModelScope.launch(Dispatchers.IO) {
                p2pUseCase.clearIrohServicesTicket()
                _uiState.update {
                    it.copy(relaySettings = it.relaySettings.copy(hasIrohServicesTicket = false))
                }
                applyCurrentRelaySettingsLive()
            }
        }

        /** Reapplies `relaySettings` (current UI state) to the already running node — called after
         *  saving/removing the ticket, since that changes the result of `resolve()` (e.g. turns the
         *  "Iroh public network" source on/off) without `RelayPreference.relaySettingsFlow` having
         *  emitted anything new (the ticket doesn't live in DataStore). Must already run on
         *  `Dispatchers.IO` (blocking call on the Rust side). */
        private suspend fun applyCurrentRelaySettingsLive() {
            val relaySettings = _uiState.value.relaySettings
            runCatching {
                p2pUseCase.applyRelaySettings(
                    RelaySettings(
                        useAcerolaRelay = relaySettings.useAcerolaRelay,
                        useIrohPublicNetwork = relaySettings.useIrohPublicNetwork,
                        customRelayUrls = relaySettings.customRelayUrls,
                    ),
                )
            }.onFailure { error ->
                AcerolaLogger.w(
                    "SyncViewModel",
                    "Failed to apply relay settings live: ${error.message}",
                    LogSource.UI,
                )
            }
        }

        /** Manual "Restart" button — shuts down the current P2P node and starts a new one from scratch
         *  (same identity/storage/paired peers, see `P2PNode::restart` in Rust). Passes the CURRENT
         *  relay config from the UI (same source as [applyCurrentRelaySettingsLive]), since the new
         *  node must resolve the relay mode from scratch, with no previous `Endpoint` to inherit from. */
        private fun restartP2p() {
            if (_uiState.value.relaySettings.restarting) return
            _uiState.update { it.copy(relaySettings = it.relaySettings.copy(restarting = true, restartError = false)) }

            viewModelScope.launch(Dispatchers.IO) {
                val relaySettings = _uiState.value.relaySettings
                val result =
                    runCatching {
                        p2pUseCase.restart(
                            RelaySettings(
                                useAcerolaRelay = relaySettings.useAcerolaRelay,
                                useIrohPublicNetwork = relaySettings.useIrohPublicNetwork,
                                customRelayUrls = relaySettings.customRelayUrls,
                            ),
                        )
                    }

                if (result.isSuccess) {
                    refreshLocalInfo()
                } else {
                    AcerolaLogger.w(
                        "SyncViewModel",
                        "Failed to restart p2p node: ${result.exceptionOrNull()?.message}",
                        LogSource.UI,
                    )
                }

                _uiState.update {
                    it.copy(
                        relaySettings =
                            it.relaySettings.copy(restarting = false, restartError = result.isFailure),
                    )
                }
            }
        }

        /** Fires `acerola/browse-cover/1` in parallel for each comic in the list — streams are
         *  cheap on a connection already pooled by `(peer, alpn)` (see `acerola-p2p`), so there's
         *  no need to serialize. `known_version` comes from the in-memory cache
         *  ([knownCoverVersions]); absent = first time this cover is seen in this session. */
        private fun fetchCoversFor(
            peerId: String,
            comics: List<ComicSummary>,
        ) {
            viewModelScope.launch(Dispatchers.IO) {
                val addr = p2pUseCase.getPairedPeers().find { it.id == peerId } ?: return@launch
                comics.forEach { comic ->
                    val key = coverKey(peerId, comic.comicName)
                    if (knownCoverVersions[key] == comic.coverVersion) return@forEach
                    p2pUseCase.browseCover(addr, comic.comicName, knownCoverVersions[key])
                }
            }
        }

        /** Only lists the peer's comics (name + count) — doesn't sync anything. The result
         *  arrives via [P2pEvent.LibraryBrowseResult]/[P2pEvent.LibraryBrowseError]. */
        private fun browseLibrary(peerId: String) {
            _uiState.update {
                it.copy(
                    browsingPeerId = peerId,
                    remoteLibrary = emptyList(),
                    remoteLibraryLoaded = false,
                    browseLibraryError = null,
                    browseLibraryErrorType = null,
                )
            }

            viewModelScope.launch(Dispatchers.IO) {
                val addr = p2pUseCase.getPairedPeers().find { it.id == peerId }
                if (addr == null) {
                    _uiState.update { it.copy(browsingPeerId = null) }
                    return@launch
                }
                p2pUseCase.browseLibrary(addr)
            }
        }

        /** Syncs a single comic (`acerola/sync-comic/1`) — reuses the SAME
         *  bookkeeping ([SYNC_KIND_FILES]/`syncingKeys`/[TransferLogEntry]) as the
         *  "Sync files" button, since both emit the same `sync:files:*` events
         *  (see `protocol::files::run_and_report_scoped` on the Rust side). */
        private fun syncComic(
            peerId: String,
            comicName: String,
        ) {
            val key = syncKey(peerId, SYNC_KIND_FILES)
            if (key in _uiState.value.syncingKeys) return

            _uiState.update {
                it.copy(
                    syncingKeys = it.syncingKeys + key,
                    browsingPeerId = null,
                    remoteLibrary = emptyList(),
                    remoteLibraryLoaded = false,
                )
            }

            viewModelScope.launch {
                delay(SYNC_IN_FLIGHT_TIMEOUT_MS)
                _uiState.update { it.copy(syncingKeys = it.syncingKeys - key) }
            }

            viewModelScope.launch(Dispatchers.IO) {
                val addr = p2pUseCase.getPairedPeers().find { it.id == peerId }
                if (addr == null) {
                    _uiState.update { it.copy(syncingKeys = it.syncingKeys - key) }
                    return@launch
                }
                // Comes from browsing the remote library (`RemoteLibrarySheet`) — the user can
                // only pick a comic they don't have yet, so it's always a pull.
                val started = p2pUseCase.syncComic(addr, comicName, SyncDirection.PULL)
                // User declined mobile data (see `MobileDataSyncGate`) — without this the spinner
                // stayed stuck until the 60s timeout waiting for an event that never arrives.
                if (!started) {
                    _uiState.update { it.copy(syncingKeys = it.syncingKeys - key) }
                }
            }
        }

        /** Unpairs a peer — removes it from trust and the native cache, and clears all local
         *  state tied to it (otherwise the peer disappears from the list but leaves garbage in
         *  [SyncUiState.lastSyncedByPeer]/[SyncUiState.lastSyncResultByPeer]/[SyncUiState.syncingKeys]). */
        private fun removePeer(peerId: String) {
            viewModelScope.launch(Dispatchers.IO) {
                p2pUseCase.removePairedPeer(peerId)
                // Also clears the local nickname — without this, re-pairing the same `peerId`
                // would "resurrect" the old nickname (same care as the Desktop side with
                // `SecureP2pStorage::remove_peer` for the persisted `device_info`).
                PeerNicknamePreference.setNickname(context, peerId, null)
                _uiState.update {
                    it.copy(
                        pairedPeers = it.pairedPeers.filterNot { peer -> peer.peerId == peerId },
                        connectedPeerIds = it.connectedPeerIds - peerId,
                        lastSyncedByPeer = it.lastSyncedByPeer - peerId,
                        lastSyncResultByPeer = it.lastSyncResultByPeer - peerId,
                        syncingKeys = it.syncingKeys.filterNot { key -> key.startsWith("$peerId:") }.toSet(),
                    )
                }
            }
        }

        /** Sets (or clears, with an empty string) a peer's local nickname — persisted via
         *  DataStore; [SyncUiState.pairedPeers] is updated by the collector of
         *  [PeerNicknamePreference.nicknamesFlow] in `init`, not here (single source of truth). */
        private fun renamePeer(
            peerId: String,
            nickname: String,
        ) {
            viewModelScope.launch(Dispatchers.IO) {
                PeerNicknamePreference.setNickname(context, peerId, nickname)
            }
        }

        /** Sets a custom local device alias — applies on the P2P node right away (next
         *  handshake already uses it) and persists it via DataStore so it survives a restart.
         *  Updates [SyncUiState.localDeviceName] optimistically since the FFI call is
         *  fire-and-forget (no synchronous confirmation from the Rust side). */
        private fun renameDevice(name: String) {
            val trimmed = name.trim()
            if (trimmed.isEmpty()) return

            AcerolaLogger.i("SyncViewModel", "Renaming local device to: $trimmed", LogSource.UI)
            _uiState.update { it.copy(localDeviceName = trimmed) }

            viewModelScope.launch(Dispatchers.IO) {
                p2pUseCase.setLocalDeviceName(trimmed)
                DeviceAliasPreference.saveAlias(context, trimmed)
            }
        }

        /** Just decodes the code and proposes confirmation — no network activity yet. */
        private fun proposeConnect(code: String) {
            val decoded = PairingCode.decode(code)
            if (decoded == null) {
                _uiState.update { it.copy(connectError = ConnectError.INVALID_CODE) }
                return
            }

            _uiState.update {
                it.copy(
                    pendingConnect = PendingConnect(peerId = decoded.id, deviceId = decoded.deviceId, addrs = decoded.addrs),
                    connectError = null,
                )
            }
        }

        private fun confirmConnect() {
            val pending = _uiState.value.pendingConnect ?: return
            AcerolaLogger.i("SyncViewModel", "Pairing with peer: ${pending.peerId}", LogSource.UI)

            _uiState.update { it.copy(pendingConnect = null, connecting = true, connectError = null) }

            viewModelScope.launch {
                // Start listening for the handshake event BEFORE firing connect() — otherwise,
                // if the handshake finishes too fast, we risk missing the event (SharedFlow
                // doesn't replay) and wrongly concluding it failed.
                val handshakeCompleted =
                    async {
                        withTimeoutOrNull(CONNECT_TIMEOUT_MS) {
                            p2pEventBus.events.first {
                                it is P2pEvent.HandshakeCompleted && it.peerId == pending.peerId
                            }
                            true
                        }
                    }

                val peerAddress = PeerAddress(id = pending.peerId, deviceId = pending.deviceId, addrs = pending.addrs)
                val started =
                    withContext(Dispatchers.IO) {
                        p2pUseCase.connect(peerAddress, HANDSHAKE_ALPN.toByteArray())
                    }

                // User declined mobile data (see `MobileDataSyncGate`) — nothing was fired,
                // and this isn't a connection failure (showing `CONNECTION_FAILED` here would be
                // misleading). Cancels the wait for the handshake instead of letting it run out the
                // 15s of `CONNECT_TIMEOUT_MS` for nothing.
                if (!started) {
                    handshakeCompleted.cancel()
                    _uiState.update { it.copy(connecting = false) }
                    return@launch
                }

                // `p2pUseCase.connect` is fire-and-forget over the FFI (no synchronous
                // success/error return) — the handshake event is the most precise signal we have
                // today that it actually worked. Without it within the timeout, we treat it as a failure.
                val succeeded = handshakeCompleted.await() ?: false

                _uiState.update {
                    it.copy(
                        connecting = false,
                        connectError = if (succeeded) null else ConnectError.CONNECTION_FAILED,
                    )
                }
                refreshLocalInfo()
            }
        }

        private fun triggerSync(
            peerId: String,
            alpn: String,
            kind: String,
        ) {
            val key = syncKey(peerId, kind)
            if (key in _uiState.value.syncingKeys) return

            _uiState.update { it.copy(syncingKeys = it.syncingKeys + key) }

            viewModelScope.launch {
                delay(SYNC_IN_FLIGHT_TIMEOUT_MS)
                _uiState.update { it.copy(syncingKeys = it.syncingKeys - key) }
            }

            viewModelScope.launch(Dispatchers.IO) {
                val addr = p2pUseCase.getPairedPeers().find { it.id == peerId }
                if (addr == null) {
                    _uiState.update { it.copy(syncingKeys = it.syncingKeys - key) }
                    return@launch
                }
                val started = p2pUseCase.connect(addr, alpn.toByteArray())
                // User declined mobile data (see `MobileDataSyncGate`) — without this the spinner
                // stayed stuck until the 60s timeout waiting for an event that never arrives.
                if (!started) {
                    _uiState.update { it.copy(syncingKeys = it.syncingKeys - key) }
                }
            }
        }

        private fun clearSyncing(
            peerId: String,
            kind: String,
        ) {
            val key = syncKey(peerId, kind)
            _uiState.update { it.copy(syncingKeys = it.syncingKeys - key) }
        }

        /**
         * [refreshLocalInfo] makes 5 blocking synchronous calls into Rust (`runtime.block_on`
         * on the native side — see the comment there) — firing it unconditionally for EVERY
         * event (including `network:latency`, which arrives periodically and falls into
         * `P2pEvent.Unknown`) competed for the same shared Tokio runtime that an active
         * sync-files session needs to make progress. We only call it explicitly on events that
         * actually change what this function fetches (peer paired/connected, session finished).
         */
        private suspend fun handleEvent(event: P2pEvent) {
            when (event) {
                is P2pEvent.PeerTrustedFirstTime -> {
                    _uiState.update { it.copy(trustedPeerDialogPeerId = event.peerId) }
                    refreshLocalInfo()
                }
                is P2pEvent.HandshakeCompleted -> refreshLocalInfo()

                is P2pEvent.HistorySyncStarted ->
                    pushLog(SYNC_KIND_HISTORY, "started", LogState.IN_PROGRESS)
                is P2pEvent.HistorySyncComplete -> {
                    clearSyncing(event.peerId, SYNC_KIND_HISTORY)
                    recordSyncResult(event.peerId, SYNC_KIND_HISTORY, "complete", null)
                    pushLog(SYNC_KIND_HISTORY, "complete", LogState.SUCCESS)
                    refreshLocalInfo()
                }
                is P2pEvent.HistorySyncError -> {
                    clearSyncing(event.peerId, SYNC_KIND_HISTORY)
                    recordSyncResult(event.peerId, SYNC_KIND_HISTORY, "error", event.message, event.error)
                    pushLog(SYNC_KIND_HISTORY, "error", LogState.ERROR, message = event.message)
                    refreshLocalInfo()
                }

                is P2pEvent.FileSyncManifestExchanged ->
                    pushLog(SYNC_KIND_FILES, "started", LogState.IN_PROGRESS)
                is P2pEvent.FileSyncProgress -> {
                    pushLog(
                        SYNC_KIND_FILES,
                        "progress",
                        LogState.IN_PROGRESS,
                        comicName = event.comicName,
                        chapter = event.chapter,
                    )
                }
                is P2pEvent.FileSyncChapterFailed -> {
                    // Empty comicName/chapter is the sentinel for a whole-SESSION failure (see
                    // `protocol/files/mod.rs::run_and_report`), not a single chapter — only
                    // that case actually ends the session.
                    if (event.comicName.isEmpty() && event.chapter.isEmpty()) {
                        clearSyncing(event.peerId, SYNC_KIND_FILES)
                        // `event.error` already comes classified as `SyncProtocolError` (ADT) by
                        // `P2pEventBus` — the ViewModel just propagates the typed value, with no `when`
                        // over a raw `code` nor a `Context` to resolve any text.
                        // `event.reason` stays raw/in English, only for the persisted history
                        // (`syncHistoryLogUseCase.record`, inside `recordSyncResult`).
                        recordSyncResult(event.peerId, SYNC_KIND_FILES, "error", event.reason, event.error)
                        refreshLocalInfo()
                    }
                    pushLog(
                        SYNC_KIND_FILES,
                        "chapterFailed",
                        LogState.ERROR,
                        comicName = event.comicName,
                        chapter = event.chapter,
                    )
                }
                // Counterpart of FileSyncChapterFailed for extra items (cover/banner/
                // ComicInfo.xml) — reuses the same "chapterFailed" status (never part of the
                // started/progress/complete transition, see isSessionStatus) to land in the
                // same permanent log entry, with `chapter` carrying the extra's `kind`
                // (there's no dedicated field in TransferLogEntry, and one isn't worth it just
                // for this). Before this change, sync:files:extra_failed fell into
                // P2pEvent.Unknown and these failures were completely invisible.
                is P2pEvent.FileSyncExtraFailed ->
                    pushLog(
                        SYNC_KIND_FILES,
                        "chapterFailed",
                        LogState.ERROR,
                        comicName = event.comicName,
                        chapter = event.kind,
                    )
                // Mirrors FileSyncChapterComplete (also not logged individually) — each
                // item's success is already implied when the session ends in
                // FileSyncComplete; only an item's FAILURE needs its own row.
                is P2pEvent.FileSyncExtraComplete -> Unit
                is P2pEvent.FileSyncComplete -> {
                    clearSyncing(event.peerId, SYNC_KIND_FILES)
                    recordSyncResult(event.peerId, SYNC_KIND_FILES, "complete", null)
                    pushLog(SYNC_KIND_FILES, "complete", LogState.SUCCESS)
                    refreshLocalInfo()
                }

                is P2pEvent.LibraryBrowseResult ->
                    if (event.peerId == _uiState.value.browsingPeerId) {
                        _uiState.update { it.copy(remoteLibrary = event.comics, remoteLibraryLoaded = true) }
                        fetchCoversFor(event.peerId, event.comics)
                    }
                is P2pEvent.LibraryBrowseError ->
                    if (event.peerId == _uiState.value.browsingPeerId) {
                        _uiState.update {
                            it.copy(browseLibraryError = event.message, browseLibraryErrorType = event.error)
                        }
                    }

                is P2pEvent.CoverBrowseResult -> {
                    val key = coverKey(event.peerId, event.comicName)
                    event.coverVersion?.let { knownCoverVersions[key] = it }
                    val path = event.path
                    if (event.status == "changed" && path != null) {
                        _uiState.update { it.copy(remoteCoverPaths = it.remoteCoverPaths + (key to path)) }
                    }
                }
                is P2pEvent.CoverBrowseError -> Unit // best-effort — the list just stays without a thumb for this item

                else -> Unit
            }
        }

        private fun isSessionStatus(status: String) = status == "started" || status == "progress" || status == "complete" || status == "error"

        private fun isTerminalStatus(status: String) = status == "complete" || status == "error"

        private fun isOpenStatus(status: String) = status == "started" || status == "progress"

        private fun replaceInFlightEntry(
            log: List<TransferLogEntry>,
            index: Int,
            status: String,
            state: LogState,
            comicName: String?,
            chapter: String?,
            message: String?,
        ) = log.mapIndexed { position, entry ->
            if (position != index) {
                entry
            } else {
                entry.copy(
                    status = status,
                    state = state,
                    comicName = comicName ?: entry.comicName,
                    chapter = chapter ?: entry.chapter,
                    message = message,
                    timestamp = System.currentTimeMillis(),
                )
            }
        }

        /** Updates the in-progress row for `kind` in place, instead of stacking a new one. */
        private fun updateInFlightLog(
            current: SyncUiState,
            kind: String,
            index: Int,
            status: String,
            state: LogState,
            comicName: String?,
            chapter: String?,
            message: String?,
        ): SyncUiState {
            val updatedLog = replaceInFlightEntry(current.transferLog, index, status, state, comicName, chapter, message)
            val inFlight =
                if (isTerminalStatus(status)) current.inFlightLogEntryByKind - kind else current.inFlightLogEntryByKind
            return current.copy(transferLog = updatedLog, inFlightLogEntryByKind = inFlight)
        }

        /** Stacks a new row — only reached when there's no in-progress session for `kind`
         *  (first event) or the status isn't part of the transition (`"chapterFailed"`). */
        private fun appendNewLog(
            current: SyncUiState,
            kind: String,
            status: String,
            state: LogState,
            comicName: String?,
            chapter: String?,
            message: String?,
        ): SyncUiState {
            val entry =
                TransferLogEntry(
                    id = nextLogId.getAndIncrement(),
                    kind = kind,
                    status = status,
                    state = state,
                    comicName = comicName,
                    chapter = chapter,
                    message = message,
                )
            val inFlight =
                if (isOpenStatus(status)) current.inFlightLogEntryByKind + (kind to entry.id) else current.inFlightLogEntryByKind
            return current.copy(
                transferLog = (listOf(entry) + current.transferLog).take(MAX_LOG_ENTRIES),
                inFlightLogEntryByKind = inFlight,
            )
        }

        /**
         * A sync session has ONE row in the log, which transitions state (started ->
         * progress -> complete/error) instead of stacking a new row per event — without this,
         * "started" stayed forever as a separate row with a spinning spinner next to the
         * "complete" row that had already resolved the same session. `"chapterFailed"` (failure
         * of ONE chapter, not the whole session — see the comment in `handleEvent`) is left out
         * of this transition on purpose: each failing chapter is its own permanent event, without
         * interrupting the session's aggregated progress row.
         */
        private fun pushLog(
            kind: String,
            status: String,
            state: LogState,
            comicName: String? = null,
            chapter: String? = null,
            message: String? = null,
        ) {
            _uiState.update { current ->
                val trackedId = current.inFlightLogEntryByKind[kind].takeIf { isSessionStatus(status) }
                val index = trackedId?.let { id -> current.transferLog.indexOfFirst { it.id == id } }?.takeIf { it >= 0 }

                if (index != null) {
                    updateInFlightLog(current, kind, index, status, state, comicName, chapter, message)
                } else {
                    appendNewLog(current, kind, status, state, comicName, chapter, message)
                }
            }
        }

        /** Calls into the Rust node block the thread (JNI + `runtime.block_on` on that side). */
        private suspend fun refreshLocalInfo() =
            withContext(Dispatchers.IO) {
                val localAddress = p2pUseCase.getLocalAddress()
                val pairingCode = PairingCode.encode(localAddress.id, localAddress.deviceId, localAddress.addrs)
                // Device name comes from getPairedPeers() (which already resolves via the
                // persistent known_peers() on the native side) -- getConnectedPeersWithInfo()
                // is now only used to know WHO is connected right now (connectedPeerIds), not
                // for the name: that list only has data during the few seconds the
                // handshake connection is actually open.
                val connectedPeers = p2pUseCase.getConnectedPeersWithInfo()
                val paired =
                    p2pUseCase.getPairedPeers().map {
                        PairedPeer(peerId = it.id, deviceName = it.deviceName, nickname = currentNicknames[it.id])
                    }
                val localId = p2pUseCase.getLocalId()
                val mode = p2pUseCase.getMode()

                _uiState.update {
                    it.copy(
                        localId = localId,
                        pairingCode = pairingCode,
                        mode = mode,
                        pairedPeers = paired,
                        connectedPeerIds = connectedPeers.map { peer -> peer.peerId }.toSet(),
                    )
                }
            }
    }
