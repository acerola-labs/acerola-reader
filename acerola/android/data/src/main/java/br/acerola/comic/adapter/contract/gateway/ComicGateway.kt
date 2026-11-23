package br.acerola.comic.adapter.contract.gateway

import android.net.Uri
import arrow.core.Either
import br.acerola.comic.error.message.LibrarySyncError
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.StateFlow

/**
 * Interface for observing the sync state.
 */
interface ComicSyncStatusGateway {
    val progress: StateFlow<Int>
    val isIndexing: StateFlow<Boolean>
}

/**
 * Interface for syncing a single item.
 * Every Engine implements this - refreshManga is universally supported.
 */
interface ComicSingleSyncGateway : ComicSyncStatusGateway {
    suspend fun refreshManga(
        comicId: Long,
        baseUri: Uri? = null,
    ): Either<LibrarySyncError, Unit>
}

/**
 * Interface for library scan operations (Incremental and Full).
 * Implemented by Directory, MangaDex and AniList.
 */
interface ComicLibraryScanGateway : ComicSyncStatusGateway {
    suspend fun incrementalScan(baseUri: Uri?): Either<LibrarySyncError, Unit>

    suspend fun refreshLibrary(baseUri: Uri?): Either<LibrarySyncError, Unit>
}

/**
 * Interface for a deep library rebuild.
 * Implemented only by ComicDirectoryEngine.
 */
interface ComicRebuildGateway : ComicSyncStatusGateway {
    suspend fun rebuildLibrary(baseUri: Uri?): Either<LibrarySyncError, Unit>
}

/**
 * Interface only for observing data (Read).
 */
interface ComicReadOnlyGateway<T> {
    fun observeLibrary(): Flow<List<T>> = kotlinx.coroutines.flow.flowOf(emptyList())
}

/**
 * Aggregate interface for full sync.
 * Engines that support every operation implement this interface.
 */
interface ComicSyncGateway :
    ComicSingleSyncGateway,
    ComicLibraryScanGateway,
    ComicRebuildGateway

/**
 * Full interface for Engines that read and write (Directories, MangaDex, AniList).
 */
interface ComicGateway<T> :
    ComicReadOnlyGateway<T>,
    ComicSyncGateway

/**
 * Interface for write operations on the local library (hide/delete).
 * Separate from MangaSyncGateway since these are not sync operations.
 */
interface ComicLibraryWriteGateway {
    suspend fun setMangaHidden(
        comicId: Long,
        hidden: Boolean,
    ): Either<LibrarySyncError, Unit>

    suspend fun hideManga(comicId: Long): Either<LibrarySyncError, Unit>

    suspend fun deleteManga(comicId: Long): Either<LibrarySyncError, Unit>

    suspend fun updateMangaSettings(
        comicId: Long,
        externalSyncEnabled: Boolean,
    ): Either<LibrarySyncError, Unit>
}
