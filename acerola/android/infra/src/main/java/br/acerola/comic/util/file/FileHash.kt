package br.acerola.comic.util.file

import android.content.Context
import androidx.documentfile.provider.DocumentFile
import java.security.MessageDigest

fun DocumentFile.sha256(context: Context): String {
    val digest = MessageDigest.getInstance("SHA-256")

    context.contentResolver.openInputStream(uri)?.use { input ->
        val buffer = ByteArray(size = 8_192)
        var read = input.read(buffer)

        // `InputStream.read()` only signals end of file with -1 — 0 is a legitimate
        // "no bytes available right now, but not done" read, possible on a
        // stream that crosses another process (SAF/ContentResolver). Stopping at `read <= 0`
        // (as before) treated that 0 as EOF and cut the hash of a freshly written
        // file before the end, even with the file on disk correct and complete.
        while (read != -1) {
            if (read > 0) digest.update(buffer, 0, read)
            read = input.read(buffer)
        }
    } ?: error("The file could not be opened.")

    return digest.digest().joinToString(separator = "") { "%02x".format(it) }
}
