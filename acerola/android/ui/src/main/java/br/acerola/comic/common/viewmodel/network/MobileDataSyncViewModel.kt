package br.acerola.comic.common.viewmodel.network

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import br.acerola.comic.config.preference.MobileDataSyncPreference
import br.acerola.comic.service.network.MobileDataSyncGate
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

/**
 * Bridge between [MobileDataSyncGate] (singleton outside any ViewModel) and the Compose UI —
 * instantiated in two places for different reasons, which is fine since both just mirror the
 * same singleton/DataStore underneath:
 * - `BaseActivity` (app root): observes [awaitingConfirmation] to show the global dialog,
 *   no matter which screen is open.
 * - Config screen: reads/writes [allowMobileDataSync] for the "Sync using mobile
 *   data" switch, which exists to revert the "always allow" choice made once in the dialog.
 */
@HiltViewModel
class MobileDataSyncViewModel
    @Inject
    constructor(
        private val mobileDataSyncGate: MobileDataSyncGate,
        @param:ApplicationContext private val context: Context,
    ) : ViewModel() {
        val awaitingConfirmation: StateFlow<Boolean> = mobileDataSyncGate.awaitingConfirmation

        val allowMobileDataSync: StateFlow<Boolean> =
            MobileDataSyncPreference
                .alwaysAllowFlow(context)
                .stateIn(viewModelScope, SharingStarted.Eagerly, false)

        fun confirm(remember: Boolean) = mobileDataSyncGate.confirm(remember)

        fun cancel() = mobileDataSyncGate.cancel()

        fun setAllowMobileDataSync(value: Boolean) {
            viewModelScope.launch { MobileDataSyncPreference.setAlwaysAllow(context, value) }
        }
    }
