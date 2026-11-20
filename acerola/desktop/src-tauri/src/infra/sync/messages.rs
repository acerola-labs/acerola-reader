use serde::{Deserialize, Serialize};

/// A reading progress entry, identified by the natural key
/// (comic name + chapter label) — the only form comparable between
/// two independent SQLite databases, since autoincrement IDs are not portable.
///
/// Field names on the wire (`chapter_sort` instead of `chapter`) follow the schema already
/// used by Android in `protocol/history/model.rs` — the shared schema of the
/// `acerola/sync-history/1` ALPN, the internal Rust names do not change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub comic_name: String,
    #[serde(rename = "chapter_sort")]
    pub chapter: String,
    pub last_page: i64,
    pub is_completed: bool,
    pub updated_at: i64,
}

/// A "chapter read" marker. Has no `updated_at` — it is a binary fact that either
/// exists or not, so the merge between devices is a union (insert_or_ignore), not
/// last-write-wins.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadMarker {
    pub comic_name: String,
    #[serde(rename = "chapter_sort")]
    pub chapter: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HistoryManifest {
    #[serde(rename = "reading_progress")]
    pub entries: Vec<HistoryEntry>,
    #[serde(rename = "chapters_read")]
    pub read_markers: Vec<ReadMarker>,
}

/// First message of the `acerola/sync-history-entry/1` protocol, written by the outbound side —
/// explicitly declares which comic and which chapters (`chapter_sort`) the following manifest
/// is scoped to, instead of the inbound only finding that out by reading the manifest content
/// (which may come empty if no selected chapter has progress/"read" marker — in that
/// case the inbound would not even know which comic to validate). Same pattern as
/// `ComicSyncRequest`, schema mirrored on Android (`protocol/history/model.rs`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntryRequest {
    pub comic_name: String,
    pub chapter_sorts: Vec<String>,
}

/// Answer of the inbound side of `acerola/sync-history-entry/1`, replacing the old empty ack
/// (`{}`) — carries the real result of the application, not just "the session did not drop". `comic_known`
/// is what lets the outbound tell "I sent and the peer applied" from "I sent, but the peer did not
/// even have that comic" (previously a silent false positive). Schema mirrored on Android.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntryAck {
    pub comic_known: bool,
    pub entries_applied: u32,
    pub markers_applied: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChapterInfo {
    pub chapter: String,
    pub file_name: String,
    pub checksum: Option<String>,
    pub size: u64,
}

/// Metadata of a per-comic "extra" item (cover, banner or `ComicInfo.xml`) — same shape
/// for all three, told apart by the field carrying them in `FileComicInfo` (`cover`/`banner`/
/// `comic_info`) and, at transfer time, by the `kind` field of `FileExtraHeader`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileExtraInfo {
    pub file_name: String,
    /// Hex SHA-256 of the file bytes — recomputed on every `build_manifest()` (small
    /// files, unlike chapters, so they do not need a checksum cached in the
    /// database like `ChapterArchive.checksum`).
    pub checksum: String,
    pub size: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileComicInfo {
    pub comic_name: String,
    pub chapters: Vec<FileChapterInfo>,
    /// `None` covers both "comic without a saved cover" and "cover file disappeared from disk" —
    /// the three fields below (`cover`/`banner`/`comic_info`) follow this same convention.
    /// `skip_serializing_if` omits the key instead of writing `null` when missing — a
    /// comic with none of the three produces exactly the same JSON as before this change.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover: Option<FileExtraInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banner: Option<FileExtraInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comic_info: Option<FileExtraInfo>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileManifest {
    pub comics: Vec<FileComicInfo>,
}

pub const EXTRA_KIND_COVER: &str = "cover";
pub const EXTRA_KIND_BANNER: &str = "banner";
pub const EXTRA_KIND_COMIC_INFO: &str = "comic_info";

