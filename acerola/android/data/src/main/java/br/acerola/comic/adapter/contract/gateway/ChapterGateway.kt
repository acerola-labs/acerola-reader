package br.acerola.comic.adapter.contract.gateway

import android.net.Uri
import arrow.core.Either
import br.acerola.comic.error.message.LibrarySyncError
import kotlinx.coroutines.flow.StateFlow

/**
 * Interface for observing the chapter sync state.
 */
interface ChapterSyncStatusGateway {
    val progress: StateFlow<Int>
    val isIndexing: StateFlow<Boolean>
}

/**
 * Interface for syncing chapters.
 * Every chapter Engine implements this.
 */
interface ChapterSyncGateway : ChapterSyncStatusGateway {
    suspend fun refreshComicChapters(
        comicId: Long,
        baseUri: Uri? = null,
    ): Either<LibrarySyncError, Unit>
}

/**
 * Interface only for reading chapters.
 * Implemented only by Engines that have persistence or access to local/remote data (Archive, Mangadex).
 */
interface ChapterReadGateway<T> {
    fun observeChapters(
        comicId: Long,
        sortType: String = "NUMBER",
        isAscending: Boolean = true,
    ): StateFlow<T>

    suspend fun getChapterPage(
        comicId: Long,
        total: Int,
        page: Int,
        pageSize: Int = 20,
        sortType: String = "NUMBER",
        isAscending: Boolean = true,
    ): T
}
