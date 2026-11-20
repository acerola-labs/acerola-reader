package br.acerola.comic.service.network

import android.content.Context
import android.net.Uri
import androidx.documentfile.provider.DocumentFile
import br.acerola.comic.adapter.contract.gateway.ComicSingleSyncGateway
import br.acerola.comic.adapter.metadata.comicinfo.ComicInfoEngine
import br.acerola.comic.config.preference.ComicDirectoryPreference
import br.acerola.comic.local.dao.archive.ChapterArchiveDao
import br.acerola.comic.local.dao.archive.ComicDirectoryDao
import br.acerola.comic.local.entity.archive.ComicDirectory
import br.acerola.comic.logging.AcerolaLogger
import br.acerola.comic.logging.LogSource
import br.acerola.comic.pattern.media.MediaFile
import br.acerola.comic.usecase.network.RegisterSyncedChapterUseCase
import br.acerola.comic.util.file.sha256
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import p2p.FfiComicSummaryEntry
import p2p.FfiExtraManifestEntry
import p2p.FfiFileManifestEntry
import p2p.FileSyncProvider
import java.io.IOException
import java.io.InputStream
import java.io.OutputStream
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.atomic.AtomicLong
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Kotlin implementation of [FileSyncProvider] used by the Rust protocol `acerola/sync-files/1`.
 * Reads/writes chapter files (and, since the extension to cover/banner/ComicInfo.xml, "extra"
 * items) via SAF/DocumentFile in chunks — Rust never sees a stream directly, only opaque
 * `Long` handles, so the handle maps here are the "memory" of transfers in flight.
 */
