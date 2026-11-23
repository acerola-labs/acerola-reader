package br.acerola.comic.service.network

import android.content.Context
import androidx.core.net.toUri
import br.acerola.comic.local.dao.archive.ComicDirectoryDao
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.runBlocking
import p2p.CoverBrowseProvider
import p2p.FfiCoverEntry
import java.io.File
import javax.inject.Inject
import javax.inject.Singleton

private const val REMOTE_COVERS_CACHE_DIR = "remote_covers"

/**
 * Kotlin implementation of [CoverBrowseProvider] used by the Rust protocol
 * `acerola/browse-cover/1`. Covers are small enough to cross the FFI boundary as a whole
 * `ByteArray`, unlike chapters — no handle-based chunking needed here.
 */
@Singleton
class CoverBrowseProviderImpl
    @Inject
    constructor(
        @param:ApplicationContext private val context: Context,
        private val comicDirectoryDao: ComicDirectoryDao,
    ) : CoverBrowseProvider {
        override fun getLocalCover(comicName: String): FfiCoverEntry =
            runBlocking {
                val directory = comicDirectoryDao.getDirectoryByName(comicName)
                val bytes =
                    directory?.cover?.let { coverUri ->
                        runCatching {
                            context.contentResolver.openInputStream(coverUri.toUri())?.use { it.readBytes() }
                        }.getOrNull()
                    }
                FfiCoverEntry(coverVersion = directory?.lastModified ?: 0L, bytes = bytes)
            }

        // Dedicated cache (never the user's tree) — `(peerId, comicName, coverVersion)` key
        // embedded in the file name, so the same version is never downloaded twice (the caller
        // in Rust already decides that BEFORE triggering the fetch, this only persists).
        override fun saveRemoteCover(
            peerId: String,
            comicName: String,
            coverVersion: Long,
            bytes: ByteArray,
        ): String {
            val cacheDir = File(context.cacheDir, REMOTE_COVERS_CACHE_DIR)
            if (!cacheDir.exists()) cacheDir.mkdirs()

            val sanitize = { value: String -> value.replace(Regex("[^A-Za-z0-9._-]"), "_") }
            val file = File(cacheDir, "${sanitize(peerId)}_${sanitize(comicName)}_$coverVersion.jpg")
            file.writeBytes(bytes)

            return file.toUri().toString()
        }
    }
