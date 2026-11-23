package br.acerola.comic.service.archive

import android.content.Context
import android.net.Uri
import androidx.documentfile.provider.DocumentFile
import br.acerola.comic.local.entity.archive.ChapterArchive
import br.acerola.comic.local.translator.persistence.toChapterArchiveEntity
import br.acerola.comic.logging.AcerolaLogger
import br.acerola.comic.logging.LogSource
import br.acerola.comic.util.file.FastFileMetadata
import br.acerola.comic.util.file.sha256
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class ChapterIndexer
    @Inject
    constructor(
        @param:ApplicationContext private val context: Context,
    ) {
        /**
         * Computes the checksum here, during the scan — in the background, with no network timeout on top —
         * instead of leaving it for P2P sync time (`FileSyncProvider.getFileManifest`), which was
         * the worst possible moment to read the whole file. `ChapterSyncService.sync()` only
         * calls this for new chapters or ones with a changed `lastModified`; an already indexed,
         * unchanged chapter never goes through here again (see the `return@forEachIndexed`
         * there), so the cost is incremental per scan, not the whole library every time.
         */
        suspend fun buildEntity(
            file: FastFileMetadata,
            comicId: Long,
            fileUri: String,
            chapterSort: String,
            volumeFk: Long?,
            isSpecial: Boolean,
        ): ChapterArchive =
            file
                .toChapterArchiveEntity(
                    comicId = comicId,
                    fileUri = fileUri,
                    chapterSort = chapterSort,
                    volumeFk = volumeFk,
                    isSpecial = isSpecial,
                ).copy(checksum = computeChecksum(fileUri))

        /**
         * Also used by [ChapterSyncService] to backfill the checksum of an
         * already indexed chapter left with `checksum = null` — without treating the
         * chapter as new/changed (no history remapping, no duplicated row).
         */
        suspend fun computeChecksum(fileUri: String): String? =
            runCatching {
                DocumentFile.fromSingleUri(context, Uri.parse(fileUri))?.sha256(context)
            }.getOrElse { error ->
                AcerolaLogger.e(TAG, "Failed to hash chapter file: $fileUri", LogSource.REPOSITORY, error)
                null
            }

        companion object {
            private const val TAG = "ChapterIndexer"
        }
    }