@Singleton
class FileSyncProviderImpl
    @Inject
    constructor(
        @param:ApplicationContext private val context: Context,
        private val comicDirectoryDao: ComicDirectoryDao,
        private val chapterArchiveDao: ChapterArchiveDao,
        private val registerSyncedChapter: RegisterSyncedChapterUseCase,
        @param:ComicInfoEngine private val comicInfoEngine: ComicSingleSyncGateway,
    ) : FileSyncProvider {
        private data class ReadHandle(
            val stream: InputStream,
        )

        /**
         * `target` tells a chapter from an extra item only at FINALIZE time — `writeChapterChunk`
         * (also reused by `beginExtraWrite`, see the trait docs in `callbacks.rs` on the Rust
         * side) writes to the same `outputStream` regardless of what `target` is.
         */
        private sealed class WriteTarget {
            data class Chapter(
                val comicName: String,
                val chapter: String,
            ) : WriteTarget()

            data class Extra(
                val comicName: String,
                val kind: String,
            ) : WriteTarget()
        }

        private data class WriteHandle(
            val target: WriteTarget,
            val fileName: String,
            val expectedChecksum: String,
            val tempFile: DocumentFile,
            val comicFolder: DocumentFile,
            val outputStream: OutputStream,
        )

        private val readHandles = ConcurrentHashMap<Long, ReadHandle>()
        private val writeHandles = ConcurrentHashMap<Long, WriteHandle>()
        private val handleCounter = AtomicLong(1)

        /**
         * Serializes "resolve or create the comic folder" across concurrent calls of
         * `beginChapterWrite`/`beginExtraWrite` — now that `receive_files` (Rust side,
         * `exchange.rs`) fetches several chapters in parallel, two chapters of the SAME new
         * comic could arrive here at the same time, both see `getDirectoryByName == null` (the
         * DB record only happens later, in `finalizeChapterWrite`) and each create its
         * own SAF folder with the same `comicName` — `DocumentsProvider` does not guarantee
         * display name uniqueness, only URI/document ID uniqueness, so this split the chapters
         * of the same comic into two distinct physical folders. Unlike Desktop
         * (`resolve_comic_dir`/`create_synced_comic`), which solves the same race via the
         * database's `UniqueViolation`, here what must be atomic is creating the SAF directory
         * itself — hence the lock instead of relying only on the Room constraint.
         */
        private val comicFolderLock = Mutex()

        @Volatile
        private var cachedLibraryRoot: DocumentFile? = null

        private suspend fun libraryRoot(): DocumentFile? {
            cachedLibraryRoot?.let { return it }
            val uriString = ComicDirectoryPreference.folderUriFlow(context).first() ?: return null
            val root = DocumentFile.fromTreeUri(context, Uri.parse(uriString)) ?: return null
            cachedLibraryRoot = root
            return root
        }

        // The checksum is computed during the library scan (`ChapterIndexer.buildEntity`), not here —
        // reading the whole file to hash it during the P2P handshake is the worst possible moment
        // for it (it was what blew the other side's 30s timeout on a large
        // or freshly scanned library). A chapter without a cached checksum (library scanned before
        // this change, not rescanned yet) is left out of the manifest — better to omit it than
        // block the whole session; the next rescan fills it in.
        override fun getFileManifest(): List<FfiFileManifestEntry> =
            runBlocking {
                comicDirectoryDao.getAllDirectories().first().flatMap { comic ->
                    chapterArchiveDao
                        .getChaptersListByDirectoryId(comic.id)
                        .map { chapter ->
                            async(Dispatchers.IO) {
                                val checksum = chapter.checksum ?: return@async null
                                val file = DocumentFile.fromSingleUri(context, Uri.parse(chapter.path))
                                if (file == null || !file.exists()) return@async null

                                FfiFileManifestEntry(
                                    comicName = comic.name,
                                    chapter = chapter.chapter,
                                    fileName = file.name ?: chapter.chapter,
                                    checksum = checksum,
                                    sizeBytes = file.length().toULong(),
                                )
                            }
                        }.awaitAll()
                        .filterNotNull()
                }
            }

        // Room/SQLite only — no `DocumentFile`/SAF here, on purpose. `getFileManifest()`
        // above pays a `DocumentFile.exists()` per chapter (one binder transaction each) because
        // it must point to a real file for the transfer; `browse-library` only
        // needs name + count, so it uses the aggregate query directly on the DB and never touches
        // SAF — it is the same cost that blew the protocol timeout before this split.
        override fun getLibrarySummary(): List<FfiComicSummaryEntry> =
            runBlocking {
                chapterArchiveDao.getLibrarySummary().map { row ->
                    FfiComicSummaryEntry(
                        comicName = row.comicName,
                        chapterCount = row.chapterCount.toUInt(),
                        coverVersion = row.coverVersion,
                    )
                }
            }

        override fun openChapterForRead(
            comicName: String,
            chapter: String,
        ): Long =
            runBlocking {
                val comic = comicDirectoryDao.getDirectoryByName(comicName) ?: return@runBlocking -1L
                val chapterArchive =
                    chapterArchiveDao.getChapterByDirectoryAndChapter(comic.id, chapter) ?: return@runBlocking -1L
                val stream =
                    context.contentResolver.openInputStream(Uri.parse(chapterArchive.path)) ?: return@runBlocking -1L

                val handle = handleCounter.getAndIncrement()
                readHandles[handle] = ReadHandle(stream)
                handle
            }

        override fun readChapterChunk(
            handle: Long,
            chunkSize: UInt,
        ): ByteArray {
            val readHandle = readHandles[handle] ?: return ByteArray(0)
            val buffer = ByteArray(chunkSize.toInt())

            // `InputStream.read()` only signals end of file with -1 — 0 means "no bytes
            // available right now, but not done" (possible on a cross-process SAF stream).
            // Treating 0 as the end (as before) made the sender stop sending bytes
            // IN THE MIDDLE of the chapter — the receiving side keeps waiting for the rest, and the stream
            // misaligns: the next bytes to arrive are the NEXT chapter's JSON header,
            // mistakenly interpreted as more binary content of the current chapter.
            var read = readHandle.stream.read(buffer)
            while (read == 0) {
                read = readHandle.stream.read(buffer)
            }
            if (read == -1) return ByteArray(0)
            return if (read == buffer.size) buffer else buffer.copyOf(read)
        }

        override fun closeReadHandle(handle: Long) {
            runCatching { readHandles.remove(handle)?.stream?.close() }
        }

        override fun beginChapterWrite(
            comicName: String,
            chapter: String,
            fileName: String,
            expectedChecksum: String,
            sizeBytes: ULong,
        ): Long =
            runBlocking {
                val root = libraryRoot() ?: return@runBlocking -1L

                // If the comic already exists locally, the chapter must land in the SAME folder
                // as the existing chapters (`comic.path`). A new comic goes straight into
                // `<root>/<comicName>/` — the same place DirectoryScanner would create if the
                // user had added it manually. No intermediate `synced/` folder:
                // it had no exclusion in DirectoryScanner (which recurses into
                // any folder with no manga directly in it), so a full rescan could
                // rediscover `synced/<comicName>/` as a second ghost ComicDirectory.
                val comicFolder =
                    comicFolderLock.withLock {
                        val existingComic = comicDirectoryDao.getDirectoryByName(comicName)
                        if (existingComic != null) {
                            DocumentFile.fromTreeUri(context, Uri.parse(existingComic.path))
                        } else {
                            root.findFile(comicName) ?: root.createDirectory(comicName)
                        }
                    }
                if (comicFolder == null) return@runBlocking -1L

                // Final name comes from the peer (preserves the original .cbz/.cbr extension);
                // if it comes empty (the "chapter unavailable" placeholder), fall back to the
                // chapter label.
                val finalName = fileName.ifBlank { chapter }
                val tempName = "$finalName.part"
                comicFolder.findFile(tempName)?.delete()
                val tempFile =
                    comicFolder.createFile("application/octet-stream", tempName) ?: return@runBlocking -1L
                val outputStream =
                    context.contentResolver.openOutputStream(tempFile.uri, "wt") ?: run {
                        tempFile.delete()
                        return@runBlocking -1L
                    }

                val handle = handleCounter.getAndIncrement()
                writeHandles[handle] =
                    WriteHandle(
                        WriteTarget.Chapter(comicName, chapter),
                        finalName,
                        expectedChecksum,
                        tempFile,
                        comicFolder,
                        outputStream,
                    )
                handle
            }

        override fun writeChapterChunk(
            handle: Long,
            bytes: ByteArray,
        ): Boolean {
            val writeHandle = writeHandles[handle] ?: return false
            return try {
                writeHandle.outputStream.write(bytes)
                true
            } catch (error: IOException) {
                AcerolaLogger.e("FileSyncProvider", "Failed to write chunk for handle $handle", LogSource.NETWORK, error)
                false
            }
        }

        /**
         * Closes the stream, verifies the temp file's checksum and removes the handle from the map —
         * shared between `finalizeChapterWrite`/`finalizeExtraWrite`, the only difference
         * between them is what happens AFTER the checksum matches (see each one's `target`).
         * `null` covers both "unknown handle" and "checksum mismatch" (temp file
         * already deleted in that second case).
         */
        private fun finalizeAndVerifyChecksum(handle: Long): WriteHandle? {
            val writeHandle = writeHandles.remove(handle) ?: return null
            runCatching { writeHandle.outputStream.close() }

            val actualChecksum = writeHandle.tempFile.sha256(context)
            if (actualChecksum != writeHandle.expectedChecksum) {
                writeHandle.tempFile.delete()
                return null
            }

            return writeHandle
        }

        override fun finalizeChapterWrite(handle: Long): Boolean =
            runBlocking {
                val writeHandle = finalizeAndVerifyChecksum(handle) ?: return@runBlocking false
                val target = writeHandle.target as? WriteTarget.Chapter ?: return@runBlocking false

                writeHandle.comicFolder.findFile(writeHandle.fileName)?.delete()
                if (!writeHandle.tempFile.renameTo(writeHandle.fileName)) {
                    writeHandle.tempFile.delete()
                    return@runBlocking false
                }

                val finalFile = writeHandle.comicFolder.findFile(writeHandle.fileName) ?: writeHandle.tempFile
                registerSyncedChapter(
                    comicName = target.comicName,
                    chapter = target.chapter,
                    checksum = writeHandle.expectedChecksum,
                    fileUri = finalFile.uri.toString(),
                    comicFolderUri = writeHandle.comicFolder.uri.toString(),
                )
                true
            }

        override fun abortChapterWrite(handle: Long) {
            val writeHandle = writeHandles.remove(handle) ?: return
            runCatching { writeHandle.outputStream.close() }
            writeHandle.tempFile.delete()
        }

        // --- Extra items (cover/banner/ComicInfo.xml) ---
        //
        // `openExtraForRead`/`beginExtraWrite` insert into the SAME maps (`readHandles`/
        // `writeHandles`) used for chapters — `readChapterChunk`/`writeChapterChunk`/
        // `closeReadHandle` are reused unchanged (see the trait docs in `callbacks.rs`
        // on the Rust side), so extra handles must live in the same space for those three
        // functions to find them.

        override fun getExtrasManifest(): List<FfiExtraManifestEntry> =
            runBlocking {
                comicDirectoryDao
                    .getAllDirectories()
                    .first()
                    .map { comic ->
                        async(Dispatchers.IO) { buildExtraEntries(comic) }
                    }.awaitAll()
                    .flatten()
            }

        private fun buildExtraEntries(comic: ComicDirectory): List<FfiExtraManifestEntry> {
            val entries = mutableListOf<FfiExtraManifestEntry>()

            comic.cover?.let { path -> artworkExtraEntry(comic.name, EXTRA_KIND_COVER, path)?.let(entries::add) }
            comic.banner?.let { path -> artworkExtraEntry(comic.name, EXTRA_KIND_BANNER, path)?.let(entries::add) }
            findComicInfoFile(comic)?.let { file ->
                runCatching {
                    entries.add(
                        FfiExtraManifestEntry(
                            comicName = comic.name,
                            kind = EXTRA_KIND_COMIC_INFO,
                            fileName = file.name ?: COMIC_INFO_FILE_NAME,
                            checksum = file.sha256(context),
                            sizeBytes = file.length().toULong(),
                        ),
                    )
                }
            }

            return entries
        }

        private fun artworkExtraEntry(
            comicName: String,
            kind: String,
            path: String,
        ): FfiExtraManifestEntry? {
            val file = DocumentFile.fromSingleUri(context, Uri.parse(path))
            if (file == null || !file.exists()) return null

            return runCatching {
                FfiExtraManifestEntry(
                    comicName = comicName,
                    kind = kind,
                    fileName = file.name ?: MediaFile.COVER.defaultFileName,
                    checksum = file.sha256(context),
                    sizeBytes = file.length().toULong(),
                )
            }.getOrNull()
        }

        /**
         * Finds `ComicInfo.xml` (case-insensitive) directly at the root of the comic folder — same
         * convention as Desktop (`core/services/metadata/mod.rs::find_comic_info_path`).
         */
        private fun findComicInfoFile(comic: ComicDirectory): DocumentFile? {
            val comicFolder = DocumentFile.fromTreeUri(context, Uri.parse(comic.path)) ?: return null
            return comicFolder.listFiles().firstOrNull {
                it.isFile && it.name?.equals(COMIC_INFO_FILE_NAME, ignoreCase = true) == true
            }
        }

        override fun openExtraForRead(
            comicName: String,
            kind: String,
        ): Long =
            runBlocking {
                val comic = comicDirectoryDao.getDirectoryByName(comicName) ?: return@runBlocking -1L

                val sourceFile =
                    when (kind) {
                        EXTRA_KIND_COVER -> comic.cover?.let { DocumentFile.fromSingleUri(context, Uri.parse(it)) }
                        EXTRA_KIND_BANNER -> comic.banner?.let { DocumentFile.fromSingleUri(context, Uri.parse(it)) }
                        EXTRA_KIND_COMIC_INFO -> findComicInfoFile(comic)
                        else -> null
                    } ?: return@runBlocking -1L

                val stream = context.contentResolver.openInputStream(sourceFile.uri) ?: return@runBlocking -1L

                val handle = handleCounter.getAndIncrement()
                readHandles[handle] = ReadHandle(stream)
                handle
            }

        override fun beginExtraWrite(
            comicName: String,
            kind: String,
            fileName: String,
            expectedChecksum: String,
            sizeBytes: ULong,
        ): Long =
            runBlocking {
                val root = libraryRoot() ?: return@runBlocking -1L

                val comicFolder =
                    comicFolderLock.withLock {
                        val existingComic = comicDirectoryDao.getDirectoryByName(comicName)
                        if (existingComic != null) {
                            DocumentFile.fromTreeUri(context, Uri.parse(existingComic.path))
                        } else {
                            root.findFile(comicName) ?: root.createDirectory(comicName)
                        }
                    }
                if (comicFolder == null) return@runBlocking -1L

                // Canonical final name per `kind` — cover/banner always `.jpg` (same convention already
                // used by `CoverSaver`/`BannerSaver` in the manual metadata flow, regardless
                // of the extension the peer announced), `ComicInfo.xml` always that exact name.
                val finalName =
                    when (kind) {
                        EXTRA_KIND_COVER -> MediaFile.COVER.defaultFileName
                        EXTRA_KIND_BANNER -> MediaFile.BANNER.defaultFileName
                        EXTRA_KIND_COMIC_INFO -> COMIC_INFO_FILE_NAME
                        else -> return@runBlocking -1L
                    }

                val tempName = "$finalName.part"
                comicFolder.findFile(tempName)?.delete()
                val tempFile =
                    comicFolder.createFile("application/octet-stream", tempName) ?: return@runBlocking -1L
                val outputStream =
                    context.contentResolver.openOutputStream(tempFile.uri, "wt") ?: run {
                        tempFile.delete()
                        return@runBlocking -1L
                    }

                val handle = handleCounter.getAndIncrement()
                writeHandles[handle] =
                    WriteHandle(WriteTarget.Extra(comicName, kind), finalName, expectedChecksum, tempFile, comicFolder, outputStream)
                handle
            }

        override fun finalizeExtraWrite(handle: Long): Boolean =
            runBlocking {
                val writeHandle = finalizeAndVerifyChecksum(handle) ?: return@runBlocking false
                val target = writeHandle.target as? WriteTarget.Extra ?: return@runBlocking false

                when (target.kind) {
                    EXTRA_KIND_COVER -> finalizeArtworkExtra(writeHandle, target, MediaFile.COVER)
                    EXTRA_KIND_BANNER -> finalizeArtworkExtra(writeHandle, target, MediaFile.BANNER)
                    EXTRA_KIND_COMIC_INFO -> finalizeComicInfoExtra(writeHandle, target)
                    else -> {
                        writeHandle.tempFile.delete()
                        false
                    }
                }
            }

        private suspend fun finalizeArtworkExtra(
            writeHandle: WriteHandle,
            target: WriteTarget.Extra,
            media: MediaFile,
        ): Boolean {
            val directory =
                comicDirectoryDao.getDirectoryByName(target.comicName) ?: run {
                    writeHandle.tempFile.delete()
                    return false
                }

            // Cleans up any pre-existing cover/banner before writing the new one — same orphan
            // cleanup `CoverSaver`/`BannerSaver` already do for the manual metadata flow.
            writeHandle.comicFolder.listFiles().forEach { file ->
                val name = file.name ?: return@forEach
                val matches = if (media == MediaFile.COVER) MediaFile.isCover(name) else MediaFile.isBanner(name)
                if (matches) file.delete()
            }

            if (!writeHandle.tempFile.renameTo(writeHandle.fileName)) {
                writeHandle.tempFile.delete()
                return false
            }

            val finalFile = writeHandle.comicFolder.findFile(writeHandle.fileName) ?: writeHandle.tempFile
            val savedUri = finalFile.uri.toString()
            val updated =
                if (media == MediaFile.COVER) {
                    directory.copy(cover = savedUri, lastModified = System.currentTimeMillis())
                } else {
                    directory.copy(banner = savedUri, lastModified = System.currentTimeMillis())
                }
            comicDirectoryDao.update(updated)
            return true
        }

        private suspend fun finalizeComicInfoExtra(
            writeHandle: WriteHandle,
            target: WriteTarget.Extra,
        ): Boolean {
            val directory =
                comicDirectoryDao.getDirectoryByName(target.comicName) ?: run {
                    writeHandle.tempFile.delete()
                    return false
                }

            writeHandle.comicFolder
                .listFiles()
                .filter { it.isFile && it.name?.equals(writeHandle.fileName, ignoreCase = true) == true }
                .forEach { it.delete() }

            if (!writeHandle.tempFile.renameTo(writeHandle.fileName)) {
                writeHandle.tempFile.delete()
                return false
            }

            // Best effort: the file was already persisted successfully even if the
            // metadata reprocessing fails — same stance as Desktop
            // (`infra/sync/protocol/transfer.rs::receive_extras`).
            runCatching { comicInfoEngine.refreshManga(comicId = directory.id, baseUri = null) }
                .onFailure { error ->
                    AcerolaLogger.e(
                        "FileSyncProvider",
                        "ComicInfo.xml received, but metadata reprocessing failed",
                        LogSource.NETWORK,
                        error,
                    )
                }

            return true
        }

        override fun abortExtraWrite(handle: Long) = abortChapterWrite(handle)

        companion object {
            private const val EXTRA_KIND_COVER = "cover"
            private const val EXTRA_KIND_BANNER = "banner"
            private const val EXTRA_KIND_COMIC_INFO = "comic_info"
            private const val COMIC_INFO_FILE_NAME = "ComicInfo.xml"
        }
    }
