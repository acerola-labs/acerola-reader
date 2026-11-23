//! Foreign traits (implemented in Kotlin) used by the sync protocols to read/write
//! data in the app's Room/SAF, keeping the manifest/diff/apply logic entirely in Rust.

use serde::{Deserialize, Serialize};

/// Source/destination of the reading history data (`reading_history` + `chapter_read`) used
/// by the `acerola/sync-history/1` protocol. Implemented in Kotlin against Room.
#[uniffi::export(with_foreign)]
pub trait HistorySyncProvider: Send + Sync {
    /// All local reading progress rows, with natural key (comic name).
    fn get_reading_progress(&self) -> Vec<FfiReadingProgressEntry>;

    /// All local rows of chapters marked as read, with natural key.
    fn get_chapters_read(&self) -> Vec<FfiChapterReadEntry>;

    /// Reading progress of ONE single comic (`comic_name`) — used by the individual push
    /// (`acerola/sync-history-entry/1`), which exists to avoid needing `get_reading_progress()`
    /// (whole library) just to carry the progress of a comic that just changed. `None`
    /// when the comic does not exist locally or never had saved progress.
    fn get_reading_progress_for_comic(&self, comic_name: String)
        -> Option<FfiReadingProgressEntry>;

    /// Reading progress of ONE comic, only returned if the current chapter (the one in
    /// `reading_history`) is among the selected `chapter_sorts` — used together with
    /// `get_chapters_read_for_chapters` by the explicit push of the selected chapter(s)
    /// (`acerola/sync-history-entry/1`), which must not leak the rest of the library. Empty (not
    /// `None`) to keep the same return shape as `get_reading_progress`.
    fn get_reading_progress_for_chapters(
        &self,
        comic_name: String,
        chapter_sorts: Vec<String>,
    ) -> Vec<FfiReadingProgressEntry>;

    /// "Read" markers restricted to the selected `chapter_sorts` of ONE comic —
    /// same reasoning as `get_reading_progress_for_chapters`, never leaks the rest of the
    /// library.
    fn get_chapters_read_for_chapters(
        &self,
        comic_name: String,
        chapter_sorts: Vec<String>,
    ) -> Vec<FfiChapterReadEntry>;

    /// Only checks whether the comic exists (without touching progress/read) — used by the
    /// `acerola/sync-history-entry/1` contract to validate BEFORE replying "applied
    /// successfully": without this, an empty manifest (no selected chapter had progress/
    /// marker) gave no error signal even when the peer never had that comic.
    fn comic_exists(&self, comic_name: String) -> bool;

    /// Applies a progress row received from the peer (Rust already decided it wins via LWW).
    /// Returns `false` if the referenced comic does not exist locally (entry ignored).
    fn apply_reading_progress(&self, entry: FfiReadingProgressEntry) -> bool;

    /// Applies a "chapter read" row received from the peer (idempotent union).
    /// Returns `false` if the referenced comic does not exist locally (entry ignored).
    fn apply_chapter_read(&self, entry: FfiChapterReadEntry) -> bool;
}

#[derive(uniffi::Record, Debug, Clone, Serialize, Deserialize)]
pub struct FfiReadingProgressEntry {
    pub comic_name: String,
    pub chapter_sort: String,
    pub last_page: i32,
    pub is_completed: bool,
    pub updated_at: i64,
}

#[derive(uniffi::Record, Debug, Clone, Serialize, Deserialize)]
pub struct FfiChapterReadEntry {
    pub comic_name: String,
    pub chapter_sort: String,
    pub created_at: i64,
}

/// Source/destination of the chapter files used by the `acerola/sync-files/1` protocol.
/// Implemented in Kotlin against SAF/DocumentFile. Based on opaque handles (`i64`) because
/// streams do not cross the FFI boundary directly — Kotlin keeps a map of open handles.
#[uniffi::export(with_foreign)]
pub trait FileSyncProvider: Send + Sync {
    /// All local chapters with checksum/size, with natural key `(comic_name, chapter)`.
    fn get_file_manifest(&self) -> Vec<FfiFileManifestEntry>;