/// What this side wants *from the other side* — computed locally after comparing
/// both manifests, and sent back for the peer to decide what to transmit.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileWantList {
    pub wanted: Vec<(String, String)>,
    /// `(comic_name, kind)` pairs, `kind` being one of the `EXTRA_KIND_*` — same idea as
    /// `wanted`, but for cover/banner/ComicInfo.xml instead of chapters. `#[serde(default)]`
    /// so a peer without this protocol version still deserializes it as an empty list instead of
    /// failing; `skip_serializing_if` keeps the wire identical to before this change when there
    /// is no extra item to request.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub wanted_extras: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHeader {
    pub comic_name: String,
    pub chapter: String,
    pub file_name: String,
    pub size: u64,
    /// Manifest SHA-256 (`FileChapterInfo.checksum`) — passed along as is for
    /// local write verification, unrelated to blobs.
    pub checksum: Option<String>,
    /// Blob hash (BLAKE3, hex) returned by `P2pBlobStore::put` on the sending side —
    /// it is what the receiving side uses in `P2pBlobStore::fetch` to pull the actual bytes.
    /// `None` only in the empty "I no longer have this chapter" header (`size: 0`).
    pub blob_hash: Option<String>,
}

/// Transfer header of an extra item (cover/banner/`ComicInfo.xml`) — parallel to
/// `FileHeader`, but with `kind` (`EXTRA_KIND_*`) instead of `chapter`: they are different
/// concepts (an extra item does not belong to any chapter), a dedicated header avoids
/// overloading the `chapter` field with a value that is not really a chapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileExtraHeader {
    pub comic_name: String,
    pub kind: String,
    pub file_name: String,
    pub size: u64,
    /// `None` only in the empty "I no longer have this item" header (`size: 0`) — same convention
    /// as `FileHeader`.
    pub checksum: Option<String>,
    pub blob_hash: Option<String>,
}

/// Explicit direction chosen by whoever starts an `acerola/sync-comic/1` — replaces the implicit
/// direction that used to emerge only from the diff of both manifests. Always from the outbound
/// side's point of view: `Push` = outbound sends the comic to the peer; `Pull` = outbound pulls the
/// comic from the peer. Schema mirrored on Android (`protocol/files/model.rs::SyncDirection`) — both
/// sides do not share code, the names/values (`"push"`/`"pull"`) must match exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncDirection {
    Push,
    Pull,
}

/// First message of the `acerola/sync-comic/1` protocol, written by the starting side — tells
/// the peer which comic to scope before the manifest exchange (see
/// `FileSyncService::build_manifest_for_comic`). Without it the answering side would have no way
/// to know which comic to scope when the initiator does not have any chapter of it
/// locally yet ("pull" case: the initiator's manifest would come empty, with no name inside).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComicSyncRequest {
    pub comic_name: String,
    pub direction: SyncDirection,
    /// Chapter labels (the same key used in `FileChapterInfo.chapter`) to scope the
    /// session to — empty means "whole comic", the behavior before this change
    /// (kept for the "Sync with device" button). Both sides filter their own
    /// local manifest by this list before writing it (see `ComicSyncOutbound`/
    /// `ComicSyncInbound::run`), so the diff never sees chapters outside the requested scope.
    /// `#[serde(default)]` so a peer without this protocol version still deserializes it as
    /// an empty list instead of failing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chapters: Vec<String>,
}

/// Minimal marker written by the outbound side of `acerola/browse-library/1` before reading the
/// answer — it carries no real data (the request itself is "list your library"),
/// but it *must* exist because of a rule of `quinn` itself (the QUIC library underneath
/// `iroh`), documented in `Connection::open_bi()`: "the Connection that calls open_bi()
/// must write to its SendStream before the other Connection is able to accept_bi() (...)
/// waiting on the RecvStream without writing anything to SendStream will never succeed".
///
/// That was exactly the bug reopened on 2026-08-22 (30s timeout, `FRAME_TIMEOUT`, on both
/// sides): `LibraryBrowseOutbound` opened the stream and only read, it never wrote anything — the
/// inbound side's `accept_bi()` never fired, so the inbound handler was never even
/// invoked, and the outbound kept waiting for an answer that would never be written. The other
/// protocols (`sync-history`, `sync-files`, `sync-comic`) never had this problem because their
/// outbound side always writes something first (`FileManifest`/`ComicSyncRequest`).
///
/// The inbound side only needs to drain this message, not look at its content — that is why it
/// carries no field.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LibraryBrowseRequest {}

