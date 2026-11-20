package br.acerola.comic.module.main.sync

import android.content.Context
import br.acerola.comic.MainDispatcherRule
import br.acerola.comic.config.preference.DeviceAliasPreference
import br.acerola.comic.config.preference.PeerNicknamePreference
import br.acerola.comic.config.preference.RelayPreference
import br.acerola.comic.module.main.sync.state.ConnectError
import br.acerola.comic.module.main.sync.state.PendingConnect
import br.acerola.comic.module.main.sync.state.SyncAction
import br.acerola.comic.module.main.sync.state.SyncUiState
import br.acerola.comic.service.NetworkMode
import br.acerola.comic.service.PeerAddress
import br.acerola.comic.service.network.P2pEventBus
import br.acerola.comic.usecase.network.P2pUseCase
import br.acerola.comic.usecase.network.SyncHistoryLogUseCase
import com.google.common.truth.Truth.assertThat
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkObject
import io.mockk.unmockkObject
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner

/**
 * Covers how [SyncViewModel] reacts to a `P2pUseCase.connect` declined by the
 * `MobileDataSyncGate` (user cancelled the global mobile data dialog) — without this, the
 * "syncing"/"connecting" spinner stayed stuck waiting for an event that never arrives
 * (60s timeout for `syncingKeys`, or the 15s `CONNECT_TIMEOUT_MS` for pairing, both
 * replaced by an immediate reaction to the returned `false`). The gate itself is already covered by
 * `P2pUseCaseTest`/`MobileDataSyncGateTest` (`core` module) — here only the UI reaction.
 *
 * Runs under Robolectric (same reason as [br.acerola.comic.module.comic.ComicViewModelTest]):
 * `refreshLocalInfo()` in `init` calls `PairingCode.encode`, which uses the real
 * `android.util.Base64`.
 */
@OptIn(ExperimentalCoroutinesApi::class)
@RunWith(RobolectricTestRunner::class)
class SyncViewModelTest {
    @get:Rule
    val coroutineRule = MainDispatcherRule()

    private val context = mockk<Context>(relaxed = true)
    private val p2pUseCase = mockk<P2pUseCase>(relaxed = true)
    private val syncHistoryLogUseCase = mockk<SyncHistoryLogUseCase>(relaxed = true)
    private val p2pEventBus = P2pEventBus()

    private val pairedPeer = PeerAddress(id = "peer-1", deviceId = null, addrs = byteArrayOf(), deviceName = "Peer 1")

    @Before
    fun setup() {
        mockkObject(RelayPreference)
        mockkObject(DeviceAliasPreference)
        mockkObject(PeerNicknamePreference)

        every { RelayPreference.relaySettingsFlow(any()) } returns flowOf(RelayPreference.RelaySettings())
        every { DeviceAliasPreference.deviceAliasFlow(any()) } returns flowOf(null)
        every { PeerNicknamePreference.nicknamesFlow(any()) } returns flowOf(emptyMap())

        every { p2pUseCase.getLocalAddress() } returns pairedPeer.copy(id = "local-id", deviceName = null)
        every { p2pUseCase.getLocalId() } returns "local-id"
        every { p2pUseCase.getMode() } returns NetworkMode.LOCAL
        every { p2pUseCase.getPairedPeers() } returns listOf(pairedPeer)
        every { p2pUseCase.getConnectedPeersWithInfo() } returns emptyList()
        every { p2pUseCase.hasIrohServicesTicket() } returns false
        coEvery { syncHistoryLogUseCase.findRecent(any()) } returns emptyList()
    }

    @After
    fun tearDown() {
        unmockkObject(RelayPreference, DeviceAliasPreference, PeerNicknamePreference)
    }

    private fun createViewModel() = SyncViewModel(p2pUseCase, p2pEventBus, syncHistoryLogUseCase, context)

    /** A real `Thread.sleep` (not `delay()` — the work being awaited runs on the real
     *  `Dispatchers.IO`, its own thread pool, unaffected by this test's `TestDispatcher`
     *  virtual time) — it only exists to give the matching `viewModelScope.launch
     *  (Dispatchers.IO)` time to finish before the assertion. */
    private fun awaitState(
        viewModel: SyncViewModel,
        timeoutMs: Long = 2000,
        predicate: (SyncUiState) -> Boolean,
    ): SyncUiState {
        val deadline = System.currentTimeMillis() + timeoutMs
        while (System.currentTimeMillis() < deadline) {
            val state = viewModel.uiState.value
            if (predicate(state)) return state
            Thread.sleep(20)
        }
        return viewModel.uiState.value
    }

    @Test
    fun `triggerSync clears the spinner immediately when the mobile-data gate declines`() =
        runTest {
            coEvery { p2pUseCase.connect(any(), any()) } returns false
            val viewModel = createViewModel()

            viewModel.onAction(SyncAction.SyncHistory(peerId = "peer-1"))

            // No "contains the key right after onAction" check: that races against the very
            // `viewModelScope.launch(Dispatchers.IO)` that decides to remove the key — in CI (slower/
            // more contended), that IO may finish before the assertion runs, making the
            // test fail for a reason that isn't a bug at all (the spinner just turned off too fast
            // for this check to see). What really matters is the final state.
            val finalState = awaitState(viewModel) { it.syncingKeys.isEmpty() }
            assertThat(finalState.syncingKeys).isEmpty()
        }

    @Test
    fun `triggerSync keeps the spinner on when the gate allows`() =
        runTest {
            coEvery { p2pUseCase.connect(any(), any()) } returns true
            val viewModel = createViewModel()

            viewModel.onAction(SyncAction.SyncHistory(peerId = "peer-1"))

            val state = awaitState(viewModel) { "peer-1:history" in it.syncingKeys }
            assertThat(state.syncingKeys).contains("peer-1:history")
        }

    @Test
    fun `confirmConnect resets without an error when the mobile-data gate declines`() =
        runTest {
            coEvery { p2pUseCase.connect(any(), any()) } returns false
            val viewModel = createViewModel()

            viewModel.onAction(SyncAction.ProposeConnect(encodePairingCode("peer-2")))
            assertThat(viewModel.uiState.value.pendingConnect).isNotNull()

            viewModel.onAction(SyncAction.ConfirmConnect)

            val finalState = awaitState(viewModel) { !it.connecting }
            assertThat(finalState.connecting).isFalse()
            // Declining mobile data isn't a connection failure — showing `CONNECTION_FAILED`
            // here would be misleading (the global dialog itself already communicated the choice).
            assertThat(finalState.connectError).isNull()
        }

    private fun encodePairingCode(peerId: String) =
        br.acerola.comic.util.p2p.PairingCode.encode(id = peerId, deviceId = null, addrs = byteArrayOf())
}
