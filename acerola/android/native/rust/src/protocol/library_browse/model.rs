use serde::{Deserialize, Serialize};

/// A comic on the replying side, summarized to name + chapter count — not the
/// full manifest (`FileManifest`/`FileChapterInfo`), which would needlessly carry checksum and
/// file name per chapter: the sole purpose here is for the user to choose WHICH comic
/// to request via `acerola/sync-comic/1` later, not to decide what is already in sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ComicSummaryEntry {
    pub comic_name: String,
    pub chapter_count: u32,
    /// Reuses `ComicDirectory.lastModified` — no new hash. The peer compares it against the
    /// version already cached locally to decide whether it needs to fetch a new cover via
    /// `acerola/browse-cover/1`.
    pub cover_version: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct LibrarySummary {
    pub comics: Vec<ComicSummaryEntry>,
}
