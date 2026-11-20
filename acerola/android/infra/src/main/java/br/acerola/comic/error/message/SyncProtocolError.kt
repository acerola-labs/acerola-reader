package br.acerola.comic.error.message

import br.acerola.comic.error.UserMessage
import br.acerola.comic.infra.R
import br.acerola.comic.type.UiText

/**
 * P2P sync protocol errors (`acerola/sync-files/1`, `acerola/sync-comic/1`) that have a
 * recognized cause — classified at the source, from the stable `code` Rust attaches to the
 * event (`classify_sync_error`, `protocol/files/mod.rs`), never by `when`/substring matching
 * on the technical text (that stays raw, in English, only for log/fallback — see
 * `P2pEvent.FileSyncChapterFailed.reason`).
 */
sealed interface SyncProtocolError : UserMessage {
    data object SessionBusy : SyncProtocolError {
        override val uiMessage = UiText.StringResource(resId = R.string.error_sync_session_busy)
    }

    data object Timeout : SyncProtocolError {
        override val uiMessage = UiText.StringResource(resId = R.string.error_sync_timeout)
    }

    data object ConnectionLost : SyncProtocolError {
        override val uiMessage = UiText.StringResource(resId = R.string.error_sync_connection_lost)
    }

    /** Unlike the other three (transport/protocol causes, from `classify_sync_error`),
     *  this `code` comes from `chapter_failed_payload`/`extra_failed_payload`
     *  (`protocol/files/exchange.rs`) — attached when `write_chapter_chunk`/blob fetch did NOT
     *  fail, but Kotlin (`finalizeChapterWrite`/`finalizeExtraWrite`) still rejected
     *  the file because of its checksum. */
    data object ChecksumMismatch : SyncProtocolError {
        override val uiMessage = UiText.StringResource(resId = R.string.error_sync_checksum_mismatch)
    }

    /** `code` comes from `HistoryEntryAck.comic_known == false` (`acerola/sync-history-entry/1`) —
     *  the peer replied that it does not have the referenced comic, so nothing was applied on the
     *  other side. History never creates a new comic at the destination, so this is expected when the
     *  other side has not yet synced that comic's files. */
    data object ComicNotFound : SyncProtocolError {
        override val uiMessage = UiText.StringResource(resId = R.string.error_sync_comic_not_found)
    }

    companion object {
        /** The only place that interprets the raw wire `code` (`"busy"`/`"timeout"`/
         *  `"connection_lost"`/`"checksum_mismatch"`/`"comic_not_found"`) — `null` (unrecognized
         *  cause) signals the caller to use the raw `reason` as fallback, instead of a
         *  `SyncProtocolError`. */
        fun fromCode(code: String?): SyncProtocolError? =
            when (code) {
                "busy" -> SessionBusy
                "timeout" -> Timeout
                "connection_lost" -> ConnectionLost
                "checksum_mismatch" -> ChecksumMismatch
                "comic_not_found" -> ComicNotFound
                else -> null
            }
    }
}
