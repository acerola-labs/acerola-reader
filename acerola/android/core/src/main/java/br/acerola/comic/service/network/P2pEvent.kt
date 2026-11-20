package br.acerola.comic.service.network

import br.acerola.comic.error.message.SyncProtocolError

/**
 * Typed representation of the raw `(event, data)` events emitted by the Rust P2P node.
 * `data` arrives as a string (usually JSON) whose format depends on the event itself.
 */
sealed interface P2pEvent {
    data class PeerTrustedFirstTime(
        val peerId: String,
    ) : P2pEvent

    /**
     * The handshake (`acerola/handshake/1`) finished exchanging identity/DeviceInfo with this
     * peer — a deterministic signal that "pairing worked", emitted by the library itself
     * (`rpc:device_info_received` on the side that connected, `rpc:device_info_exchanged` on
     * the side that received). Far more reliable than polling the peer list with a timeout.
     */
    data class HandshakeCompleted(
        val peerId: String,
    ) : P2pEvent

    data class HistorySyncStarted(
        val peerId: String,
    ) : P2pEvent

    data class HistorySyncComplete(
        val peerId: String,
        val progressApplied: Int,
        val progressSkipped: Int,
        val chaptersReadApplied: Int,
        val chaptersReadSkipped: Int,
    ) : P2pEvent

    data class HistorySyncError(
        val peerId: String,
        val message: String,
        val error: SyncProtocolError? = null,
    ) : P2pEvent

    /** Push of progress + "read" markers of ONE or more selected chapters
     *  (`acerola/sync-history-entry/1`) — lighter than [HistorySyncStarted]/[HistorySyncComplete]
     *  (whole library), triggered by the "send to device" action on the comic screen. */
    data class HistoryEntrySyncStarted(
        val peerId: String,
    ) : P2pEvent

    data class HistoryEntrySyncComplete(
        val peerId: String,
    ) : P2pEvent

    data class HistoryEntrySyncError(
        val peerId: String,
        val message: String,
        val error: SyncProtocolError? = null,
    ) : P2pEvent

    data class FileSyncManifestExchanged(
        val peerId: String,
        val missingCount: Int,
        val offeringCount: Int,
    ) : P2pEvent

    data class FileSyncProgress(
        val peerId: String,
        val comicName: String,
        val chapter: String,
        val bytesTransferred: Long,
        val totalBytes: Long,
    ) : P2pEvent

    data class FileSyncChapterComplete(
        val peerId: String,
        val comicName: String,
        val chapter: String,
    ) : P2pEvent

    data class FileSyncChapterFailed(
        val peerId: String,
        val comicName: String,
        val chapter: String,
        // Always raw/in English — it is the original technical text (sometimes low-level from
        // QUIC/iroh, like "stream reset by peer"), used only for this bus's debug log and as a
        // fallback in `error` when the cause is not recognized. It must never be shown directly
        // in the UI without going through a template (see `SyncScreen::describeEntry`, same pattern
        // already used by `HistorySyncError.message`).
        val reason: String,
        // Recognized cause (`SyncProtocolError.fromCode`, from the `code` Rust attaches —
        // see `classify_sync_error` in `protocol/files/mod.rs`), already as a typed value — not a
        // raw `code: String` for consumers to keep reclassifying with `when`. `null` when the
        // cause is not recognized (uses raw `reason` as fallback). `UserMessage`/`UiText` does not
        // need a `Context` to be built, only at render time (`SyncScreen`).
        val error: SyncProtocolError? = null,
    ) : P2pEvent

    /** Counterpart of [FileSyncChapterComplete] for extra items (cover/banner/`ComicInfo.xml`) —
     *  `kind` is `"cover"`/`"banner"`/`"comic_info"` (see `EXTRA_KIND_*` in
     *  `protocol/files/model.rs`), not a chapter name. */
    data class FileSyncExtraComplete(
        val peerId: String,
        val comicName: String,
        val kind: String,
    ) : P2pEvent

    /** Counterpart of [FileSyncChapterFailed] for extra items — same `reason`/
     *  `error` contract (see docs there: raw/in English only for the log, `error` already classified by
     *  `SyncProtocolError.fromCode`). There is no SESSION failure sentinel here — that only
     *  happens on the chapter channel (`FileSyncChapterFailed` with empty `comicName`/`chapter`). */
    data class FileSyncExtraFailed(
        val peerId: String,
        val comicName: String,
        val kind: String,
        val reason: String,
        val error: SyncProtocolError? = null,
    ) : P2pEvent

    data class FileSyncComplete(
        val peerId: String,
        val receivedCount: Int,
        val sentCount: Int,
        val failedCount: Int,
        /** How many received chapters already existed locally with a checksum different from the
         *  peer's — a real conflict (see `missing_from` in `protocol/files/exchange.rs`).
         *  They are still pulled normally; only reported separately so the final notification
         *  can mention it happened, without one notification per chapter. */
        val conflictsCount: Int = 0,
    ) : P2pEvent

    data class LibraryBrowseResult(
        val peerId: String,
        val comics: List<ComicSummary>,
    ) : P2pEvent

    data class LibraryBrowseError(
        val peerId: String,
        val message: String,
        val error: SyncProtocolError? = null,
    ) : P2pEvent

    /** Result of `acerola/browse-cover/1` — `status` is `"not_modified"`, `"changed"` (with
     *  `path` pointing to the freshly written local cache) or `"unavailable"` (peer does not have that
     *  cover). `coverVersion` comes along to update the local known-version cache even
     *  on `not_modified` (confirms the already cached version is still valid). */
    data class CoverBrowseResult(
        val peerId: String,
        val comicName: String,
        val status: String,
        val coverVersion: Long?,
        val path: String?,
    ) : P2pEvent

    data class CoverBrowseError(
        val peerId: String,
        val comicName: String,
        val message: String,
    ) : P2pEvent

    data class Unknown(
        val event: String,
        val data: String,
    ) : P2pEvent
}

/** A comic from a remote peer's library — summarized to name + chapter count, not
 *  the full manifest (checksum/file name per chapter). Only used to choose which
 *  comic to request via `SyncAction.SyncComic` later. `coverVersion` reuses the peer's
 *  `ComicDirectory.lastModified` — used to decide whether `acerola/browse-cover/1` needs to
 *  fetch a new cover before triggering the fetch. */
data class ComicSummary(
    val comicName: String,
    val chapterCount: Int,
    val coverVersion: Long,
)
