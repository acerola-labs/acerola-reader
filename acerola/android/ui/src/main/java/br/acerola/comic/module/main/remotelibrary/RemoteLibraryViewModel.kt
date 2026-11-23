package br.acerola.comic.module.main.remotelibrary

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import br.acerola.comic.error.UserMessage
import br.acerola.comic.logging.AcerolaLogger
import br.acerola.comic.logging.LogSource
import br.acerola.comic.module.main.remotelibrary.state.RemoteLibraryUiState
import br.acerola.comic.service.SyncDirection
import br.acerola.comic.service.network.ComicSummary
import br.acerola.comic.service.network.P2pEvent
import br.acerola.comic.service.network.P2pEventBus
import br.acerola.comic.type.UiText
import br.acerola.comic.ui.R
import br.acerola.comic.usecase.network.P2pUseCase
import br.acerola.comic.usecase.network.SyncComicWithPeerUseCase
import br.acerola.comic.usecase.network.SyncWithPeerResult
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import java.util.concurrent.ConcurrentHashMap
import javax.inject.Inject

/**
 * Remote library of a SINGLE peer (picked before opening this screen, via `PeerPickerSheet`) —
 * a lean version of what [br.acerola.comic.module.main.sync.SyncViewModel] already does for
 * `browseLibrary`/`fetchCoversFor`/`syncComic`, without the rest of the pairing/QR/log state that
 * doesn't make sense here.
 */
