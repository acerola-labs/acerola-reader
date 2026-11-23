use serde::{Deserialize, Serialize};

use crate::callbacks::{FfiChapterReadEntry, FfiReadingProgressEntry};

/// Manifest exchanged only once per `acerola/sync-history/1` session —
/// contains the whole local state of reading progress and read chapters, since both
/// are small (no binary payload), letting each side apply its own diff/conflict logic
/// locally, without a second network round.
///
/// Wire schema shared with Desktop (`acerola-desktop/.../infra/sync/messages.rs`,
/// via `#[serde(rename)]` on the `entries`/`read_markers`/`chapter` fields). The two sides do not
/// share code, so the field names here (`reading_progress`, `chapters_read`,
/// `chapter_sort`) are the contract — do not rename without updating the other side too.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct HistoryManifest {
    pub reading_progress: Vec<FfiReadingProgressEntry>,
    pub chapters_read: Vec<FfiChapterReadEntry>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct HistorySyncStats {
    pub progress_applied: u32,
    pub progress_skipped: u32,
    pub chapters_read_applied: u32,
    pub chapters_read_skipped: u32,
}

/// First message of the `acerola/sync-history-entry/1` protocol, written by the outbound side —
/// explicitly declares which comic and which chapters (`chapter_sort`) the following manifest
/// is scoped to, instead of the inbound only finding out by reading the manifest content
/// (which may be empty if no selected chapter has progress/"read" marker — in that
/// case the inbound would not even know which comic to validate). Schema mirrored on Desktop
/// (`infra/sync/messages.rs::HistoryEntryRequest`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoryEntryRequest {
    pub comic_name: String,
    pub chapter_sorts: Vec<String>,
}

/// Reply from the inbound side of `acerola/sync-history-entry/1`, instead of an empty ack —
/// carries the real apply result. `comic_known` is what lets the outbound tell
/// "I sent it and the peer applied it" from "I sent it, but the peer did not even have that comic"
/// (previously a silent false positive). Schema mirrored on Desktop
/// (`infra/sync/messages.rs::HistoryEntryAck`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct HistoryEntryAck {
    pub comic_known: bool,
    pub entries_applied: u32,
    pub markers_applied: u32,
}
