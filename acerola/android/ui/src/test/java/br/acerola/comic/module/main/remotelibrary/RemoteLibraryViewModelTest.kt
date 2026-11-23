package br.acerola.comic.module.main.remotelibrary

import app.cash.turbine.test
import br.acerola.comic.MainDispatcherRule
import br.acerola.comic.error.UserMessage
import br.acerola.comic.module.main.remotelibrary.state.RemoteLibraryUiState
import br.acerola.comic.service.PeerAddress
import br.acerola.comic.service.SyncDirection
import br.acerola.comic.service.network.P2pEventBus
import br.acerola.comic.usecase.network.P2pUseCase
import br.acerola.comic.usecase.network.SyncComicWithPeerUseCase
import br.acerola.comic.usecase.network.SyncWithPeerResult
import com.google.common.truth.Truth.assertThat
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.runTest
import org.junit.Before
import org.junit.Rule
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class RemoteLibraryViewModelTest {
    @get:Rule
    val coroutineRule = MainDispatcherRule()

    private val p2pUseCase = mockk<P2pUseCase>(relaxed = true)
    private val p2pEventBus = P2pEventBus()
    private val syncComicWithPeerUseCase = mockk<SyncComicWithPeerUseCase>(relaxed = true)

    private lateinit var viewModel: RemoteLibraryViewModel

    @Before
    fun setup() {
        val pairedPeer = PeerAddress(id = "peer-1", deviceId = null, addrs = byteArrayOf())
        every { p2pUseCase.getPairedPeers() } returns listOf(pairedPeer)

        viewModel = RemoteLibraryViewModel(p2pUseCase, p2pEventBus, syncComicWithPeerUseCase)
        // `browseLibrary` fired by `init` already resolves the peer (stub above) — no "not
        // paired" error competing with the events the tests below expect.
        viewModel.init(peerId = "peer-1", peerDisplayName = "Peer 1")
    }

    /** A real `Thread.sleep` (not `delay()` — the work being awaited runs on the real
     *  `Dispatchers.IO`, its own thread pool, unaffected by this test's `TestDispatcher`
     *  virtual time). Needed for the `STARTED` path: `syncComic` schedules a
     *  `viewModelScope.launch` (60s timeout) right after setting `syncingComicName` — without
     *  waiting for that `launch` to actually have been fired (not just the mocked call having
     *  been observed by `coVerify`), the test may finish and the `@get:Rule` reset
     *  `Dispatchers.Main` BEFORE that coroutine on `Dispatchers.IO` gets there — which
     *  crashes with "Dispatchers.Main was accessed... test dispatcher was unset" on a thread with
     *  no handler, and that error doesn't land on this test, it lands on the NEXT one running in
     *  the same JVM (seen in CI: it showed up as a failure in `ReaderViewModelTest`, totally unrelated). */
    private fun awaitState(
        viewModel: RemoteLibraryViewModel,
        timeoutMs: Long = 2000,
        predicate: (RemoteLibraryUiState) -> Boolean,
    ): RemoteLibraryUiState {
        val deadline = System.currentTimeMillis() + timeoutMs
        while (System.currentTimeMillis() < deadline) {
            val state = viewModel.uiState.value
            if (predicate(state)) return state
            Thread.sleep(20)
        }
        return viewModel.uiState.value
    }

    @Test
    fun `syncComic marks syncing when the mobile-data gate allows`() =
        runTest {
            coEvery { syncComicWithPeerUseCase(any(), any(), any(), any()) } returns SyncWithPeerResult.STARTED

            viewModel.syncComic("One Piece")

            val state = awaitState(viewModel) { it.syncingComicName != null }
            assertThat(state.syncingComicName).isEqualTo("One Piece")
            coVerify { syncComicWithPeerUseCase("peer-1", "One Piece", SyncDirection.PULL) }
        }

    @Test
    fun `syncComic sends an error and does not mark syncing when the peer is not paired`() =
        runTest {
            coEvery { syncComicWithPeerUseCase(any(), any(), any(), any()) } returns SyncWithPeerResult.NOT_PAIRED

            viewModel.uiEvents.test {
                viewModel.syncComic("One Piece")
                assertThat(awaitItem()).isInstanceOf(UserMessage.Raw::class.java)
            }
            assertThat(viewModel.uiState.value.syncingComicName).isNull()
        }

    @Test
    fun `syncComic does nothing when the mobile-data gate declines`() =
        runTest {
            coEvery { syncComicWithPeerUseCase(any(), any(), any(), any()) } returns SyncWithPeerResult.DECLINED_MOBILE_DATA

            viewModel.syncComic("One Piece")

            // Confirms the coroutine on `Dispatchers.IO` already ran to completion before checking
            // "nothing else happened" — without this the assertions below could pass just because
            // there wasn't enough time yet.
            coVerify(timeout = 2000) { syncComicWithPeerUseCase("peer-1", "One Piece", SyncDirection.PULL) }

            assertThat(viewModel.uiState.value.syncingComicName).isNull()
            viewModel.uiEvents.test {
                expectNoEvents()
            }
        }
}