@HiltViewModel
class RemoteLibraryViewModel
    @Inject
    constructor(
        private val p2pUseCase: P2pUseCase,
        private val p2pEventBus: P2pEventBus,
        private val syncComicWithPeerUseCase: SyncComicWithPeerUseCase,
    ) : ViewModel() {
        private val _uiState = MutableStateFlow(RemoteLibraryUiState())
        val uiState: StateFlow<RemoteLibraryUiState> = _uiState.asStateFlow()

        private val _uiEvents = Channel<UserMessage>(capacity = Channel.BUFFERED)
        val uiEvents: Flow<UserMessage> = _uiEvents.receiveAsFlow()

        private var peerId: String? = null

        /** `comicName` -> latest cover version confirmed in this session — never downloads the same
         *  version twice (same idea as `knownCoverVersions` in
         *  [br.acerola.comic.module.main.sync.SyncViewModel], but scoped to a SINGLE peer, so the
         *  key is just the comic name). */
        private val knownCoverVersions = ConcurrentHashMap<String, Long>()

        /** A real network may never deliver `FileSyncComplete`/`FileSyncChapterFailed` (peer vanished,
         *  or a cover/banner blob fetch hangs without ever resolving — see
         *  `exchange::receive_extra_from_blob` on the Rust side) — without this the card would keep
         *  the loading spinner forever, requiring the app to be closed and reopened. Same mechanism
         *  [br.acerola.comic.module.main.sync.SyncViewModel.syncComic] already uses
         *  (`SYNC_IN_FLIGHT_TIMEOUT_MS`), except this screen had none. */
        private var syncTimeoutJob: Job? = null

        init {
            viewModelScope.launch {
                p2pEventBus.events.collect(::handleEvent)
            }
        }

        /** Called by the `Screen` as soon as `peerId` is known — idempotent so recomposition
         *  doesn't fire `browseLibrary` again for nothing. */
        fun init(
            peerId: String,
            peerDisplayName: String,
        ) {
            if (this.peerId == peerId) return
            this.peerId = peerId
            knownCoverVersions.clear()
            _uiState.value = RemoteLibraryUiState(peerDisplayName = peerDisplayName)
            browseLibrary(peerId)
        }

        /** "Try again" of the error screen: asks the peer for its library once more. */
        fun retry() {
            val id = peerId ?: return
            _uiState.update { it.copy(loaded = false, errorMessage = null, errorType = null) }
            browseLibrary(id)
        }

        private fun browseLibrary(peerId: String) {
            viewModelScope.launch(Dispatchers.IO) {
                val addr = p2pUseCase.getPairedPeers().find { it.id == peerId }
                if (addr == null) {
                    _uiState.update { it.copy(loaded = true) }
                    _uiEvents.send(UserMessage.Raw(UiText.StringResource(R.string.error_sync_comic_peer_not_paired)))
                    return@launch
                }
                p2pUseCase.browseLibrary(addr)
            }
        }

        fun syncComic(comicName: String) {
            val peerId = peerId ?: return
            if (_uiState.value.syncingComicName != null) {
                // Tap on the card while it only shows the loading spinner — with no real
                // cancellation available in the P2P flow today, the least we can do is explain why
                // the tap did nothing, instead of silently ignoring it.
                viewModelScope.launch {
                    _uiEvents.send(UserMessage.Raw(UiText.StringResource(R.string.message_sync_comic_already_syncing)))
                }
                return
            }

            AcerolaLogger.audit(
                TAG,
                "Pulling comic from remote library",
                LogSource.VIEWMODEL,
                mapOf("peerId" to peerId, "comicName" to comicName),
            )

            viewModelScope.launch(Dispatchers.IO) {
                // Comes from browsing the remote library — the user can only pick a
                // comic they don't have yet, so it's always a pull.
                when (syncComicWithPeerUseCase(peerId, comicName, SyncDirection.PULL)) {
                    SyncWithPeerResult.NOT_PAIRED ->
                        _uiEvents.send(UserMessage.Raw(UiText.StringResource(R.string.error_sync_comic_peer_not_paired)))
                    SyncWithPeerResult.DECLINED_MOBILE_DATA -> Unit
                    SyncWithPeerResult.STARTED -> {
                        _uiState.update { it.copy(syncingComicName = comicName) }

                        syncTimeoutJob?.cancel()
                        syncTimeoutJob =
                            viewModelScope.launch {
                                delay(SYNC_COMIC_TIMEOUT_MS)
                                _uiState.update { it.copy(syncingComicName = null) }
                                _uiEvents.send(UserMessage.Raw(UiText.StringResource(R.string.error_sync_comic_timeout)))
                            }
                    }
                }
            }
        }

        private fun clearSyncTimeout() {
            syncTimeoutJob?.cancel()
            syncTimeoutJob = null
        }

        /** Fires `acerola/browse-cover/1` in parallel for each comic in the list — streams are
         *  cheap on a connection already pooled by `(peer, alpn)`, so there's no need to
         *  serialize (same logic as [br.acerola.comic.module.main.sync.SyncViewModel]). */
        private fun fetchCoversFor(comics: List<ComicSummary>) {
            val peerId = peerId ?: return
            viewModelScope.launch(Dispatchers.IO) {
                val addr = p2pUseCase.getPairedPeers().find { it.id == peerId } ?: return@launch
                comics.forEach { comic ->
                    if (knownCoverVersions[comic.comicName] == comic.coverVersion) return@forEach
                    p2pUseCase.browseCover(addr, comic.comicName, knownCoverVersions[comic.comicName])
                }
            }
        }

        private fun handleEvent(event: P2pEvent) {
            val currentPeerId = peerId ?: return
            when (event) {
                is P2pEvent.LibraryBrowseResult ->
                    if (event.peerId == currentPeerId) {
                        _uiState.update { it.copy(comics = event.comics, loaded = true, errorMessage = null, errorType = null) }
                        fetchCoversFor(event.comics)
                    }
                is P2pEvent.LibraryBrowseError ->
                    if (event.peerId == currentPeerId) {
                        _uiState.update { it.copy(loaded = true, errorMessage = event.message, errorType = event.error) }
                    }

                is P2pEvent.CoverBrowseResult ->
                    if (event.peerId == currentPeerId) {
                        event.coverVersion?.let { knownCoverVersions[event.comicName] = it }
                        val path = event.path
                        if (event.status == "changed" && path != null) {
                            _uiState.update { it.copy(coverPaths = it.coverPaths + (event.comicName to path)) }
                        }
                    }
                is P2pEvent.CoverBrowseError -> Unit // best-effort — the item just stays without a thumb

                is P2pEvent.FileSyncComplete ->
                    if (event.peerId == currentPeerId) {
                        clearSyncTimeout()
                        _uiState.update { it.copy(syncingComicName = null) }
                    }
                is P2pEvent.FileSyncChapterFailed ->
                    // Empty comicName/chapter = failure of the whole session, not of one chapter
                    // (see protocol::files::run_and_report_scoped on the Rust side).
                    if (event.peerId == currentPeerId && event.comicName.isEmpty() && event.chapter.isEmpty()) {
                        clearSyncTimeout()
                        _uiState.update { it.copy(syncingComicName = null) }
                    }

                else -> Unit
            }
        }

        companion object {
            private const val TAG = "RemoteLibraryViewModel"
            private const val SYNC_COMIC_TIMEOUT_MS = 60_000L
        }
    }
