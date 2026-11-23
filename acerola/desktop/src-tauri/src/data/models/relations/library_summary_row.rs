/// Result of `ChapterRepository::get_library_summary` — SQL only, no disk access (no
/// `tokio::fs::metadata` per chapter), used by the `acerola/browse-library/1` protocol
/// (`LibraryBrowseInbound`) so large libraries don't blow the protocol timeout.
/// `cover_version` reuses `comic_directory.last_modified` — no new hash.
#[derive(Debug, sqlx::FromRow, Clone)]
pub struct LibrarySummaryRow {
    pub comic_name: String,
    pub chapter_count: i64,
    pub cover_version: i64,
}
