use acerola_p2p::api::error::P2pError;

/// Stable identifier of a sync error cause, serialized in `snake_case` (`"busy"`,
/// `"timeout"`, `"connection_lost"`) — the Kotlin side (`SyncProtocolError.fromCode`, `:infra`)
/// maps it to a `UiText.StringResource`, since Android has no language switch (only
/// pt-BR). Shared by all P2P protocols (files/comic, history, cover-browse,
/// library-browse) — same `code` in all of them, same pattern as desktop (`SyncErrorCode` in
/// `transfer.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SyncErrorCode {
    Busy,
    Timeout,
    ConnectionLost,
    /// Unlike the other three (transport/protocol causes), this one does not come from
    /// `classify_sync_error` — `receive_one_chapter`/`receive_one_extra` (`protocol/files/
    /// exchange.rs`) attach this code directly to the `chapter_failed`/
    /// `extra_failed` payload when they know, right then, that the failure was specifically a
    /// checksum mismatch (not an I/O error): `write_chapter_chunk` did not fail on any chunk, but
    /// `finalize_chapter_write`/`finalize_extra_write` (which only does the checksum comparison
    /// at that point, on the Kotlin side) still returned `false`.
    ChecksumMismatch,
    /// The peer replied that it does not have the referenced comic (`HistoryEntryAck.comic_known ==
    /// false`, `acerola/sync-history-entry/1`) — history never creates a new comic at the
    /// destination, so this is expected when the other side has not yet synced that
    /// comic's files. Same reasoning as `Busy`: it does not come from `classify_sync_error`, it is
    /// checked by exact equality against a string this module controls.
    ComicNotFound,
}

/// Reason used when the outbound of `acerola/sync-history-entry/1` receives `comic_known:
/// false` in the ack — our own string recognized by exact equality in `classify_sync_error`, not a
/// heuristic on third-party text. Mirrored on Desktop
/// (`infra/sync/protocol/transfer.rs::PEER_COMIC_NOT_FOUND_REASON`).
pub(crate) const PEER_COMIC_NOT_FOUND_REASON: &str = "peer does not have this comic";

/// Classifies a `P2pError` (not its text) into a `SyncErrorCode`. Only reliable because each
/// protocol's `ChapterTransfer`/blob-fetch stopped flattening `ConnectionError` into
/// `StreamFailed(err.to_string())` — the error that reaches here is already the real variant that
/// `lib/p2p` classified at the source (`classify_connection_error`/`classify_get_error`, crate
/// `acerola-p2p`), not text to guess via substring matching. `Busy` is the exception: it is not
/// a `ConnectionError` variant (it is an app protocol concept, not a transport one) —
/// each `run_and_report*`/guard builds that text inline with the peer id interpolated, so the
/// match here is by substring on a phrase THIS VERY MODULE controls, not a heuristic
/// on third-party text.
pub(crate) fn classify_sync_error(error: &P2pError) -> Option<SyncErrorCode> {
    match error {
        P2pError::Timeout => Some(SyncErrorCode::Timeout),
        P2pError::PeerDisconnected(_) => Some(SyncErrorCode::ConnectionLost),
        P2pError::StreamFailed(msg) if msg.contains("already in progress") => {
            Some(SyncErrorCode::Busy)
        }
        P2pError::StreamFailed(msg) if msg == PEER_COMIC_NOT_FOUND_REASON => {
            Some(SyncErrorCode::ComicNotFound)
        }
        _ => None,
    }
}