    /// Per-comic summary (name + chapter count), straight from Room — used only by
    /// `acerola/browse-library/1`. Kept separate from `get_file_manifest()` on purpose: that
    /// method does a `DocumentFile.exists()` via SAF per chapter (one binder transaction each),
    /// which only pays off when the result really needs to point to a file (an actual
    /// transfer). Just to list titles that is pure cost, and it already blew the
    /// protocol timeout on a large/freshly scanned library — see the comment in
    /// `FileSyncProviderImpl.getFileManifest()`.
    fn get_library_summary(&self) -> Vec<FfiComicSummaryEntry>;

    /// Opens a local chapter for chunked reading (sending side). `-1` if not found.
    fn open_chapter_for_read(&self, comic_name: String, chapter: String) -> i64;

    /// Reads up to `chunk_size` bytes from the open handle; an empty `Vec` signals end of file.
    fn read_chapter_chunk(&self, handle: i64, chunk_size: u32) -> Vec<u8>;

    /// Closes a read handle opened by `open_chapter_for_read`.
    fn close_read_handle(&self, handle: i64);

    /// Starts receiving a chapter (receiving side): creates a temp file in `synced/`.
    /// `file_name` is the real file name announced by the peer in the header (schema shared
    /// with Desktop) — used as the final name instead of the chapter label, to keep the
    /// original extension (.cbz/.cbr) of the sending side.
    fn begin_chapter_write(
        &self,
        comic_name: String,
        chapter: String,
        file_name: String,
        expected_checksum: String,
        size_bytes: u64,
    ) -> i64;

    /// Writes a received chunk to the write handle opened by `begin_chapter_write`.
    fn write_chapter_chunk(&self, handle: i64, bytes: Vec<u8>) -> bool;

    /// Verifies the temp file's checksum and moves it to the final destination in `synced/`.
    fn finalize_chapter_write(&self, handle: i64) -> bool;

    /// Aborts an in-progress write (peer disconnected midway, checksum mismatch, etc).
    fn abort_chapter_write(&self, handle: i64);

    /// Local extra items (cover/banner/`ComicInfo.xml`), one `FfiExtraManifestEntry` per present
    /// item — comics with none of the three simply do not appear here. Mirrors Desktop's
    /// `FileSyncService::build_manifest` extras
    /// (`infra/sync/messages.rs::FileExtraInfo`).
    fn get_extras_manifest(&self) -> Vec<FfiExtraManifestEntry>;

    /// Opens a local extra item for chunked reading (sending side). `-1` if not found.
    /// `kind` is one of `EXTRA_KIND_*` (`protocol::files::model`). Reuses the existing
    /// `read_chapter_chunk`/`close_read_handle` to read/close — the handle is
    /// opaque, agnostic to chapter vs. extra.
    fn open_extra_for_read(&self, comic_name: String, kind: String) -> i64;

    /// Starts receiving an extra item (receiving side). Reuses the existing `write_chapter_chunk`
    /// to write the received chunks.
    fn begin_extra_write(
        &self,
        comic_name: String,
        kind: String,
        file_name: String,
        expected_checksum: String,
        size_bytes: u64,
    ) -> i64;

    /// Verifies the temp file's checksum and persists it to the final destination: the
    /// `cover`/`banner` column of `comic_directory` for those two `kind`s, or plain `ComicInfo.xml`
    /// (plus metadata reprocessing) for the third.
    fn finalize_extra_write(&self, handle: i64) -> bool;

    /// Aborts an in-progress extra write.
    fn abort_extra_write(&self, handle: i64);
}

#[derive(uniffi::Record, Debug, Clone, Serialize, Deserialize)]
pub struct FfiExtraManifestEntry {
    pub comic_name: String,
    /// One of `EXTRA_KIND_*` (`protocol::files::model`).
    pub kind: String,
    pub file_name: String,
    pub checksum: String,
    pub size_bytes: u64,
}

