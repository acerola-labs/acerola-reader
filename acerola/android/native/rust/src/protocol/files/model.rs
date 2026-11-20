use serde::{Deserialize, Serialize};

use crate::callbacks::{FfiExtraManifestEntry, FfiFileManifestEntry};

/// All message types exchanged throughout a session of the
/// `acerola/sync-files/1` protocol. Schema shared with Desktop
/// (`acerola-desktop/.../infra/sync/messages.rs`) — the names and formats here must
/// match those exactly, since the two sides do not share code.
///
/// Each message is a standalone JSON frame (no enum tag) via length-delimited codec.
/// A chapter's bytes do not travel inside a JSON message — they are raw binary frames
/// sent right after the matching `FileHeader`, until they add up to `size` bytes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FileChapterInfo {
    pub chapter: String,
    pub file_name: String,
    pub checksum: Option<String>,
    pub size: u64,
}

/// Metadata of a per-comic "extra" item (cover, banner or `ComicInfo.xml`) — schema
/// mirrored on Desktop (`infra/sync/messages.rs::FileExtraInfo`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FileExtraInfo {
    pub file_name: String,
    pub checksum: String,
    pub size: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct FileComicInfo {
    pub comic_name: String,
    pub chapters: Vec<FileChapterInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover: Option<FileExtraInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub banner: Option<FileExtraInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comic_info: Option<FileExtraInfo>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct FileManifest {
    pub comics: Vec<FileComicInfo>,
}

pub(crate) const EXTRA_KIND_COVER: &str = "cover";
pub(crate) const EXTRA_KIND_BANNER: &str = "banner";
pub(crate) const EXTRA_KIND_COMIC_INFO: &str = "comic_info";

/// What this side wants *from the other side* — `(comic_name, chapter)` pairs, plus
/// `wanted_extras`: `(comic_name, kind)` pairs for cover/banner/ComicInfo.xml.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct FileWantList {
    pub wanted: Vec<(String, String)>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub wanted_extras: Vec<(String, String)>,
}

/// Explicit direction chosen by whoever starts an `acerola/sync-comic/1` — replaces the implicit
/// direction that used to emerge only from diffing the two manifests. Always from the outbound
/// side's point of view: `Push` = outbound sends the comic to the peer; `Pull` = outbound pulls the
/// comic from the peer. Schema mirrored on Desktop (`infra/sync/messages.rs::SyncDirection`) — the
/// two sides do not share code, the names/values (`"push"`/`"pull"`) must match exactly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SyncDirection {
    Push,
    Pull,
}

/// Phase 0, exclusive to the `acerola/sync-comic/1` protocol (sync of a single
/// comic) — exchanged before anything of the regular `sync-files` protocol. Only the outbound
/// side knows which comic the user chose (it came from a Kotlin FFI call ->
/// `P2PNode::sync_comic`, not from the wire); the inbound learns it here and filters its own
/// manifest to the same `comic_name` before moving on to phases 1-3 shared with
/// `run_exchange`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ComicSyncScope {
    pub comic_name: String,
    pub direction: SyncDirection,
    /// Chapter labels (same key used in `FileChapterInfo.chapter`) to scope the
    /// session to — empty means "whole comic", the behavior before this change.
    /// Mirrors Desktop (`infra/sync/messages.rs::ComicSyncRequest.chapters`): both sides
    /// filter their own local manifest by this list before writing it (see
    /// `exchange::run_exchange_scoped`). `#[serde(default)]` so a peer without this protocol
    /// version still deserializes it as an empty list instead of failing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chapters: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FileHeader {
    pub comic_name: String,
    pub chapter: String,
    pub file_name: String,
    pub size: u64,
    /// Manifest SHA-256 (`FileChapterInfo.checksum`) — passed as-is to
    /// `FileSyncProvider::begin_chapter_write`/Kotlin verification, unrelated to blobs.
    pub checksum: Option<String>,
    /// Blob hash (BLAKE3, hex) returned by `P2pBlobStore::put` on the sending side —
    /// it is what the receiving side uses in `P2pBlobStore::fetch` to pull the actual bytes.
    /// `None` only in the empty "I no longer have this chapter" header (`size: 0`).
    pub blob_hash: Option<String>,
}

/// Transfer header of an extra item (cover/banner/`ComicInfo.xml`) — parallel to
/// `FileHeader`, schema mirrored on Desktop (`infra/sync/messages.rs::FileExtraHeader`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FileExtraHeader {
    pub comic_name: String,
    pub kind: String,
    pub file_name: String,
    pub size: u64,
    pub checksum: Option<String>,
    pub blob_hash: Option<String>,
}

