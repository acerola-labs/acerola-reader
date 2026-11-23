package br.acerola.comic.module.main.remotelibrary.state

import br.acerola.comic.error.message.SyncProtocolError
import br.acerola.comic.service.network.ComicSummary

data class RemoteLibraryUiState(
    val peerDisplayName: String = "",
    val comics: List<ComicSummary> = emptyList(),
    /** Distinguishes "still waiting for `browse:library:result`" from "peer really has an empty
     *  library" — [comics] alone can't tell those apart once it's empty. */
    val loaded: Boolean = false,
    /** Raw — only used as a fallback/template argument when [errorType] is `null` (see
     *  `RemoteLibraryScreen`, same pattern as `TransferLogEntry`/`SyncResult`). */
    val errorMessage: String? = null,
    /** Recognized cause (`SyncProtocolError`), when there is one — the UI prefers this over
     *  embedding the raw [errorMessage] in a template. */
    val errorType: SyncProtocolError? = null,
    /** `comicName` -> local path (`file://...`) of the cover already downloaded via
     *  `acerola/browse-cover/1`, ready for Coil to load. */
    val coverPaths: Map<String, String> = emptyMap(),
    /** `comicName` of the in-progress `acerola/sync-comic/1` session, or `null` if none — only one
     *  at a time, same protocol restriction as
     *  [br.acerola.comic.module.comic.ComicViewModel]'s `_syncingPeerId`. */
    val syncingComicName: String? = null,
)