/// Encrypted storage (Android Keystore, via `EncryptedSharedPreferences`) used by
/// `SecureP2pStorage`/`SecureTrustedStore` (see `storage.rs`/`trust_store.rs`) to persist
/// the node identity, peer cache and TOFU trust list without touching plain text on disk.
/// Plain key/value — the whole layout (which keys exist, JSON format of each) is
/// decided on the Rust side, Kotlin only does opaque storage.
#[uniffi::export(with_foreign)]
pub trait SecureBlobStore: Send + Sync {
    /// `Err` only on a real backend failure (e.g. `EncryptedSharedPreferences`/Keystore
    /// unavailable or corrupted) — never used for "key does not exist yet", which is not
    /// an error. Telling them apart matters: if `load_identity` treated a real failure as
    /// "identity never existed", the node would silently mint a new identity and
    /// break all existing pairing without telling anyone.
    fn save_blob(&self, key: String, value: Vec<u8>) -> Result<(), SecureBlobStoreError>;

    /// `Ok(None)` if the key was never saved (normal state). `Err` only on a real
    /// backend failure — see `save_blob`.
    fn load_blob(&self, key: String) -> Result<Option<Vec<u8>>, SecureBlobStoreError>;

    /// Removes a key — idempotent (a missing key is not an error). Used today only by the
    /// Iroh Services ticket (see `storage.rs::clear_iroh_services_ticket`), when the
    /// user turns the source off or replaces it with a new one.
    fn clear_blob(&self, key: String) -> Result<(), SecureBlobStoreError>;
}

#[derive(uniffi::Error, thiserror::Error, Debug)]
pub enum SecureBlobStoreError {
    #[error("secure blob store access failed: {reason}")]
    AccessFailed { reason: String },
}

#[derive(uniffi::Record, Debug, Clone, Serialize, Deserialize)]
pub struct FfiFileManifestEntry {
    pub comic_name: String,
    pub chapter: String,
    /// Real file name (with extension), read from the `DocumentFile` in Kotlin — needed
    /// to speak the same wire schema Desktop uses in `FileHeader`/`FileChapterInfo`.
    pub file_name: String,
    pub checksum: String,
    pub size_bytes: u64,
}

#[derive(uniffi::Record, Debug, Clone, Serialize, Deserialize)]
pub struct FfiComicSummaryEntry {
    pub comic_name: String,
    pub chapter_count: u32,
    /// Reuses `ComicDirectory.lastModified`/`comic_directory.last_modified` — no new
    /// hash. The peer compares it against the version already cached locally to decide whether it
    /// needs to fetch a new cover via `acerola/browse-cover/1`.
    pub cover_version: i64,
}

/// Source/destination of a comic's cover (`cover.jpg`), used by the
/// `acerola/browse-cover/1` protocol. Separate from `FileSyncProvider` on purpose: covers are small
/// enough (thumbnail) to travel as a whole `Vec<u8>` in a single FFI call, without needing the
/// opaque handle machinery that chapters (potentially hundreds of MB) require.
#[uniffi::export(with_foreign)]
pub trait CoverBrowseProvider: Send + Sync {
    /// Local cover of `comic_name`, if any. `bytes: None` covers both "comic does not exist"
    /// and "exists but has no saved cover" — both cases result in the same
    /// `not_modified`/absent reply to the peer, no need to tell them apart on the wire.
    fn get_local_cover(&self, comic_name: String) -> FfiCoverEntry;

    /// Writes the cover received from `peer_id` for `comic_name` to a local cache (never in the
    /// user's tree) and returns the resulting path/URI, for the UI to load via Coil. Cache key
    /// `(peer_id, comic_name, cover_version)` — the caller decides whether it already has that version
    /// cached before triggering the fetch, this only persists what was already downloaded.
    fn save_remote_cover(
        &self,
        peer_id: String,
        comic_name: String,
        cover_version: i64,
        bytes: Vec<u8>,
    ) -> String;
}

#[derive(uniffi::Record, Debug, Clone, Serialize, Deserialize)]
pub struct FfiCoverEntry {
    pub cover_version: i64,
    pub bytes: Option<Vec<u8>>,
}