/// Message sent in place of the manifest when `FileSyncSessionGuard` already rejected the
/// session locally (peer already has an active session) — schema mirrored on Desktop
/// (`file_handler.rs`), the only way for both sides to speak the same thing without
/// sharing code. The `error` field is the discriminator: no other protocol message
/// uses that key (`FileManifest` uses `comics`, `FileWantList` uses `wanted`), so
/// the reader can detect it before trying to deserialize as `FileManifest`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct SessionBusy {
    pub error: String,
    pub reason: String,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct FileSyncStats {
    pub received_count: u32,
    pub sent_count: u32,
    pub failed_count: u32,
    /// How many requested chapters already existed locally with a checksum different from the peer's —
    /// a real conflict, not just "I did not have it yet" (see `missing_from` in `exchange.rs`).
    /// They are still pulled normally (overwrite behavior unchanged); only
    /// counted separately so `sync:files:complete` can report that it happened.
    pub conflicts_count: u32,
}

/// Groups the flat list from `get_file_manifest()` (FFI) into a `FileManifest` nested by
/// comic — wire format shared with Desktop.
pub(super) fn build_manifest(entries: Vec<FfiFileManifestEntry>) -> FileManifest {
    use std::collections::HashMap;

    let mut by_comic: HashMap<String, Vec<FileChapterInfo>> = HashMap::new();
    for entry in entries {
        by_comic
            .entry(entry.comic_name)
            .or_default()
            .push(FileChapterInfo {
                chapter: entry.chapter,
                file_name: entry.file_name,
                checksum: Some(entry.checksum),
                size: entry.size_bytes,
            });
    }

    let comics = by_comic
        .into_iter()
        .map(|(comic_name, chapters)| FileComicInfo {
            comic_name,
            chapters,
            ..Default::default()
        })
        .collect();

    FileManifest { comics }
}

/// Merges the extra items (cover/banner/`ComicInfo.xml`) from `get_extras_manifest()` (FFI) into
/// the `FileManifest` already built by `build_manifest()` — creates a new comic entry (with no
/// chapters at all) if the only data that exists for it is an extra.
pub(super) fn merge_extras(manifest: &mut FileManifest, extras: Vec<FfiExtraManifestEntry>) {
    use std::collections::HashMap;

    // Indices (not `&mut FileComicInfo` directly) because pushing a new comic further below
    // may reallocate `manifest.comics` — any reference borrowed before the `push` would become
    // invalid. Keys are owned `String`s (not borrowed from `manifest`) precisely so a new entry
    // can be inserted into this same map without conflicting with the `manifest` borrow.
    let mut index_by_comic: HashMap<String, usize> = manifest
        .comics
        .iter()
        .enumerate()
        .map(|(index, comic)| (comic.comic_name.clone(), index))
        .collect();

    for extra in extras {
        let info = FileExtraInfo {
            file_name: extra.file_name,
            checksum: extra.checksum,
            size: extra.size_bytes,
        };

        let index = *index_by_comic
            .entry(extra.comic_name.clone())
            .or_insert_with(|| {
                manifest.comics.push(FileComicInfo {
                    comic_name: extra.comic_name.clone(),
                    ..Default::default()
                });
                manifest.comics.len() - 1
            });

        assign_extra(&mut manifest.comics[index], &extra.kind, info);
    }
}

fn assign_extra(comic: &mut FileComicInfo, kind: &str, info: FileExtraInfo) {
    match kind {
        EXTRA_KIND_COVER => comic.cover = Some(info),
        EXTRA_KIND_BANNER => comic.banner = Some(info),
        EXTRA_KIND_COMIC_INFO => comic.comic_info = Some(info),
        _ => {}
    }
}

#[cfg(test)]
mod wire_contract_tests {
    use super::*;

