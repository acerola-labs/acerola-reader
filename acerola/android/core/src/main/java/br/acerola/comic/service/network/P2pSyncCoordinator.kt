package br.acerola.comic.service.network

import android.content.Context
import br.acerola.comic.core.R
import br.acerola.comic.service.P2pSyncForegroundService
import br.acerola.comic.usecase.network.SyncHistoryLogUseCase
import br.acerola.comic.util.notification.NotificationHelper
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject
import javax.inject.Singleton

/** Mirror `SYNC_KIND_HISTORY`/`SYNC_KIND_FILES` from `SyncViewModel` (`:ui` module, not
 *  accessible from here) — they must keep matching those values AND the `kind` stored
 *  by `SyncHistoryLogUseCase`/read back in `SyncViewModel.loadPersistedLog`. */
private const val SYNC_KIND_HISTORY = "history"
private const val SYNC_KIND_FILES = "files"

/**
 * Application-scoped counterpart of what `SyncViewModel` used to do on its own: persists the
 * result of a P2P session ([SyncHistoryLogUseCase]), fires the completion/error
 * notification, and starts/stops [P2pSyncForegroundService] — all of it regardless of
 * any screen being open.
 *
 * Before, these three responsibilities lived inside `SyncViewModel` (`hiltViewModel()`
 * scoped to the Sync screen's `NavBackStackEntry`). Since [P2pEventBus] is a `SharedFlow` WITHOUT
 * replay, an event emitted with no active collector is lost forever — it was enough for the user to
 * trigger a sync and press "back" before it finished for `SyncViewModel` to be destroyed,
 * the bus subscription to die with it, and the result (including the notification) to never reach
 * anywhere. The same hole existed for syncs triggered from the History/
 * Comic/Remote library screens — only `SyncViewModel` persisted/notified, the others only
 * updated their own local UI state.
 *
 * Instantiated only once (`@Singleton`, see `NetworkCaseModule.provideP2pService`) as soon
 * as the P2P subsystem exists — never tied to any ViewModel's lifecycle.
 * `SyncViewModel` keeps collecting the same [P2pEventBus] for its OWN screen's UI state
 * (in-memory log, button spinners); only persistence/notification/foreground service
 * moved from there to here.
 */