/// Summary of a remote library comic, only used to list/search (without transferring
/// anything yet) — see the `acerola/browse-library/1` protocol. Names mirrored on Android
/// (`protocol/library_browse/model.rs::ComicSummaryEntry`/`LibrarySummary`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComicSummaryEntry {
    pub comic_name: String,
    pub chapter_count: u32,
    /// Reuses `ComicDirectory.last_modified`/`comic_directory.last_modified` — no new
    /// hash. The client compares it against the locally cached version to decide whether it needs
    /// to fetch a new cover via `acerola/browse-cover/1`.
    pub cover_version: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LibrarySummary {
    pub comics: Vec<ComicSummaryEntry>,
}

/// Single frame written by the outbound side of `acerola/browse-cover/1` — the connection itself is
/// not the request (unlike `browse-library`), because here the outbound must say WHICH
/// comic and which version it already has cached, so the inbound can decide between `not_modified` and
/// publishing a new blob.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverRequest {
    pub comic_name: String,
    /// `None` = "I never fetched this cover before" — any version the inbound has counts
    /// as "changed".
    pub known_version: Option<i64>,
}

pub const COVER_STATUS_NOT_MODIFIED: &str = "not_modified";
pub const COVER_STATUS_CHANGED: &str = "changed";
pub const COVER_STATUS_UNAVAILABLE: &str = "unavailable";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverResponse {
    pub status: String,
    pub cover_version: Option<i64>,
    /// Only present when `status == "changed"` — blob hash (BLAKE3, hex) the outbound uses
    /// in `ChapterTransfer::fetch_reader` to pull the actual bytes.
    pub cover_hash: Option<String>,
}

/// Message sent instead of the manifest when `FileSyncSessionGuard` already rejected the session
/// locally (the peer already has an active session) — schema mirrored on Android
/// (`protocol/files/model.rs::SessionBusy`), the only way for both sides to say the same thing
/// without sharing code. The `error` field is the discriminator: no other message
/// of the file protocol uses this key (`FileManifest` uses `comics`, `FileWantList` uses
/// `wanted`), so the reader can detect it before trying to deserialize it as
/// `FileManifest`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionBusy {
    pub error: String,
    pub reason: String,
}

#[cfg(test)]
mod wire_contract_tests {
    use super::*;