    /// Locks the `sync-files` wire schema against what Desktop produces/expects
    /// (`acerola-desktop/.../infra/sync/messages.rs`) — the two sides do not share
    /// code, so this is what guarantees they do not silently diverge again.
    #[test]
    fn file_manifest_matches_desktop_wire_shape() {
        let manifest = build_manifest(vec![FfiFileManifestEntry {
            comic_name: "Berserk".into(),
            chapter: "Cap 1".into(),
            file_name: "Cap 1.cbz".into(),
            checksum: "abc123".into(),
            size_bytes: 42,
        }]);

        let value = serde_json::to_value(&manifest).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "comics": [{
                    "comic_name": "Berserk",
                    "chapters": [{
                        "chapter": "Cap 1",
                        "file_name": "Cap 1.cbz",
                        "checksum": "abc123",
                        "size": 42
                    }]
                }]
            })
        );

        // Payload in the exact format Desktop sends ("comics" key, no enum tag)
        // must deserialize without error.
        let desktop_wire = serde_json::json!({
            "comics": [{
                "comic_name": "Berserk",
                "chapters": [{
                    "chapter": "Cap 1",
                    "file_name": "Cap 1.cbz",
                    "checksum": "abc123",
                    "size": 42
                }]
            }]
        });
        let decoded: FileManifest = serde_json::from_value(desktop_wire).unwrap();
        assert_eq!(decoded.comics[0].chapters[0].file_name, "Cap 1.cbz");
    }

    #[test]
    fn file_want_list_serializes_as_tuple_array() {
        let wanted = FileWantList {
            wanted: vec![("Berserk".into(), "Cap 1".into())],
            ..Default::default()
        };
        let value = serde_json::to_value(&wanted).unwrap();
        assert_eq!(
            value,
            serde_json::json!({ "wanted": [["Berserk", "Cap 1"]] })
        );
    }

    /// Locks the wire schema of `FileComicInfo` with the new `cover`/`banner`/`comic_info`
    /// fields against the format mirrored on Desktop
    /// (`infra/sync/messages.rs::FileComicInfo`) — `skip_serializing_if` guarantees that a
    /// comic with no extra item produces the same JSON as before this change.
    #[test]
    fn file_comic_info_matches_desktop_wire_shape_with_extras() {
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

        // Payload from an old peer (Desktop without this protocol version) — deserializes
        // as None, does not fail.
        let old_peer_wire = serde_json::json!({ "comic_name": "Berserk", "chapters": [] });
        let decoded: FileComicInfo = serde_json::from_value(old_peer_wire).unwrap();
        assert!(decoded.cover.is_none());
        assert!(decoded.banner.is_none());
        assert!(decoded.comic_info.is_none());
    }

    /// Locks the wire schema of `FileWantList.wanted_extras` against Desktop.
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

        let old_peer_wire = serde_json::json!({ "wanted": [] });
        let decoded: FileWantList = serde_json::from_value(old_peer_wire).unwrap();
        assert!(decoded.wanted_extras.is_empty());
    }

    /// Locks the wire schema of `ComicSyncScope`/`SyncDirection` against Desktop
    /// (`infra/sync/messages.rs::ComicSyncRequest`/`SyncDirection`) — `"push"`/`"pull"` values
    /// in `snake_case`, with no extra enum tag.
    #[test]
    fn comic_sync_scope_serializes_direction_as_snake_case_string() {
        let scope = ComicSyncScope {
            comic_name: "Berserk".into(),
            direction: SyncDirection::Push,
            chapters: vec![],
        };

        let value = serde_json::to_value(&scope).unwrap();
        assert_eq!(
            value,
            serde_json::json!({ "comic_name": "Berserk", "direction": "push" })
        );

        let desktop_wire = serde_json::json!({ "comic_name": "Berserk", "direction": "pull" });
        let decoded: ComicSyncScope = serde_json::from_value(desktop_wire).unwrap();
        assert_eq!(decoded.direction, SyncDirection::Pull);
        assert!(decoded.chapters.is_empty());
    }

    /// Locks that a session scoped to specific chapters serializes/deserializes the label
    /// list correctly, in the format mirrored on Desktop
    /// (`infra/sync/messages.rs::ComicSyncRequest`).
    #[test]
    fn comic_sync_scope_serializes_chapters_when_scoped() {
        let scope = ComicSyncScope {
            comic_name: "Berserk".into(),
            direction: SyncDirection::Push,
            chapters: vec!["Cap 1".into(), "Cap 2".into()],
        };

        let value = serde_json::to_value(&scope).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "comic_name": "Berserk",
                "direction": "push",
                "chapters": ["Cap 1", "Cap 2"]
            })
        );

        let desktop_wire = serde_json::json!({
            "comic_name": "Berserk",
            "direction": "pull",
            "chapters": ["Cap 1"]
        });
        let decoded: ComicSyncScope = serde_json::from_value(desktop_wire).unwrap();
        assert_eq!(decoded.chapters, vec!["Cap 1".to_string()]);
    }

    /// Locks the wire schema of `FileExtraHeader` against Desktop
    /// (`infra/sync/messages.rs::FileExtraHeader`).
    #[test]
    fn file_extra_header_matches_desktop_wire_shape() {
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
