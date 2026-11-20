package br.acerola.comic.module.main.sync

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import br.acerola.comic.module.main.sync.state.TransferLogEntry
import br.acerola.comic.ui.R
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

/** Turns a raw [TransferLogEntry] into display text — the only place doing that resolution,
 *  so [SyncViewModel] never needs an Android [android.content.Context]-flavored dependency
 *  just to pre-render a string. Shared with
 *  [br.acerola.comic.module.main.transferlog.TransferLogViewModel] (different module, but
 *  same text — the full history is the same data, just filtered/paginated differently). */
@Composable
internal fun describeEntry(entry: TransferLogEntry): String =
    when ("${entry.kind}:${entry.status}") {
        "history:started" -> stringResource(id = R.string.log_sync_history_started)
        "history:complete" -> stringResource(id = R.string.log_sync_history_complete)
        "history:error" -> stringResource(id = R.string.log_sync_history_error, entry.message.orEmpty())
        "files:started" -> stringResource(id = R.string.log_sync_files_started)
        "files:progress" ->
            stringResource(id = R.string.log_sync_files_progress, entry.comicName.orEmpty(), entry.chapter.orEmpty())
        "files:chapterFailed" ->
            stringResource(
                id = R.string.log_sync_files_chapter_failed,
                entry.comicName.orEmpty(),
                entry.chapter.orEmpty(),
            )
        "files:error" -> stringResource(id = R.string.log_sync_files_error, entry.message.orEmpty())
        "files:complete" -> stringResource(id = R.string.log_sync_files_complete)
        else -> entry.message ?: entry.status
    }

/** Same formatting used both in "last synced" per peer (here) and in the full-screen history
 *  ([br.acerola.comic.module.main.transferlog]). */
internal fun formatLogTimestamp(timestampMillis: Long): String = SimpleDateFormat("dd/MM HH:mm", Locale.getDefault()).format(Date(timestampMillis))
