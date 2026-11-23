package br.acerola.comic.service.network

import android.content.Context
import br.acerola.comic.config.network.isOnCellularConnection
import br.acerola.comic.config.preference.MobileDataSyncPreference
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Single gate that ANY P2P call that actually moves data must pass through before
 * moving on (see [P2pUseCase.connect]/[P2pUseCase.syncComic]/[P2pUseCase.syncHistoryEntry],
 * the three places where [ensureAllowed] is called). `@Singleton` — not tied to any screen — on
 * purpose: Home, History, Comic, Remote library and the Sync screen trigger P2P through
 * completely different paths (see the use cases in `br.acerola.comic.usecase.network`), and
 * no single screen ViewModel sees all of them. Putting the check inside [P2pUseCase]
 * itself (instead of in each ViewModel/screen) guarantees that a new screen that starts calling
 * `connect`/`syncComic`/`syncHistoryEntry` tomorrow is already covered, with nothing to remember.
 *
 * [awaitingConfirmation] is observed by a single dialog mounted at the app root
 * (`br.acerola.comic.common.activity.BaseActivity`, via `MobileDataSyncViewModel`) — the
 * confirmation appears on top of any screen, no matter where the action was triggered from.
 */
@Singleton
class MobileDataSyncGate
    @Inject
    constructor(
        @param:ApplicationContext private val context: Context,
    ) {
        private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)

        private val _awaitingConfirmation = MutableStateFlow(false)
        val awaitingConfirmation: StateFlow<Boolean> = _awaitingConfirmation.asStateFlow()

        /** No replay: only the answer to one open request at a time matters. If two actions
         *  ask for confirmation at the same time (e.g. "Sync all" triggers history and
         *  files together), a single user answer releases both — there is no reason to
         *  stack one identical dialog behind another. */
        private val answers = MutableSharedFlow<Boolean>(extraBufferCapacity = 1)

        /** `true` releases right away; `false` means "user cancelled", and the caller must
         *  give up on the action (see the `if (!ensureAllowed()) return` in [P2pUseCase]). */
        suspend fun ensureAllowed(): Boolean {
            if (MobileDataSyncPreference.alwaysAllowFlow(context).first() || !isOnCellularConnection(context)) {
                return true
            }

            _awaitingConfirmation.value = true
            val allowed = answers.first()
            _awaitingConfirmation.value = false
            return allowed
        }

        fun confirm(remember: Boolean) {
            if (remember) {
                scope.launch { MobileDataSyncPreference.setAlwaysAllow(context, true) }
            }
            answers.tryEmit(true)
        }

        fun cancel() {
            answers.tryEmit(false)
        }
    }