@Singleton
class P2pSyncCoordinator
    @Inject
    constructor(
        @param:ApplicationContext private val context: Context,
        private val p2pEventBus: P2pEventBus,
        private val syncHistoryLogUseCase: SyncHistoryLogUseCase,
        private val notificationHelper: NotificationHelper,
    ) {
        private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)

        /** `"$peerId:$kind"` of in-progress sessions — only controls starting/stopping the
         *  foreground service, no finer granularity needed. */
        private val activeSessions = MutableStateFlow<Set<String>>(emptySet())

        init {
            scope.launch { p2pEventBus.events.collect(::handleEvent) }

            // Unlike what `SyncViewModel` did (tied only to syncs triggered FROM the
            // Sync screen), this reacts to ANY session in the whole app — syncs triggered from
            // Home/History/Comic/Remote library also get the Doze/App Standby
            // exemption, not just those going through the Sync screen.
            scope.launch {
                activeSessions
                    .map { it.isNotEmpty() }
                    .distinctUntilChanged()
                    .collect { active ->
                        if (active) P2pSyncForegroundService.start(context) else P2pSyncForegroundService.stop(context)
                    }
            }
        }

        private fun sessionKey(
            peerId: String,
            kind: String,
        ) = "$peerId:$kind"

        private fun startSession(
            peerId: String,
            kind: String,
        ) {
            activeSessions.update { it + sessionKey(peerId, kind) }
        }

        private fun endSession(
            peerId: String,
            kind: String,
        ) {
            activeSessions.update { it - sessionKey(peerId, kind) }
        }

        private suspend fun handleEvent(event: P2pEvent) {
            when (event) {
                is P2pEvent.HistorySyncStarted -> startSession(event.peerId, SYNC_KIND_HISTORY)
                is P2pEvent.HistorySyncComplete -> {
                    endSession(event.peerId, SYNC_KIND_HISTORY)
                    syncHistoryLogUseCase.record(event.peerId, SYNC_KIND_HISTORY, "complete", null)
                    notifyHistorySyncComplete(event)
                }
                is P2pEvent.HistorySyncError -> {
                    endSession(event.peerId, SYNC_KIND_HISTORY)
                    syncHistoryLogUseCase.record(event.peerId, SYNC_KIND_HISTORY, "error", event.message)
                    notifyHistorySyncFailed(event)
                }

                is P2pEvent.FileSyncManifestExchanged -> startSession(event.peerId, SYNC_KIND_FILES)
                is P2pEvent.FileSyncChapterFailed ->
                    // Empty comicName/chapter = failure of the WHOLE session, not of a chapter
                    // (see protocol/files/mod.rs::run_and_report on the Rust side) — only this case
                    // ends the session.
                    if (event.comicName.isEmpty() && event.chapter.isEmpty()) {
                        endSession(event.peerId, SYNC_KIND_FILES)
                        syncHistoryLogUseCase.record(event.peerId, SYNC_KIND_FILES, "error", event.reason)
                        notifyFileSyncFailed(event)
                    }
                is P2pEvent.FileSyncComplete -> {
                    endSession(event.peerId, SYNC_KIND_FILES)
                    syncHistoryLogUseCase.record(event.peerId, SYNC_KIND_FILES, "complete", null)
                    notifyFileSyncComplete(event)
                }

                else -> Unit
            }
        }

        private fun notifyFileSyncComplete(event: P2pEvent.FileSyncComplete) {
            val base =
                if (event.failedCount > 0) {
                    context.getString(
                        R.string.notification_sync_files_complete_content_with_failures,
                        event.receivedCount,
                        event.sentCount,
                        event.failedCount,
                    )
                } else {
                    context.getString(
                        R.string.notification_sync_files_complete_content,
                        event.receivedCount,
                        event.sentCount,
                    )
                }
            // No dedicated toast/notification for this — a per-session total appended to the same
            // notification that was already going to appear, instead of one notification per conflicting chapter.
            val content =
                if (event.conflictsCount > 0) {
                    base +
                        context.getString(
                            R.string.notification_sync_files_complete_conflicts_suffix,
                            event.conflictsCount,
                        )
                } else {
                    base
                }
            notificationHelper.showFinishedNotification(
                title = context.getString(R.string.notification_sync_files_complete_title),
                content = content,
                notificationId = NotificationHelper.P2P_FILE_SYNC_RESULT_NOTIFICATION_ID,
            )
        }

        private fun notifyFileSyncFailed(event: P2pEvent.FileSyncChapterFailed) {
            notificationHelper.showFinishedNotification(
                title = context.getString(R.string.notification_sync_files_error_title),
                content = event.error?.uiMessage?.asString(context) ?: event.reason,
                notificationId = NotificationHelper.P2P_FILE_SYNC_RESULT_NOTIFICATION_ID,
            )
        }

        private fun notifyHistorySyncComplete(event: P2pEvent.HistorySyncComplete) {
            notificationHelper.showFinishedNotification(
                title = context.getString(R.string.notification_sync_history_complete_title),
                content =
                    context.getString(
                        R.string.notification_sync_history_complete_content,
                        event.progressApplied,
                        event.chaptersReadApplied,
                    ),
                notificationId = NotificationHelper.P2P_FILE_SYNC_RESULT_NOTIFICATION_ID,
            )
        }

        private fun notifyHistorySyncFailed(event: P2pEvent.HistorySyncError) {
            notificationHelper.showFinishedNotification(
                title = context.getString(R.string.notification_sync_history_error_title),
                content = event.error?.uiMessage?.asString(context) ?: event.message,
                notificationId = NotificationHelper.P2P_FILE_SYNC_RESULT_NOTIFICATION_ID,
            )
        }
    }
