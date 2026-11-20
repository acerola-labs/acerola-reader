package br.acerola.comic.module.main.transferlog

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import br.acerola.comic.module.main.sync.state.LogState
import br.acerola.comic.module.main.sync.state.TransferLogEntry
import br.acerola.comic.module.main.transferlog.state.TransferLogUiState
import br.acerola.comic.usecase.network.SyncHistoryLogUseCase
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

private const val MAX_ENTRIES = 50

/**
 * Dedicated ViewModel for the full-screen history ([TransferLogActivity]) — lightweight on purpose,
 * it only depends on [SyncHistoryLogUseCase]. Unlike
 * [br.acerola.comic.module.main.sync.SyncViewModel] (which brings up peers, relay and the whole P2P
 * module), this screen only lists already persisted sessions (`complete`/`error`); live sessions
 * stay visible through the per-peer spinners on the Network tab
 * ([br.acerola.comic.module.main.sync.PeerRow]).
 */
@HiltViewModel
class TransferLogViewModel
    @Inject
    constructor(
        private val syncHistoryLogUseCase: SyncHistoryLogUseCase,
    ) : ViewModel() {
        private val _uiState = MutableStateFlow(TransferLogUiState())
        val uiState: StateFlow<TransferLogUiState> = _uiState.asStateFlow()

        init {
            refresh()
        }

        fun refresh() {
            viewModelScope.launch(Dispatchers.IO) {
                val rows = syncHistoryLogUseCase.findRecent(MAX_ENTRIES)
                val entries =
                    rows.map { row ->
                        TransferLogEntry(
                            id = row.id,
                            kind = row.kind,
                            status = row.status,
                            state = if (row.status == "complete") LogState.SUCCESS else LogState.ERROR,
                            message = row.message,
                            timestamp = row.createdAt,
                        )
                    }
                _uiState.update { it.copy(entries = entries, loaded = true) }
            }
        }

        fun clear() {
            viewModelScope.launch(Dispatchers.IO) {
                syncHistoryLogUseCase.clearAll()
                _uiState.update { it.copy(entries = emptyList()) }
            }
        }
    }
