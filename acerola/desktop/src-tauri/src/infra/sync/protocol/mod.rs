pub mod comic_handler;
pub mod comic_sync_registry;
pub mod cover_browse_handler;
pub mod cover_request_registry;
pub mod file_handler;
pub mod file_session_guard;
pub mod history_entry_handler;
pub mod history_entry_registry;
pub mod history_handler;
pub mod library_browse_handler;
pub mod transfer;

pub const HISTORY_SYNC_ALPN: &[u8] = b"acerola/sync-history/1";
/// Individual push of ONE history entry (progress of ONE comic), without exchanging the
/// whole library manifest — see `history_entry_handler.rs`. Complements
/// `HISTORY_SYNC_ALPN`, does not replace it: that one is still the full sync (the "Sync
/// history" button), this one is the one-off "send just this chapter".
pub const HISTORY_ENTRY_SYNC_ALPN: &[u8] = b"acerola/sync-history-entry/1";
pub const FILE_SYNC_ALPN: &[u8] = b"acerola/sync-files/1";
/// Individual sync of ONE comic (push or pull, see `comic_handler.rs`) — reuses the
/// same `FileSyncSessionGuard` as `FILE_SYNC_ALPN`.
pub const COMIC_SYNC_ALPN: &[u8] = b"acerola/sync-comic/1";
/// Lists/searches the remote library before deciding an individual sync — single round-trip,
/// no guard (see `library_browse_handler.rs`). String and format identical to Android
/// (`protocol/library_browse/mod.rs::LIBRARY_BROWSE_ALPN`) — both sides do not share
/// code, so it must match exactly or the transport (which routes by exact ALPN) never
/// finds a handler on the other side.
pub const LIBRARY_BROWSE_ALPN: &[u8] = b"acerola/browse-library/1";
/// Fetches the cover (thumbnail) of ONE remote comic at a time, complementing `browse-library` —
/// see `cover_browse_handler.rs`. No session/guard: short round-trip, it does not run the same
/// risk of duplicated heavy I/O that `sync-files`/`sync-comic` do. String identical to Android
/// (`protocol/cover_browse/mod.rs::COVER_BROWSE_ALPN`).
pub const COVER_BROWSE_ALPN: &[u8] = b"acerola/browse-cover/1";