    /// Locks the `HistoryManifest` wire schema against the format Android
    /// (`protocol/history/model.rs` + `callbacks.rs`) produces and expects — both sides
    /// do not share code, so this test is what guarantees they do not silently diverge
    /// again.
    #[test]
    fn history_manifest_matches_android_wire_shape() {
        let manifest = HistoryManifest {
            entries: vec![HistoryEntry {
                comic_name: "Berserk".into(),
                chapter: "12".into(),
                last_page: 5,
                is_completed: false,
                updated_at: 1000,
            }],
            read_markers: vec![ReadMarker {
                comic_name: "Berserk".into(),
                chapter: "11".into(),
                created_at: 900,
            }],
        };

        let value = serde_json::to_value(&manifest).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "reading_progress": [{
                    "comic_name": "Berserk",
                    "chapter_sort": "12",
                    "last_page": 5,
                    "is_completed": false,
                    "updated_at": 1000
                }],
                "chapters_read": [{
                    "comic_name": "Berserk",
                    "chapter_sort": "11",
                    "created_at": 900
                }]
            })
        );

        // Message exactly in the format Android sends (keys reading_progress/
        // chapters_read, field chapter_sort) must deserialize without error.
        let android_wire = serde_json::json!({
            "reading_progress": [{
                "comic_name": "Berserk",
                "chapter_sort": "12",
                "last_page": 5,
                "is_completed": false,
                "updated_at": 1000
            }],
            "chapters_read": []
        });
        let decoded: HistoryManifest = serde_json::from_value(android_wire).unwrap();
        assert_eq!(decoded.entries[0].chapter, "12");
    }

    /// Locks the wire schema of `HistoryEntryRequest`/`HistoryEntryAck` (new messages of the
    /// standardized `acerola/sync-history-entry/1` contract) against the format mirrored on
    /// Android (`protocol/history/model.rs::HistoryEntryRequest`/`HistoryEntryAck`) — identical field
    /// names on both sides, no `rename` (new messages, no legacy to preserve).
    #[test]
    fn history_entry_request_and_ack_match_android_wire_shape() {
        let request =
            HistoryEntryRequest { comic_name: "Berserk".into(), chapter_sorts: vec!["12".into()] };
        let value = serde_json::to_value(&request).unwrap();
        assert_eq!(value, serde_json::json!({ "comic_name": "Berserk", "chapter_sorts": ["12"] }));

        let android_ack_wire =
            serde_json::json!({ "comic_known": false, "entries_applied": 0, "markers_applied": 0 });
        let decoded: HistoryEntryAck = serde_json::from_value(android_ack_wire).unwrap();
        assert!(!decoded.comic_known);
        assert_eq!(decoded.entries_applied, 0);
        assert_eq!(decoded.markers_applied, 0);
    }

    /// `LibraryBrowseRequest` only needs to exist on the wire (see the type docs) — locks that the
    /// format is an empty object, so an old peer ("the connection is the request" design, without
    /// any read before answering) receives a harmless, ignorable JSON, not something
    /// that could confuse a stricter parser on the other side.
    #[test]
    fn library_browse_request_serializes_as_empty_object() {
        let value = serde_json::to_value(LibraryBrowseRequest::default()).unwrap();
        assert_eq!(value, serde_json::json!({}));
    }

    /// Locks the `FileComicInfo` wire schema with the new `cover`/`banner`/`comic_info` fields
    /// against the format produced/expected by Android
    /// (`protocol/files/model.rs::FileComicInfo`) — same convention as the other tests of this
    /// suite: no enum tag, `#[serde(default)]` covering the "missing item" case.
    #[test]
    fn file_comic_info_matches_android_wire_shape_with_extras() {
        let comic = FileComicInfo {
            comic_name: "Berserk".into(),
            chapters: vec![FileChapterInfo {
                chapter: "Cap 1".into(),
                file_name: "Cap 1.cbz".into(),
                checksum: Some("abc123".into()),
                size: 42,
            }],
            cover: Some(FileExtraInfo {
                file_name: "cover.jpg".into(),
                checksum: "cover-hash".into(),
                size: 1024,
            }),
            banner: None,
            comic_info: Some(FileExtraInfo {
                file_name: "ComicInfo.xml".into(),
                checksum: "info-hash".into(),
                size: 256,
            }),
        };

        let value = serde_json::to_value(&comic).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "comic_name": "Berserk",
                "chapters": [{
                    "chapter": "Cap 1",
                    "file_name": "Cap 1.cbz",
                    "checksum": "abc123",
                    "size": 42
                }],
                "cover": { "file_name": "cover.jpg", "checksum": "cover-hash", "size": 1024 },
                "comic_info": { "file_name": "ComicInfo.xml", "checksum": "info-hash", "size": 256 }
            })
        );

        // Payload from an old peer, without the new fields — must deserialize as None,
        // not fail.
        let old_peer_wire = serde_json::json!({
            "comic_name": "Berserk",
            "chapters": []
        });
        let decoded: FileComicInfo = serde_json::from_value(old_peer_wire).unwrap();
        assert!(decoded.cover.is_none());
        assert!(decoded.banner.is_none());
        assert!(decoded.comic_info.is_none());
    }

    /// Locks the `FileWantList.wanted_extras` wire schema — same string pair pattern
    /// used by `wanted`, but `(comic_name, kind)` instead of `(comic_name,
    /// chapter)`.
    #[test]
    fn file_want_list_serializes_wanted_extras_as_tuple_array() {
        let wanted = FileWantList {
            wanted: vec![("Berserk".into(), "Cap 1".into())],
            wanted_extras: vec![("Berserk".into(), EXTRA_KIND_COVER.into())],
        };

        let value = serde_json::to_value(&wanted).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "wanted": [["Berserk", "Cap 1"]],
                "wanted_extras": [["Berserk", "cover"]]
            })
        );

        // Old peer without `wanted_extras` in the payload — deserializes as an empty list.
        let old_peer_wire = serde_json::json!({ "wanted": [] });
        let decoded: FileWantList = serde_json::from_value(old_peer_wire).unwrap();
        assert!(decoded.wanted_extras.is_empty());
    }

    /// Locks the `ComicSyncRequest`/`SyncDirection` wire schema against the format mirrored
    /// on Android (`protocol/files/model.rs::ComicSyncScope`/`SyncDirection`) — values
    /// `"push"`/`"pull"` in `snake_case`, no extra enum tag. An empty `chapters` disappears from the JSON
    /// (`skip_serializing_if`), keeping the wire identical to before this change when the session
    /// is not scoped to specific chapters.
    #[test]
    fn comic_sync_request_serializes_direction_as_snake_case_string() {
        let request = ComicSyncRequest {
            comic_name: "Berserk".into(),
            direction: SyncDirection::Push,
            chapters: vec![],
        };

        let value = serde_json::to_value(&request).unwrap();
        assert_eq!(value, serde_json::json!({ "comic_name": "Berserk", "direction": "push" }));

        let android_wire = serde_json::json!({ "comic_name": "Berserk", "direction": "pull" });
        let decoded: ComicSyncRequest = serde_json::from_value(android_wire).unwrap();
        assert_eq!(decoded.direction, SyncDirection::Pull);
        assert!(decoded.chapters.is_empty());
    }

    /// Locks that a session scoped to specific chapters serializes/deserializes the list of
    /// labels correctly, in the format mirrored on Android
    /// (`protocol/files/model.rs::ComicSyncScope`).
    #[test]
    fn comic_sync_request_serializes_chapters_when_scoped() {
        let request = ComicSyncRequest {
            comic_name: "Berserk".into(),
            direction: SyncDirection::Push,
            chapters: vec!["Cap 1".into(), "Cap 2".into()],
        };

        let value = serde_json::to_value(&request).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "comic_name": "Berserk",
                "direction": "push",
                "chapters": ["Cap 1", "Cap 2"]
            })
        );

        let android_wire = serde_json::json!({
            "comic_name": "Berserk",
            "direction": "pull",
            "chapters": ["Cap 1"]
        });
        let decoded: ComicSyncRequest = serde_json::from_value(android_wire).unwrap();
        assert_eq!(decoded.chapters, vec!["Cap 1".to_string()]);
    }

    /// Locks the `FileExtraHeader` wire schema against the format mirrored on Android
    /// (`protocol/files/model.rs::FileExtraHeader`).
    #[test]
    fn file_extra_header_matches_android_wire_shape() {
        let header = FileExtraHeader {
            comic_name: "Berserk".into(),
            kind: EXTRA_KIND_BANNER.into(),
            file_name: "banner.png".into(),
            size: 2048,
            checksum: Some("banner-hash".into()),
            blob_hash: Some("blake3-hex".into()),
        };

        let value = serde_json::to_value(&header).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "comic_name": "Berserk",
                "kind": "banner",
                "file_name": "banner.png",
                "size": 2048,
                "checksum": "banner-hash",
                "blob_hash": "blake3-hex"
            })
        );
    }
}
