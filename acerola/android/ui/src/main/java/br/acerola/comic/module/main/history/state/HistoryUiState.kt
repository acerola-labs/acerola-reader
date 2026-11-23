package br.acerola.comic.module.main.history.state
import br.acerola.comic.dto.ComicDto
import br.acerola.comic.dto.history.ReadingHistoryWithChapterDto
import br.acerola.comic.module.main.sync.state.PairedPeer

data class HistoryItemState(
    val comic: ComicDto,
    val history: ReadingHistoryWithChapterDto,
    val chapterCount: Int = 0,
)

data class HistoryUiState(
    val items: List<HistoryItemState> = emptyList(),
    /** Paired peers for the history sync `PeerPickerSheet` — same component reused
     *  by individual comic sync (Comic Detail / Home). */
    val pairedPeers: List<PairedPeer> = emptyList(),
)
