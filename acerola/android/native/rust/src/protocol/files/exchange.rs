use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use acerola_p2p::api::{error::P2pError, peer::PeerIdentity, protocol::EventEmitter};
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite},
    sync::Semaphore,
    task::JoinSet,
};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use super::{
    model::{
        build_manifest, merge_extras, ComicSyncScope, FileComicInfo, FileExtraHeader, FileHeader,
        FileManifest, FileSyncStats, FileWantList, SessionBusy, SyncDirection, EXTRA_KIND_BANNER,
        EXTRA_KIND_COMIC_INFO, EXTRA_KIND_COVER,
    },
    transfer::ChapterTransfer,
};
use crate::{
    callbacks::FileSyncProvider,
    protocol::{ffi_blocking::run_blocking, sync_error::SyncErrorCode},
};

/// Value of the `error` field in `SessionBusy` — same literal used by Desktop
/// (`file_handler.rs`); it is what both sides use to recognize this specific message
/// instead of a regular `FileManifest`.
pub(super) const SESSION_BUSY_TAG: &str = "busy";

const FRAME_READ_TIMEOUT: Duration = Duration::from_secs(30);
const CHUNK_SIZE: usize = 64 * 1024;
const PROGRESS_EVERY_BYTES: u64 = 1024 * 1024;

pub(super) use crate::protocol::framing::{Recv, Writer};

/// Thin layer over `framing::write_json` — only exists to keep the name already used across the
/// rest of this file without touching every call site after the consolidation (see
/// `protocol::framing`, which used to be reimplemented identically here, in `history`,
/// `library_browse` and `cover_browse`).
async fn write_json<T: Serialize>(send: &mut Writer, value: &T) -> Result<(), P2pError> {
    crate::protocol::framing::write_json(send, value).await
}

/// Same, over `framing::read_json` — `FRAME_READ_TIMEOUT` is still THIS protocol's own
/// setting; only the mechanics of reading/classifying the frame are shared now.
async fn read_json<T: DeserializeOwned>(recv: &mut Recv) -> Result<T, P2pError> {
    crate::protocol::framing::read_json(recv, FRAME_READ_TIMEOUT).await
}

/// Writes the session rejection message straight to the stream, in place of the manifest —
/// used when `FileSyncSessionGuard::try_acquire` already blocked the session in `run_and_report`,
/// before `run_exchange` is called. Takes the raw `send` (not an assembled `Writer`) because
/// that is exactly the state `run_and_report` is in at the moment of rejection: the stream
/// has not been wrapped in any framing yet.
pub(super) async fn write_session_busy(
    send: Box<dyn AsyncWrite + Send + Unpin>,
    reason: &str,
) -> Result<(), P2pError> {
    let mut writer: Writer = FramedWrite::new(send, LengthDelimitedCodec::new());
    write_json(
        &mut writer,
        &SessionBusy {
            error: SESSION_BUSY_TAG.to_string(),
            reason: reason.to_string(),
        },
    )
    .await
}

/// Peeks at the raw JSON and returns `Err("peer busy: ...")` if it is a `SessionBusy` — extracted
/// from `read_manifest_or_busy`/`read_comic_scope_or_busy` to avoid duplicating the
/// `error == "busy"` check at every protocol point that can legitimately receive this rejection
/// instead of the expected message. If not busy, returns the raw `value` for the caller to
/// deserialize into the right concrete type (`FileManifest`, `ComicSyncScope`, etc.).
async fn peek_busy(reader: &mut Recv) -> Result<serde_json::Value, P2pError> {
    let value: serde_json::Value = read_json(reader).await?;

    let is_busy = value.get("error").and_then(|tag| tag.as_str()) == Some(SESSION_BUSY_TAG);
    if is_busy {
        let reason = value
            .get("reason")
            .and_then(|r| r.as_str())
            .unwrap_or("peer is busy");
        return Err(P2pError::StreamFailed(format!("peer busy: {reason}")));
    }

    Ok(value)
}

/// Reads the peer's first message, which may be either the regular `FileManifest` or a
/// `SessionBusy` rejection — the only way to tell which without an enum tag on the wire (a rule
/// already established by the rest of the protocol) is to peek at the raw JSON before picking the
/// concrete type. Only the initial read points in `exchange_manifests`/`exchange_comic_scope`
/// use this: after the first message, an accepted session never sends `SessionBusy` again.
async fn read_manifest_or_busy(reader: &mut Recv) -> Result<FileManifest, P2pError> {
    let value = peek_busy(reader).await?;
    serde_json::from_value(value)
        .map_err(|err| P2pError::StreamFailed(format!("failed to decode file sync message: {err}")))
}

/// Same idea as `read_manifest_or_busy`, for `ComicSyncScope` — phase 0, exclusive to
/// `acerola/sync-comic/1`, read BEFORE even the manifest. Real regression (05/09/2026): without
/// this, when the dialing side was already busy with another session for the same peer and replied
/// with `SessionBusy` instead of `ComicSyncScope` (see `run_and_report_scoped`/`ComicSyncOutbound::handle`
/// on Desktop, which do exactly that), this side tried to deserialize `{"error":"busy",...}`
/// directly as `ComicSyncScope` and broke with "missing field `comic_name`" instead of reporting
/// "peer busy" — the only initial read point of the protocol that did not use the same
/// peek-before-decode that `exchange_manifests` already used on both sides.
async fn read_comic_scope_or_busy(reader: &mut Recv) -> Result<ComicSyncScope, P2pError> {
    let value = peek_busy(reader).await?;
    serde_json::from_value(value)
        .map_err(|err| P2pError::StreamFailed(format!("failed to decode file sync message: {err}")))
}

// --- FFI boundary (Kotlin): one function per `FileSyncProvider` method, each one just
// cloning the `Arc` and delegating to the blocking thread pool via `run_blocking` — never
// directly on the async runtime's worker thread. They only exist so `send_one_chapter`/
// `receive_one_chapter` do not repeat the `{ Arc::clone + run_blocking }` block on every call.

async fn open_chapter_for_read(
    provider: &Arc<dyn FileSyncProvider>,
    comic_name: String,
    chapter: String,
) -> Result<i64, P2pError> {
    let provider = Arc::clone(provider);
    run_blocking(move || provider.open_chapter_for_read(comic_name, chapter)).await
}

async fn read_chapter_chunk(
    provider: &Arc<dyn FileSyncProvider>,
    handle: i64,
) -> Result<Vec<u8>, P2pError> {
    let provider = Arc::clone(provider);
    run_blocking(move || provider.read_chapter_chunk(handle, CHUNK_SIZE as u32)).await
}

async fn close_read_handle(
    provider: &Arc<dyn FileSyncProvider>,
    handle: i64,
) -> Result<(), P2pError> {
    let provider = Arc::clone(provider);
    run_blocking(move || provider.close_read_handle(handle)).await
}

async fn begin_chapter_write(
    provider: &Arc<dyn FileSyncProvider>,
    header: &FileHeader,
) -> Result<i64, P2pError> {
    let provider = Arc::clone(provider);
    let (comic_name, chapter, file_name, checksum, size) = (
        header.comic_name.clone(),
        header.chapter.clone(),
        header.file_name.clone(),
        header.checksum.clone().unwrap_or_default(),
        header.size,
    );

    // Strategic log 3/4: last point on the Rust side before the checksum crosses the FFI
    // back to Kotlin (`beginChapterWrite`), where it is kept in `WriteHandle` until
    // `finalizeChapterWrite` compares it against the freshly computed hash of the received file.
    // If this value matches log 2/4 (same comic/chapter) but Kotlin still reports a
    // mismatch, the corruption is entirely on the Kotlin side (write/re-read),
    // beyond what can be instrumented here.
    tracing::debug!(
        comic = %comic_name,
        chapter = %chapter,
        checksum = %checksum,
        "sync-files: passing expected_checksum to Rust->Kotlin FFI (begin_chapter_write)",
    );

    run_blocking(move || {
        provider.begin_chapter_write(comic_name, chapter, file_name, checksum, size)
    })
    .await
}

async fn write_chapter_chunk(
    provider: &Arc<dyn FileSyncProvider>,
    handle: i64,
    chunk: Vec<u8>,
) -> Result<bool, P2pError> {
    let provider = Arc::clone(provider);
    run_blocking(move || provider.write_chapter_chunk(handle, chunk)).await
}

async fn finalize_chapter_write(
    provider: &Arc<dyn FileSyncProvider>,
    handle: i64,
) -> Result<bool, P2pError> {
    let provider = Arc::clone(provider);
    run_blocking(move || provider.finalize_chapter_write(handle)).await
}

async fn abort_chapter_write(
    provider: &Arc<dyn FileSyncProvider>,
    handle: i64,
) -> Result<(), P2pError> {
    let provider = Arc::clone(provider);
    run_blocking(move || provider.abort_chapter_write(handle)).await
}

async fn build_local_manifest(
    provider: &Arc<dyn FileSyncProvider>,
) -> Result<FileManifest, P2pError> {
    let entries = run_blocking({
        let provider = Arc::clone(provider);
        move || provider.get_file_manifest()
    })
    .await?;
    let mut manifest = build_manifest(entries);

    let extras = run_blocking({
        let provider = Arc::clone(provider);
        move || provider.get_extras_manifest()
    })
    .await?;
    merge_extras(&mut manifest, extras);

    Ok(manifest)
}

async fn open_extra_for_read(
    provider: &Arc<dyn FileSyncProvider>,
    comic_name: String,
    kind: String,
) -> Result<i64, P2pError> {
    let provider = Arc::clone(provider);
    run_blocking(move || provider.open_extra_for_read(comic_name, kind)).await
}

async fn begin_extra_write(
    provider: &Arc<dyn FileSyncProvider>,
    header: &FileExtraHeader,
) -> Result<i64, P2pError> {
    let provider = Arc::clone(provider);
    let (comic_name, kind, file_name, checksum, size) = (
        header.comic_name.clone(),
        header.kind.clone(),
        header.file_name.clone(),
        header.checksum.clone().unwrap_or_default(),
        header.size,
    );
    run_blocking(move || provider.begin_extra_write(comic_name, kind, file_name, checksum, size))
        .await
}

async fn finalize_extra_write(
    provider: &Arc<dyn FileSyncProvider>,
    handle: i64,
) -> Result<bool, P2pError> {
    let provider = Arc::clone(provider);
    run_blocking(move || provider.finalize_extra_write(handle)).await
}

async fn abort_extra_write(
    provider: &Arc<dyn FileSyncProvider>,
    handle: i64,
) -> Result<(), P2pError> {
    let provider = Arc::clone(provider);
    run_blocking(move || provider.abort_extra_write(handle)).await
}

/// Returns `(missing, conflicts)`: `missing` is every `(comic_name, chapter)` that `peer` offers
/// and `local` does not have with the same content (absent entirely, OR present with a different
/// checksum — still pulled/overwritten normally, behavior unchanged).
/// `conflicts` is the subset of `missing` where the chapter already EXISTED locally with a
/// recorded checksum, just different from the peer's — both sides truly diverged, it is not
/// just "I never had this chapter". Mirrors Desktop's `FileSyncService::diff_wanted`.
fn missing_from(
    local: &FileManifest, peer: &FileManifest,
) -> (Vec<(String, String)>, Vec<(String, String)>) {
    let local_checksums: HashMap<(&str, &str), Option<&str>> = local
        .comics
        .iter()
        .flat_map(|comic| {
            comic.chapters.iter().map(move |chapter| {
                (
                    (comic.comic_name.as_str(), chapter.chapter.as_str()),
                    chapter.checksum.as_deref(),
                )
            })
        })
        .collect();

    let mut missing = Vec::new();
    let mut conflicts = Vec::new();

    for comic in &peer.comics {
        for chapter in &comic.chapters {
            let key = (comic.comic_name.as_str(), chapter.chapter.as_str());
            let local_checksum = local_checksums.get(&key);
            let is_missing = match local_checksum {
                Some(checksum) => *checksum != chapter.checksum.as_deref(),
                None => true,
            };
            if !is_missing {
                continue;
            }
            if local_checksum.is_some_and(|checksum| checksum.is_some()) {
                conflicts.push((comic.comic_name.clone(), chapter.chapter.clone()));
            }
            missing.push((comic.comic_name.clone(), chapter.chapter.clone()));
        }
    }

    (missing, conflicts)
}

fn manifest_index(
    manifest: &FileManifest,
) -> HashMap<(&str, &str), &super::model::FileChapterInfo> {
    manifest
        .comics
        .iter()
        .flat_map(|comic: &FileComicInfo| {
            comic.chapters.iter().map(move |chapter| {
                (
                    (comic.comic_name.as_str(), chapter.chapter.as_str()),
                    chapter,
                )
            })
        })
        .collect()
}

/// Returns `(comic_name, kind)` of extra items (cover/banner/`ComicInfo.xml`) that `peer` offers
/// and `local` does not have (absent, or with a different checksum) — same logic as `missing_from`,
/// applied to the three optional fields of `FileComicInfo` instead of the chapter list.
fn missing_extras_from(local: &FileManifest, peer: &FileManifest) -> Vec<(String, String)> {
    let local_by_key = extras_index(local);

    peer.comics
        .iter()
        .flat_map(|comic| {
            let local_by_key = &local_by_key;
            EXTRA_KINDS.iter().filter_map(move |kind| {
                let peer_extra = extra_for_kind(comic, kind)?;
                let key = (comic.comic_name.as_str(), *kind);
                let missing = match local_by_key.get(&key) {
                    Some(local_extra) => local_extra.checksum != peer_extra.checksum,
                    None => true,
                };
                missing.then(|| (comic.comic_name.clone(), kind.to_string()))
            })
        })
        .collect()
}

const EXTRA_KINDS: [&str; 3] = [EXTRA_KIND_COVER, EXTRA_KIND_BANNER, EXTRA_KIND_COMIC_INFO];

fn extra_for_kind<'a>(
    comic: &'a FileComicInfo,
    kind: &str,
) -> Option<&'a super::model::FileExtraInfo> {
    match kind {
        EXTRA_KIND_COVER => comic.cover.as_ref(),
        EXTRA_KIND_BANNER => comic.banner.as_ref(),
        EXTRA_KIND_COMIC_INFO => comic.comic_info.as_ref(),
        _ => None,
    }
}

fn extras_index(manifest: &FileManifest) -> HashMap<(&str, &str), &super::model::FileExtraInfo> {
    manifest
        .comics
        .iter()
        .flat_map(|comic| {
            EXTRA_KINDS.iter().filter_map(move |kind| {
                extra_for_kind(comic, kind).map(|extra| ((comic.comic_name.as_str(), *kind), extra))
            })
        })
        .collect()
}

async fn write_missing_header(
    writer: &mut Writer,
    comic_name: &str,
    chapter: &str,
) -> Result<(), P2pError> {
    write_json(
        writer,
        &FileHeader {
            comic_name: comic_name.to_string(),
            chapter: chapter.to_string(),
            file_name: String::new(),
            size: 0,
            checksum: None,
            blob_hash: None,
        },
    )
    .await
}

async fn write_missing_extra_header(
    writer: &mut Writer,
    comic_name: &str,
    kind: &str,
) -> Result<(), P2pError> {
    write_json(
        writer,
        &FileExtraHeader {
            comic_name: comic_name.to_string(),
            kind: kind.to_string(),
            file_name: String::new(),
            size: 0,
            checksum: None,
            blob_hash: None,
        },
    )
    .await
}

/// Sends a single requested extra item — same mechanics as `send_one_chapter`
/// (opens via `open_extra_for_read`, publishes to the blob store via `publish_chapter_to_blob`,
/// which is already agnostic to chapter vs. extra, closes via the reused `close_read_handle`),
/// with `FileExtraHeader` instead of `FileHeader`.
async fn send_one_extra(
    writer: &mut Writer,
    provider: &Arc<dyn FileSyncProvider>,
    comic_name: &str,
    kind: &str,
    entry: &super::model::FileExtraInfo,
    emit: &EventEmitter,
    peer: &PeerIdentity,
    transfer: &Arc<dyn ChapterTransfer>,
) -> Result<bool, P2pError> {
    let handle = open_extra_for_read(provider, comic_name.to_string(), kind.to_string()).await?;
    if handle < 0 {
        write_missing_extra_header(writer, comic_name, kind).await?;
        return Ok(false);
    }

    let publish_result = publish_chapter_to_blob(
        provider, handle, entry.size, emit, peer, comic_name, kind, transfer,
    )
    .await;
    close_read_handle(provider, handle).await?;
    let blob_hash = publish_result?;

    write_json(
        writer,
        &FileExtraHeader {
            comic_name: comic_name.to_string(),
            kind: kind.to_string(),
            file_name: entry.file_name.clone(),
            size: entry.size,
            checksum: Some(entry.checksum.clone()),
            blob_hash: Some(blob_hash),
        },
    )
    .await?;

    Ok(true)
}

/// Sends the requested extra items — same logic as `send_files`, looking at `cover`/`banner`/
/// `comic_info` of `local_manifest` instead of the chapter list.
async fn send_extras(
    writer: &mut Writer,
    provider: &Arc<dyn FileSyncProvider>,
    local_manifest: &FileManifest,
    requested: &[(String, String)],
    emit: &EventEmitter,
    peer: &PeerIdentity,
    stats: &mut FileSyncStats,
    transfer: &Arc<dyn ChapterTransfer>,
) -> Result<(), P2pError> {
    let by_comic: HashMap<&str, &FileComicInfo> = local_manifest
        .comics
        .iter()
        .map(|comic| (comic.comic_name.as_str(), comic))
        .collect();

    for (comic_name, kind) in requested {
        let entry = by_comic
            .get(comic_name.as_str())
            .and_then(|comic| extra_for_kind(comic, kind));
        let Some(entry) = entry else {
            write_missing_extra_header(writer, comic_name, kind).await?;
            continue;
        };

        if send_one_extra(
            writer, provider, comic_name, kind, entry, emit, peer, transfer,
        )
        .await?
        {
            stats.sent_count += 1;
        }
    }

    Ok(())
}

/// Fetches the blob of an extra item and writes it chunk by chunk — same mechanics as
/// `receive_chapter_from_blob`, with `FileExtraHeader` instead of `FileHeader`.
async fn receive_extra_from_blob(
    provider: &Arc<dyn FileSyncProvider>,
    handle: i64,
    header: &FileExtraHeader,
    emit: &EventEmitter,
    peer: &PeerIdentity,
    transfer: &Arc<dyn ChapterTransfer>,
) -> Result<bool, P2pError> {
    let Some(blob_hash) = header.blob_hash.as_deref() else {
        return Ok(true);
    };

    let mut blob_reader = match transfer.fetch_reader(blob_hash, peer).await {
        Ok(reader) => reader,
        Err(err) => {
            tracing::warn!(
                comic = %header.comic_name,
                kind = %header.kind,
                blob_hash,
                error = %err,
                "sync-files: failed to fetch extra blob from peer",
            );
            return Ok(true);
        }
    };

    let mut received_bytes: u64 = 0;
    let mut bytes_since_progress: u64 = 0;
    let mut write_failed = handle < 0;
    let mut buf = vec![0u8; CHUNK_SIZE];

    loop {
        let n = blob_reader.read(&mut buf).await.map_err(|err| {
            P2pError::StreamFailed(format!("failed to read fetched extra blob: {err}"))
        })?;
        if n == 0 {
            break;
        }

        received_bytes += n as u64;
        bytes_since_progress += n as u64;

        if !write_failed && !write_chapter_chunk(provider, handle, buf[..n].to_vec()).await? {
            write_failed = true;
        }

        if bytes_since_progress >= PROGRESS_EVERY_BYTES {
            bytes_since_progress = 0;
            emit_progress(
                emit,
                peer,
                &header.comic_name,
                &header.kind,
                received_bytes,
                header.size,
            );
        }
    }

    Ok(write_failed)
}

fn extra_complete_payload(peer: &PeerIdentity, header: &FileExtraHeader) -> String {
    serde_json::json!({ "peerId": peer.id, "comicName": header.comic_name, "kind": header.kind })
        .to_string()
}

/// See `chapter_failed_payload` docs — same I/O vs. checksum mismatch distinction, applied to
/// extra items (cover/banner/`ComicInfo.xml`).
fn extra_failed_payload(
    peer: &PeerIdentity,
    header: &FileExtraHeader,
    write_failed: bool,
) -> String {
    let (reason, code) = if write_failed {
        ("write or blob-fetch I/O failure", None)
    } else {
        (
            "checksum mismatch — file corrupted in transit",
            Some(SyncErrorCode::ChecksumMismatch),
        )
    };
    serde_json::json!({
        "peerId": peer.id,
        "comicName": header.comic_name,
        "kind": header.kind,
        "reason": reason,
        "code": code,
    })
    .to_string()
}

async fn receive_one_extra(
    provider: &Arc<dyn FileSyncProvider>,
    emit: &EventEmitter,
    peer: &PeerIdentity,
    header: &FileExtraHeader,
    stats: &mut FileSyncStats,
    transfer: &Arc<dyn ChapterTransfer>,
) -> Result<(), P2pError> {
    let handle = begin_extra_write(provider, header).await?;
    let write_failed =
        receive_extra_from_blob(provider, handle, header, emit, peer, transfer).await?;

    let succeeded = if !write_failed && handle >= 0 {
        finalize_extra_write(provider, handle).await?
    } else {
        false
    };

    if succeeded {
        stats.received_count += 1;
        emit(
            "sync:files:extra_complete",
            extra_complete_payload(peer, header),
        );
    } else {
        if handle >= 0 {
            abort_extra_write(provider, handle).await?;
        }
        stats.failed_count += 1;
        emit(
            "sync:files:extra_failed",
            extra_failed_payload(peer, header, write_failed),
        );
    }

    Ok(())
}

/// Receives exactly `expected_count` extra items — same logic as `receive_files`, with
/// `FileExtraHeader` instead of `FileHeader`.
async fn receive_extras(
    reader: &mut Recv,
    expected_count: usize,
    provider: &Arc<dyn FileSyncProvider>,
    emit: &EventEmitter,
    peer: &PeerIdentity,
    stats: &mut FileSyncStats,
    transfer: &Arc<dyn ChapterTransfer>,
) -> Result<(), P2pError> {
    for _ in 0..expected_count {
        let header: FileExtraHeader = read_json(reader).await?;

        if header.size == 0 {
            continue;
        }

        receive_one_extra(provider, emit, peer, &header, stats, transfer).await?;
    }

    Ok(())
}

/// Reads the already opened chapter (valid `handle`) one chunk at a time and publishes the
/// accumulated bytes to the local blob store (`ChapterTransfer::publish`) — returns the blob hash
/// (hex) that goes into the `FileHeader`. Reading itself is still chunk by chunk (never breaks the
/// existing progress loop), only the bytes' destination changes: they used to go to the wire, now
/// they go to `put()`.
async fn publish_chapter_to_blob(
    provider: &Arc<dyn FileSyncProvider>,
    handle: i64,
    total_size: u64,
    emit: &EventEmitter,
    peer: &PeerIdentity,
    comic_name: &str,
    chapter: &str,
    transfer: &Arc<dyn ChapterTransfer>,
) -> Result<String, P2pError> {
    let mut bytes = Vec::new();
    let mut read_bytes_total: u64 = 0;
    let mut bytes_since_progress: u64 = 0;

    while read_bytes_total < total_size {
        let chunk = read_chapter_chunk(provider, handle).await?;
        if chunk.is_empty() {
            break;
        }

        read_bytes_total += chunk.len() as u64;
        bytes_since_progress += chunk.len() as u64;
        bytes.extend_from_slice(&chunk);

        if bytes_since_progress >= PROGRESS_EVERY_BYTES {
            bytes_since_progress = 0;
            emit_progress(
                emit,
                peer,
                comic_name,
                chapter,
                read_bytes_total,
                total_size,
            );
        }
    }

    transfer.publish(bytes).await
}

/// Sends a single requested chapter: opens, publishes the bytes to the local blob store, writes
/// the header with the resulting hash, closes. `Ok(false)` signals "could not serve" (item gone or
/// invalid handle) — empty header already written, nothing published.
async fn send_one_chapter(
    writer: &mut Writer,
    provider: &Arc<dyn FileSyncProvider>,
    comic_name: &str,
    chapter: &str,
    entry: &super::model::FileChapterInfo,
    emit: &EventEmitter,
    peer: &PeerIdentity,
    transfer: &Arc<dyn ChapterTransfer>,
) -> Result<bool, P2pError> {
    let handle =
        open_chapter_for_read(provider, comic_name.to_string(), chapter.to_string()).await?;
    if handle < 0 {
        write_missing_header(writer, comic_name, chapter).await?;
        return Ok(false);
    }

    let publish_result = publish_chapter_to_blob(
        provider, handle, entry.size, emit, peer, comic_name, chapter, transfer,
    )
    .await;
    close_read_handle(provider, handle).await?;
    let blob_hash = publish_result?;

    tracing::debug!(
        comic = %comic_name,
        chapter = %chapter,
        blob_hash = %blob_hash,
        size = entry.size,
        "sync-files: published to local blob store, sending header",
    );

    write_json(
        writer,
        &FileHeader {
            comic_name: comic_name.to_string(),
            chapter: chapter.to_string(),
            file_name: entry.file_name.clone(),
            size: entry.size,
            checksum: entry.checksum.clone(),
            blob_hash: Some(blob_hash),
        },
    )
    .await?;

    Ok(true)
}

/// Sends the requested chapters, in the requested order. An item that no longer exists locally
/// (race with a concurrent deletion) sends a `FileHeader` with `size: 0` instead of being
/// simply skipped — the other side counts exactly `requested.len()` headers, so
/// the count must stay deterministic even when an item cannot be served.
async fn send_files(
    writer: &mut Writer,
    provider: &Arc<dyn FileSyncProvider>,
    local_manifest: &FileManifest,
    requested: &[(String, String)],
    emit: &EventEmitter,
    peer: &PeerIdentity,
    stats: &mut FileSyncStats,
    transfer: &Arc<dyn ChapterTransfer>,
) -> Result<(), P2pError> {
    let by_key = manifest_index(local_manifest);

    for (comic_name, chapter) in requested {
        let Some(entry) = by_key.get(&(comic_name.as_str(), chapter.as_str())) else {
            write_missing_header(writer, comic_name, chapter).await?;
            continue;
        };

        if send_one_chapter(
            writer, provider, comic_name, chapter, entry, emit, peer, transfer,
        )
        .await?
        {
            stats.sent_count += 1;
        }
    }

    Ok(())
}

/// How many chapters to fetch in parallel, based on the RTT measured NOW with the peer (see
/// `ChapterTransfer::latency`) — same calculation as Desktop
/// (`infra/sync/protocol/transfer.rs::chapter_fetch_concurrency`, see the docs there for the full
/// reasoning): a slow link (distant relay) needs several fetches in flight at once
/// to avoid idling between one round-trip and the next; LAN/mDNS already saturates with very little
/// parallelism. Without a measurement it falls back to the most conservative of the three.
fn chapter_fetch_concurrency(latency: Option<Duration>) -> usize {
    match latency {
        Some(rtt) if rtt <= Duration::from_millis(20) => 2,
        Some(rtt) if rtt <= Duration::from_millis(200) => 4,
        Some(_) => 6,
        None => 2,
    }
}

/// See the docs of the Desktop equivalent (`transfer.rs::permits_to_forget`) — same calculation:
/// halves the pool the first time a fetch times out, never below 1.
fn permits_to_forget(initial_permits: usize) -> usize {
    initial_permits.saturating_sub((initial_permits / 2).max(1))
}

/// Result of processing ONE header already read from the wire — extracted from `receive_files` so
/// it can run as an independent task (`tasks.spawn`), in parallel with reading the NEXT
/// header, instead of blocking the whole loop on fetching+writing a single chapter at a time.
enum ChapterOutcome {
    Received,
    /// `timed_out: true` only when the cause was specifically `P2pError::Timeout` while fetching
    /// the blob (`ChapterTransfer::fetch_reader`) — the only signal `receive_files` uses to
    /// reduce parallelism for the rest of the session. Mirrors Desktop's
    /// `ChapterOutcome::Skipped { timed_out }`.
    Failed {
        timed_out: bool,
    },
}

/// Receives exactly `expected_count` chapters (the same count this side requested in
/// `FileWantList`). Each header is read sequentially from the wire (a single ordered stream, header
/// 2 cannot be read before 1), but fetching+writing the blob it references runs in its own
/// task, in a pool of up to `chapter_fetch_concurrency(...)` concurrent chapters — same
/// design as Desktop (`infra/sync/protocol/transfer.rs::receive_files`), motivated by the same
/// real problem: a sync over a high-RTT relay connection taking minutes, one chapter at a
/// time, because the loop blocked on the whole fetch of ONE chapter before even reading the next
/// header.
///
/// Before this change, any failure in `begin_chapter_write`/`receive_chapter_from_blob`/
/// `finalize_chapter_write` (`?`) aborted the WHOLE session. Now every per-chapter failure is
/// non-fatal (`ChapterOutcome::Failed`, counted in `stats.failed_count` + a per-item
/// `chapter_failed` event) — same philosophy Desktop has used since the bug of
/// 22/08/2026. Needed here specifically because of parallelism: aborting the session on the
/// first error would leave SIBLING tasks, still running, no chance to run their own
/// `finalize_chapter_write`/`abort_chapter_write` — an orphaned `WriteHandle` (Kotlin side) in the
/// `ConcurrentHashMap`, with `OutputStream`/temp file never closed.
async fn receive_files(
    reader: &mut Recv,
    expected_count: usize,
    provider: &Arc<dyn FileSyncProvider>,
    emit: &EventEmitter,
    peer: &PeerIdentity,
    stats: &mut FileSyncStats,
    transfer: &Arc<dyn ChapterTransfer>,
) -> Result<(), P2pError> {
    let initial_permits = chapter_fetch_concurrency(transfer.latency(peer).await);
    let semaphore = Arc::new(Semaphore::new(initial_permits));
    // `backed_off` ensures the `permits_to_forget` cut only fires once per session.
    let backed_off = Arc::new(AtomicBool::new(false));
    let permits_to_forget_on_timeout = permits_to_forget(initial_permits);

    let mut tasks = JoinSet::new();

    for _ in 0..expected_count {
        let header: FileHeader = read_json(reader).await?;

        if header.size == 0 {
            continue;
        }

        tracing::debug!(
            comic = %header.comic_name,
            chapter = %header.chapter,
            blob_hash = %header.blob_hash.as_deref().unwrap_or("<none>"),
            size = header.size,
            "sync-files: header received from wire",
        );

        let permit = Arc::clone(&semaphore)
            .acquire_owned()
            .await
            .expect("semaphore is never closed while receive_files is running");
        let semaphore_for_backoff = Arc::clone(&semaphore);
        let backed_off = Arc::clone(&backed_off);
        let task_provider = Arc::clone(provider);
        let task_emit = Arc::clone(emit);
        let task_peer = peer.clone();
        let task_transfer = Arc::clone(transfer);

        tasks.spawn(async move {
            let _permit = permit;
            let outcome =
                receive_one_chapter(task_provider, task_emit, task_peer, header, task_transfer)
                    .await;

            if matches!(outcome, ChapterOutcome::Failed { timed_out: true })
                && backed_off
                    .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                && permits_to_forget_on_timeout > 0
            {
                semaphore_for_backoff.forget_permits(permits_to_forget_on_timeout);
            }

            outcome
        });
    }

    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(ChapterOutcome::Received) => stats.received_count += 1,
            Ok(ChapterOutcome::Failed { .. }) => stats.failed_count += 1,
            // Only reached if the task itself panicked (a real bug, not a network/protocol error) —
            // no branch of `receive_one_chapter` propagates an error, it only returns the enum.
            Err(join_err) => {
                tracing::error!(
                    error = %join_err,
                    "sync-files: chapter fetch task panicked — counting as failure"
                );
                stats.failed_count += 1;
            }
        }
    }

    Ok(())
}

/// Result of `receive_chapter_from_blob` — `write_failed` keeps its original meaning (generic
/// write or blob-fetch I/O); `timed_out` signals specifically that the CAUSE
/// was `P2pError::Timeout` during the fetch (not any other step), the only signal
/// `receive_files` uses to decide the parallelism backoff.
struct BlobFetchOutcome {
    write_failed: bool,
    timed_out: bool,
}

/// Fetches the blob referenced by `header.blob_hash` from the peer (`ChapterTransfer::fetch_reader`)
/// and writes the result chunk by chunk via `FileSyncProvider::write_chapter_chunk`, keeping the
/// same progress events as before — only the bytes' source changed (it was the raw wire, now it is
/// the `AsyncRead` returned by the blob store after `fetch`). An invalid `handle` counts as an
/// immediate failure.
async fn receive_chapter_from_blob(
    provider: &Arc<dyn FileSyncProvider>,
    handle: i64,
    header: &FileHeader,
    emit: &EventEmitter,
    peer: &PeerIdentity,
    transfer: &Arc<dyn ChapterTransfer>,
) -> Result<BlobFetchOutcome, P2pError> {
    let Some(blob_hash) = header.blob_hash.as_deref() else {
        return Ok(BlobFetchOutcome {
            write_failed: true,
            timed_out: false,
        });
    };

    let mut blob_reader = match transfer.fetch_reader(blob_hash, peer).await {
        Ok(reader) => reader,
        Err(err) => {
            tracing::warn!(
                comic = %header.comic_name,
                chapter = %header.chapter,
                blob_hash,
                error = %err,
                "sync-files: failed to fetch blob from peer",
            );
            return Ok(BlobFetchOutcome {
                write_failed: true,
                timed_out: matches!(err, P2pError::Timeout),
            });
        }
    };

    let mut received_bytes: u64 = 0;
    let mut bytes_since_progress: u64 = 0;
    let mut write_failed = handle < 0;
    let mut buf = vec![0u8; CHUNK_SIZE];

    loop {
        let n = blob_reader
            .read(&mut buf)
            .await
            .map_err(|err| P2pError::StreamFailed(format!("failed to read fetched blob: {err}")))?;
        if n == 0 {
            break;
        }

        received_bytes += n as u64;
        bytes_since_progress += n as u64;

        if !write_failed && !write_chapter_chunk(provider, handle, buf[..n].to_vec()).await? {
            write_failed = true;
        }

        if bytes_since_progress >= PROGRESS_EVERY_BYTES {
            bytes_since_progress = 0;
            emit_progress(
                emit,
                peer,
                &header.comic_name,
                &header.chapter,
                received_bytes,
                header.size,
            );
        }
    }

    Ok(BlobFetchOutcome {
        write_failed,
        timed_out: false,
    })
}

fn chapter_complete_payload(peer: &PeerIdentity, header: &FileHeader) -> String {
    serde_json::json!({ "peerId": peer.id, "comicName": header.comic_name, "chapter": header.chapter }).to_string()
}

/// `write_failed`: `true` when `write_chapter_chunk` (or the blob fetch) failed on some
/// chunk — a generic I/O/network failure. `false` means ALL chunks were written
/// without error, and still `finalize_chapter_write` returned `false` — at that point the only
/// thing `finalizeChapterWrite` does on the Kotlin side is re-read the file and compare the hash
/// against `expected_checksum` (see strategic log 4/4 above), so this IS specifically a
/// checksum mismatch, not just any I/O failure — reported with a recognizable `code`
/// (`SyncErrorCode::ChecksumMismatch`) instead of the generic fallback without `code`.
fn chapter_failed_payload(peer: &PeerIdentity, header: &FileHeader, write_failed: bool) -> String {
    let (reason, code) = if write_failed {
        ("write or blob-fetch I/O failure", None)
    } else {
        (
            "checksum mismatch — file corrupted in transit",
            Some(SyncErrorCode::ChecksumMismatch),
        )
    };
    serde_json::json!({
        "peerId": peer.id,
        "comicName": header.comic_name,
        "chapter": header.chapter,
        "reason": reason,
        "code": code,
    })
    .to_string()
}

/// Processes ONE header completely (begin -> fetch/write -> finalize/abort), always
/// returning an outcome — never propagates an error (see `receive_files` docs for why: with
/// several tasks in flight at once, aborting the whole session on the first error would leave
/// sibling tasks no chance to close their own Kotlin-side `WriteHandle`). Takes everything by
/// value (instead of by reference) so it can run as an independent task via `tasks.spawn`.
async fn receive_one_chapter(
    provider: Arc<dyn FileSyncProvider>,
    emit: EventEmitter,
    peer: PeerIdentity,
    header: FileHeader,
    transfer: Arc<dyn ChapterTransfer>,
) -> ChapterOutcome {
    let handle = match begin_chapter_write(&provider, &header).await {
        Ok(handle) => handle,
        Err(err) => {
            tracing::error!(
                comic = %header.comic_name,
                chapter = %header.chapter,
                error = %err,
                "sync-files: begin_chapter_write failed — skipping chapter",
            );
            emit(
                "sync:files:chapter_failed",
                chapter_failed_payload(&peer, &header, true),
            );
            return ChapterOutcome::Failed { timed_out: false };
        }
    };

    let fetch_outcome = match receive_chapter_from_blob(
        &provider, handle, &header, &emit, &peer, &transfer,
    )
    .await
    {
        Ok(outcome) => outcome,
        Err(err) => {
            tracing::error!(
                comic = %header.comic_name,
                chapter = %header.chapter,
                error = %err,
                "sync-files: I/O failure while fetching/writing blob — skipping chapter",
            );
            if handle >= 0 {
                if let Err(abort_err) = abort_chapter_write(&provider, handle).await {
                    tracing::warn!(error = %abort_err, "sync-files: abort_chapter_write failed");
                }
            }
            emit(
                "sync:files:chapter_failed",
                chapter_failed_payload(&peer, &header, true),
            );
            return ChapterOutcome::Failed { timed_out: false };
        }
    };

    let succeeded = if !fetch_outcome.write_failed && handle >= 0 {
        match finalize_chapter_write(&provider, handle).await {
            Ok(succeeded) => succeeded,
            Err(err) => {
                tracing::error!(
                    comic = %header.comic_name,
                    chapter = %header.chapter,
                    error = %err,
                    "sync-files: finalize_chapter_write failed — skipping chapter",
                );
                false
            }
        }
    } else {
        false
    };

    // Strategic log 4/4: final result of the checksum verification, which happens
    // entirely inside Kotlin (`finalizeChapterWrite` re-reads the written file and
    // compares it with `expectedChecksum`). Rust only sees the boolean back — not the actual
    // hash computed on the Kotlin side — so this log shows the `expected_checksum` we
    // had (log 3/4) next to the result, but does not prove on its own where the difference
    // is; it serves to confirm WHICH chapter failed and correlate with logs 1-3.
    tracing::debug!(
        comic = %header.comic_name,
        chapter = %header.chapter,
        expected_checksum = %header.checksum.as_deref().unwrap_or("<none>"),
        write_failed = fetch_outcome.write_failed,
        succeeded,
        "sync-files: finalization result (checksum verified on the Kotlin side)",
    );

    if succeeded {
        emit(
            "sync:files:chapter_complete",
            chapter_complete_payload(&peer, &header),
        );
        ChapterOutcome::Received
    } else {
        if handle >= 0 {
            if let Err(err) = abort_chapter_write(&provider, handle).await {
                tracing::warn!(error = %err, "sync-files: abort_chapter_write failed");
            }
        }
        emit(
            "sync:files:chapter_failed",
            chapter_failed_payload(&peer, &header, fetch_outcome.write_failed),
        );
        ChapterOutcome::Failed {
            timed_out: fetch_outcome.timed_out,
        }
    }
}

fn emit_progress(
    emit: &EventEmitter,
    peer: &PeerIdentity,
    comic_name: &str,
    chapter: &str,
    transferred: u64,
    total: u64,
) {
    emit(
        "sync:files:progress",
        serde_json::json!({
            "peerId": peer.id,
            "comicName": comic_name,
            "chapter": chapter,
            "bytesTransferred": transferred,
            "totalBytes": total,
        })
        .to_string(),
    );
}

fn manifest_exchanged_payload(
    peer: &PeerIdentity,
    missing_count: usize,
    offering_count: usize,
) -> String {
    serde_json::json!({ "peerId": peer.id, "missingCount": missing_count, "offeringCount": offering_count }).to_string()
}

/// Phase 1: manifest exchange (rule 4) — outbound writes first, inbound reads first.
async fn exchange_manifests(
    outbound_role: bool,
    writer: &mut Writer,
    reader: &mut Recv,
    local_manifest: &FileManifest,
) -> Result<FileManifest, P2pError> {
    if outbound_role {
        write_json(writer, local_manifest).await?;
        read_manifest_or_busy(reader).await
    } else {
        let peer_manifest = read_manifest_or_busy(reader).await?;
        write_json(writer, local_manifest).await?;
        Ok(peer_manifest)
    }
}

/// Phase 2: want-list exchange, same mirrored order as phase 1. Returns the peer's
/// `(wanted, wanted_extras)`.
async fn exchange_want_lists(
    outbound_role: bool,
    writer: &mut Writer,
    reader: &mut Recv,
    wanted_locally: &[(String, String)],
    wanted_extras_locally: &[(String, String)],
) -> Result<(Vec<(String, String)>, Vec<(String, String)>), P2pError> {
    let mine = FileWantList {
        wanted: wanted_locally.to_vec(),
        wanted_extras: wanted_extras_locally.to_vec(),
    };

    if outbound_role {
        write_json(writer, &mine).await?;
        let theirs: FileWantList = read_json(reader).await?;
        Ok((theirs.wanted, theirs.wanted_extras))
    } else {
        let theirs: FileWantList = read_json(reader).await?;
        write_json(writer, &mine).await?;
        Ok((theirs.wanted, theirs.wanted_extras))
    }
}

/// Phase 3: each side sends what the other requested and receives what it requested, in mirrored
/// order — outbound always writes first at each step, inbound always reads first. Chapters
/// first, then extras, in both directions — same order used by Desktop
/// (`infra/sync/protocol/file_handler.rs`), since both sides read/write the wire in the same
/// sequence without an enum tag telling which message is which.
#[allow(clippy::too_many_arguments)]
async fn transfer_files(
    outbound_role: bool,
    writer: &mut Writer,
    reader: &mut Recv,
    provider: &Arc<dyn FileSyncProvider>,
    local_manifest: &FileManifest,
    wanted_locally: &[(String, String)],
    wanted_extras_locally: &[(String, String)],
    their_wanted: &[(String, String)],
    their_wanted_extras: &[(String, String)],
    emit: &EventEmitter,
    peer: &PeerIdentity,
    stats: &mut FileSyncStats,
    transfer: &Arc<dyn ChapterTransfer>,
) -> Result<(), P2pError> {
    if outbound_role {
        receive_files(
            reader,
            wanted_locally.len(),
            provider,
            emit,
            peer,
            stats,
            transfer,
        )
        .await?;
        receive_extras(
            reader,
            wanted_extras_locally.len(),
            provider,
            emit,
            peer,
            stats,
            transfer,
        )
        .await?;
        send_files(
            writer,
            provider,
            local_manifest,
            their_wanted,
            emit,
            peer,
            stats,
            transfer,
        )
        .await?;
        send_extras(
            writer,
            provider,
            local_manifest,
            their_wanted_extras,
            emit,
            peer,
            stats,
            transfer,
        )
        .await?;
    } else {
        send_files(
            writer,
            provider,
            local_manifest,
            their_wanted,
            emit,
            peer,
            stats,
            transfer,
        )
        .await?;
        send_extras(
            writer,
            provider,
            local_manifest,
            their_wanted_extras,
            emit,
            peer,
            stats,
            transfer,
        )
        .await?;
        receive_files(
            reader,
            wanted_locally.len(),
            provider,
            emit,
            peer,
            stats,
            transfer,
        )
        .await?;
        receive_extras(
            reader,
            wanted_extras_locally.len(),
            provider,
            emit,
            peer,
            stats,
            transfer,
        )
        .await?;
    }

    Ok(())
}

/// Tail shared by phases 1-3 (manifest exchange -> want-list -> transfer), shared
/// between `run_exchange` (whole library) and `run_exchange_scoped` (a single comic,
/// protocol `acerola/sync-comic/1`) — the difference between them is how `local_manifest` was
/// built/filtered before getting here, plus `force_empty_wanted`: `run_exchange` always passes
/// `false` (library sync is always bidirectional); `run_exchange_scoped` passes `true` when
/// this side is the "sender" of the chosen explicit direction (see `SyncDirection` and
/// `run_exchange_scoped`) — in that case the local want-list is emptied even if the diff shows
/// something missing, because this side should only deliver, never request back.
async fn run_exchange_with_manifest(
    outbound_role: bool,
    peer: &PeerIdentity,
    emit: &EventEmitter,
    provider: &Arc<dyn FileSyncProvider>,
    local_manifest: FileManifest,
    writer: &mut Writer,
    reader: &mut Recv,
    transfer: &Arc<dyn ChapterTransfer>,
    force_empty_wanted: bool,
) -> Result<FileSyncStats, P2pError> {
    let mut stats = FileSyncStats::default();

    let peer_manifest = exchange_manifests(outbound_role, writer, reader, &local_manifest).await?;

    let (wanted_locally, wanted_extras_locally, conflicts_locally) = if force_empty_wanted {
        (Vec::new(), Vec::new(), Vec::new())
    } else {
        let (missing, conflicts) = missing_from(&local_manifest, &peer_manifest);
        (missing, missing_extras_from(&local_manifest, &peer_manifest), conflicts)
    };
    stats.conflicts_count = conflicts_locally.len() as u32;
    let (wanted_by_peer, _) = missing_from(&peer_manifest, &local_manifest);
    emit(
        "sync:files:manifest_exchanged",
        manifest_exchanged_payload(peer, wanted_locally.len(), wanted_by_peer.len()),
    );

    let (their_wanted, their_wanted_extras) = exchange_want_lists(
        outbound_role,
        writer,
        reader,
        &wanted_locally,
        &wanted_extras_locally,
    )
    .await?;

    transfer_files(
        outbound_role,
        writer,
        reader,
        provider,
        &local_manifest,
        &wanted_locally,
        &wanted_extras_locally,
        &their_wanted,
        &their_wanted_extras,
        emit,
        peer,
        &mut stats,
        transfer,
    )
    .await?;

    Ok(stats)
}

/// Runs the whole `acerola/sync-files/1` protocol session for one of the two sides.
/// Wire schema and sequence identical to Desktop (`infra/sync/protocol/file_handler.rs`):
/// (1) manifest exchange; (2) want-list exchange; (3) file transfer.
pub(super) async fn run_exchange(
    outbound_role: bool,
    peer: &PeerIdentity,
    emit: &EventEmitter,
    provider: &Arc<dyn FileSyncProvider>,
    transfer: &Arc<dyn ChapterTransfer>,
    send: Box<dyn AsyncWrite + Send + Unpin>,
    recv: Box<dyn AsyncRead + Send + Unpin>,
) -> Result<FileSyncStats, P2pError> {
    let mut writer: Writer = FramedWrite::new(send, LengthDelimitedCodec::new());
    let mut reader: Recv = FramedRead::new(recv, LengthDelimitedCodec::new());

    let local_manifest = build_local_manifest(provider).await?;
    run_exchange_with_manifest(
        outbound_role,
        peer,
        emit,
        provider,
        local_manifest,
        &mut writer,
        &mut reader,
        transfer,
        false,
    )
    .await
}

/// Phase 0, exclusive to `acerola/sync-comic/1`: outbound sends `ComicSyncScope` (the comic and
/// direction chosen by the user, coming from `P2PNode::sync_comic`), inbound only reads — neither
/// side decides the target locally, only the dialer knows. Returns the agreed
/// `(comic_name, direction)`: `comic_name` filters the manifests, `direction` decides which side is
/// the "sender" (see `run_exchange_scoped`).
async fn exchange_comic_scope(
    outbound_role: bool,
    writer: &mut Writer,
    reader: &mut Recv,
    comic_name_outbound: Option<(&str, SyncDirection, &[String])>,
) -> Result<(String, SyncDirection, Vec<String>), P2pError> {
    if outbound_role {
        let (comic_name, direction, chapters) = comic_name_outbound.ok_or_else(|| {
            P2pError::StreamFailed(
                "sync-comic: missing target comic_name for outbound session".into(),
            )
        })?;
        let comic_name = comic_name.to_string();
        let chapters = chapters.to_vec();
        write_json(
            writer,
            &ComicSyncScope {
                comic_name: comic_name.clone(),
                direction,
                chapters: chapters.clone(),
            },
        )
        .await?;
        Ok((comic_name, direction, chapters))
    } else {
        let scope = read_comic_scope_or_busy(reader).await?;
        Ok((scope.comic_name, scope.direction, scope.chapters))
    }
}

/// Keeps only the target comic's entries — the rest of the manifest is never built in vain
/// (filtering happens after `get_file_manifest()`, which already covers the whole library;
/// there is no way to ask `FileSyncProvider` for a single comic without changing the FFI trait).
fn filter_manifest_to_comic(manifest: FileManifest, comic_name: &str) -> FileManifest {
    FileManifest {
        comics: manifest
            .comics
            .into_iter()
            .filter(|c| c.comic_name == comic_name)
            .collect(),
    }
}

/// Restricts a manifest (already filtered to ONE comic, via `filter_manifest_to_comic`) to the
/// requested chapter labels — an empty list means "no filter" (whole comic),
/// keeping the full sync behavior. Extra items (cover/banner/`ComicInfo.xml`)
/// are never filtered here: they are metadata of the comic as a whole, not of a specific
/// chapter. Mirrors Desktop (`core/services/sync/file_sync.rs::restrict_manifest_to_chapters`).
fn restrict_manifest_to_chapters(mut manifest: FileManifest, chapters: &[String]) -> FileManifest {
    if chapters.is_empty() {
        return manifest;
    }
    for comic in &mut manifest.comics {
        comic
            .chapters
            .retain(|chapter| chapters.iter().any(|wanted| wanted == &chapter.chapter));
    }
    manifest
}

/// Runs the whole `acerola/sync-comic/1` protocol session (sync of a single
/// comic) for one of the two sides. Same wire sequence as the regular `sync-files`, but
/// preceded by phase 0 (`ComicSyncScope`) and with `local_manifest` filtered to the agreed
/// comic. The explicit direction (`SyncDirection`, always from the outbound's point of view)
/// decides which side is the "sender" in this session — each side's role is always the opposite:
/// `outbound_role=true` + `Push` or `outbound_role=false` + `Pull` → this side is sender (want
/// list forced empty in `run_exchange_with_manifest`); the other two cases → receiver (requests
/// normally). This covers push and pull with the same diff/transfer pipeline, without changing
/// the manifest exchange itself.
#[allow(clippy::too_many_arguments)]
pub(super) async fn run_exchange_scoped(
    outbound_role: bool,
    peer: &PeerIdentity,
    emit: &EventEmitter,
    provider: &Arc<dyn FileSyncProvider>,
    transfer: &Arc<dyn ChapterTransfer>,
    comic_name_outbound: Option<(String, SyncDirection, Vec<String>)>,
    send: Box<dyn AsyncWrite + Send + Unpin>,
    recv: Box<dyn AsyncRead + Send + Unpin>,
) -> Result<FileSyncStats, P2pError> {
    let mut writer: Writer = FramedWrite::new(send, LengthDelimitedCodec::new());
    let mut reader: Recv = FramedRead::new(recv, LengthDelimitedCodec::new());

    let comic_name_outbound_ref = comic_name_outbound
        .as_ref()
        .map(|(name, direction, chapters)| (name.as_str(), *direction, chapters.as_slice()));
    let (comic_name, direction, chapters) = exchange_comic_scope(
        outbound_role,
        &mut writer,
        &mut reader,
        comic_name_outbound_ref,
    )
    .await?;
    let is_sender = (direction == SyncDirection::Push) == outbound_role;

    let local_manifest = build_local_manifest(provider).await?;
    let local_manifest = filter_manifest_to_comic(local_manifest, &comic_name);
    let local_manifest = restrict_manifest_to_chapters(local_manifest, &chapters);

    run_exchange_with_manifest(
        outbound_role,
        peer,
        emit,
        provider,
        local_manifest,
        &mut writer,
        &mut reader,
        transfer,
        is_sender,
    )
    .await
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{
            atomic::{AtomicI64, AtomicU32, AtomicUsize, Ordering},
            Mutex,
        },
        time::Duration,
    };

    use async_trait::async_trait;

    use super::*;
    use crate::callbacks::FfiFileManifestEntry;
    use crate::protocol::files::model::FileChapterInfo;
    use crate::protocol::files::transfer::InMemoryChapterTransfer;

    /// A pair of in-memory transports sharing the same hash->bytes — simulates both
    /// sides of a real connection publishing/fetching from the same content-addressed "network store",
    /// without needing a real iroh `Endpoint` (only `BlobChapterTransfer`, used in
    /// production, needs that).
    fn test_transfer_pair() -> (Arc<dyn ChapterTransfer>, Arc<dyn ChapterTransfer>) {
        InMemoryChapterTransfer::shared_pair()
    }

    fn sample_header() -> FileHeader {
        FileHeader {
            comic_name: "One Piece".to_string(),
            chapter: "Ch. 3".to_string(),
            file_name: "Ch. 3.cbz".to_string(),
            size: 10,
            checksum: Some("abc".to_string()),
            blob_hash: Some("hash".to_string()),
        }
    }

    fn sample_extra_header() -> FileExtraHeader {
        FileExtraHeader {
            comic_name: "One Piece".to_string(),
            kind: EXTRA_KIND_COVER.to_string(),
            file_name: "cover.jpg".to_string(),
            size: 10,
            checksum: Some("abc".to_string()),
            blob_hash: Some("hash".to_string()),
        }
    }

    fn manifest_with_chapter(comic: &str, chapter: &str, checksum: &str) -> FileManifest {
        FileManifest {
            comics: vec![FileComicInfo {
                comic_name: comic.to_string(),
                chapters: vec![FileChapterInfo {
                    chapter: chapter.to_string(),
                    file_name: format!("{chapter}.cbz"),
                    checksum: Some(checksum.to_string()),
                    size: 10,
                }],
                ..Default::default()
            }],
        }
    }

    /// Same comic/chapter on both sides, different checksum — a real conflict
    /// (reported bug: "conflict is neither detected nor reported"). Still goes to `missing`
    /// (overwrite behavior unchanged), but now also shows up in `conflicts`.
    #[test]
    fn missing_from_reports_a_real_conflict_when_checksums_differ() {
        let local = manifest_with_chapter("Test", "Cap 1", "checksum-local");
        let peer = manifest_with_chapter("Test", "Cap 1", "checksum-peer");

        let (missing, conflicts) = missing_from(&local, &peer);

        assert_eq!(missing, vec![("Test".to_string(), "Cap 1".to_string())]);
        assert_eq!(
            conflicts,
            vec![("Test".to_string(), "Cap 1".to_string())],
            "local checksum different from the peer's is a real conflict, not just an update"
        );
    }

    /// Comic never seen locally: goes to `missing` (that is what makes the download
    /// happen), but it is not a conflict — there was nothing to diverge from.
    #[test]
    fn missing_from_does_not_report_a_conflict_for_a_chapter_never_seen_locally() {
        let local = FileManifest { comics: vec![] };
        let peer = manifest_with_chapter("Novo Quadrinho", "Cap 1", "checksum-peer");

        let (missing, conflicts) = missing_from(&local, &peer);

        assert_eq!(missing, vec![("Novo Quadrinho".to_string(), "Cap 1".to_string())]);
        assert!(conflicts.is_empty(), "a comic never seen locally is not a conflict");
    }

    /// Same checksum on both sides: neither `missing` nor `conflicts` — already in sync.
    #[test]
    fn missing_from_reports_nothing_when_checksums_match() {
        let local = manifest_with_chapter("Test", "Cap 1", "same");
        let peer = manifest_with_chapter("Test", "Cap 1", "same");

        let (missing, conflicts) = missing_from(&local, &peer);

        assert!(missing.is_empty());
        assert!(conflicts.is_empty());
    }

    /// `write_failed: false` is the only case where `chapter_failed_payload` can claim the
    /// cause was specifically a checksum mismatch (see the function docs) — locks that contract
    /// at the lowest possible level, without building a full round-trip just to prove
    /// an `if` check.
    #[test]
    fn chapter_failed_payload_reports_checksum_mismatch_code_when_write_did_not_fail() {
        let peer = PeerIdentity {
            id: "peer-x".to_string(),
            device_id: None,
        };
        let payload = chapter_failed_payload(&peer, &sample_header(), false);
        let parsed: serde_json::Value = serde_json::from_str(&payload).unwrap();

        assert_eq!(parsed["code"], "checksum_mismatch");
        assert!(parsed["reason"]
            .as_str()
            .unwrap()
            .to_lowercase()
            .contains("checksum"));
    }

    /// Counter-proof: `write_failed: true` (I/O failure writing chunks, or the blob fetch
    /// failed) must not be mistaken for a checksum mismatch — without a recognized `code`, it falls
    /// back to the generic Kotlin-side fallback (`SyncProtocolError.fromCode(null)`).
    #[test]
    fn chapter_failed_payload_reports_no_code_for_a_generic_io_failure() {
        let peer = PeerIdentity {
            id: "peer-x".to_string(),
            device_id: None,
        };
        let payload = chapter_failed_payload(&peer, &sample_header(), true);
        let parsed: serde_json::Value = serde_json::from_str(&payload).unwrap();

        assert!(parsed["code"].is_null());
        assert!(parsed["reason"]
            .as_str()
            .unwrap()
            .to_lowercase()
            .contains("i/o"));
    }

    /// Same contract as `chapter_failed_payload`, applied to extra items.
    #[test]
    fn extra_failed_payload_reports_checksum_mismatch_code_when_write_did_not_fail() {
        let peer = PeerIdentity {
            id: "peer-x".to_string(),
            device_id: None,
        };
        let payload = extra_failed_payload(&peer, &sample_extra_header(), false);
        let parsed: serde_json::Value = serde_json::from_str(&payload).unwrap();

        assert_eq!(parsed["code"], "checksum_mismatch");
        assert!(parsed["reason"]
            .as_str()
            .unwrap()
            .to_lowercase()
            .contains("checksum"));
    }

    #[test]
    fn extra_failed_payload_reports_no_code_for_a_generic_io_failure() {
        let peer = PeerIdentity {
            id: "peer-x".to_string(),
            device_id: None,
        };
        let payload = extra_failed_payload(&peer, &sample_extra_header(), true);
        let parsed: serde_json::Value = serde_json::from_str(&payload).unwrap();

        assert!(parsed["code"].is_null());
        assert!(parsed["reason"]
            .as_str()
            .unwrap()
            .to_lowercase()
            .contains("i/o"));
    }

    /// Provider never used to transfer anything — all data methods return
    /// "empty"/"-1"/"false". Only `get_file_manifest` does something (defined by a closure in each
    /// specific mock via composition).
    struct CountingSlowProvider {
        manifest_calls: Arc<AtomicUsize>,
        sleep_ms: u64,
    }

    impl FileSyncProvider for CountingSlowProvider {
        fn get_file_manifest(&self) -> Vec<FfiFileManifestEntry> {
            self.manifest_calls.fetch_add(1, Ordering::SeqCst);
            if self.sleep_ms > 0 {
                std::thread::sleep(Duration::from_millis(self.sleep_ms));
            }
            vec![]
        }
        fn get_library_summary(&self) -> Vec<crate::callbacks::FfiComicSummaryEntry> {
            vec![]
        }
        fn open_chapter_for_read(&self, _comic_name: String, _chapter: String) -> i64 {
            -1
        }
        fn read_chapter_chunk(&self, _handle: i64, _chunk_size: u32) -> Vec<u8> {
            vec![]
        }
        fn close_read_handle(&self, _handle: i64) {}
        fn begin_chapter_write(
            &self,
            _comic_name: String,
            _chapter: String,
            _file_name: String,
            _expected_checksum: String,
            _size_bytes: u64,
        ) -> i64 {
            -1
        }
        fn write_chapter_chunk(&self, _handle: i64, _bytes: Vec<u8>) -> bool {
            false
        }
        fn finalize_chapter_write(&self, _handle: i64) -> bool {
            false
        }
        fn abort_chapter_write(&self, _handle: i64) {}
        fn get_extras_manifest(&self) -> Vec<crate::callbacks::FfiExtraManifestEntry> {
            vec![]
        }
        fn open_extra_for_read(&self, _comic_name: String, _kind: String) -> i64 {
            -1
        }
        fn begin_extra_write(
            &self,
            _comic_name: String,
            _kind: String,
            _file_name: String,
            _expected_checksum: String,
            _size_bytes: u64,
        ) -> i64 {
            -1
        }
        fn finalize_extra_write(&self, _handle: i64) -> bool {
            false
        }
        fn abort_extra_write(&self, _handle: i64) {}
    }

    fn make_peer(id: &str) -> PeerIdentity {
        PeerIdentity {
            id: id.to_string(),
            device_id: None,
        }
    }

    fn no_op_emitter() -> EventEmitter {
        Arc::new(|_event: &str, _payload: String| {})
    }

    /// Proves, at the `build_local_manifest` level (already using `run_blocking`), that a slow
    /// FFI call does not block other concurrent async tasks running on the same runtime — the
    /// exact signature of the reported bug (sync-files froze the whole app for minutes).
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn build_local_manifest_does_not_block_runtime() {
        let provider: Arc<dyn FileSyncProvider> = Arc::new(CountingSlowProvider {
            manifest_calls: Arc::new(AtomicUsize::new(0)),
            sleep_ms: 200,
        });

        let ticks = Arc::new(AtomicU32::new(0));
        let ticks_clone = Arc::clone(&ticks);
        let ticker = tokio::spawn(async move {
            for _ in 0..15 {
                tokio::time::sleep(Duration::from_millis(10)).await;
                ticks_clone.fetch_add(1, Ordering::SeqCst);
            }
        });

        let manifest = build_local_manifest(&provider).await.unwrap();
        let _ = tokio::join!(ticker);

        assert!(manifest.comics.is_empty());
        assert!(
            ticks.load(Ordering::SeqCst) >= 8,
            "runtime was blocked during get_file_manifest"
        );
    }

    /// Proves `run_exchange` builds the local manifest exactly once per session — before
    /// the fix, `send_files` rebuilt it from scratch internally, doubling the work.
    #[tokio::test]
    async fn run_exchange_builds_local_manifest_exactly_once_per_side() {
        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let outbound_calls = Arc::new(AtomicUsize::new(0));
        let inbound_calls = Arc::new(AtomicUsize::new(0));
        let outbound_provider: Arc<dyn FileSyncProvider> = Arc::new(CountingSlowProvider {
            manifest_calls: Arc::clone(&outbound_calls),
            sleep_ms: 0,
        });
        let inbound_provider: Arc<dyn FileSyncProvider> = Arc::new(CountingSlowProvider {
            manifest_calls: Arc::clone(&inbound_calls),
            sleep_ms: 0,
        });

        let peer = make_peer("peer-x");
        let emit = no_op_emitter();
        let (outbound_transfer, inbound_transfer) = test_transfer_pair();

        let outbound_fut = run_exchange(
            true,
            &peer,
            &emit,
            &outbound_provider,
            &outbound_transfer,
            Box::new(client_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(client_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );
        let inbound_fut = run_exchange(
            false,
            &peer,
            &emit,
            &inbound_provider,
            &inbound_transfer,
            Box::new(server_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(server_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );

        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        outbound_result.unwrap();
        inbound_result.unwrap();

        assert_eq!(outbound_calls.load(Ordering::SeqCst), 1);
        assert_eq!(inbound_calls.load(Ordering::SeqCst), 1);
    }

    // --- Transfer integrity tests ---
    //
    // The mocks above (`CountingSlowProvider`) never touch real bytes — `read_chapter_chunk`
    // always returns empty, `write_chapter_chunk` always fails. That leaves a gap: no
    // existing test exercises the real send -> framing -> receive -> checksum verification
    // path, which is exactly where we suspect a "checksum mismatch" bug in production
    // (already proven, with a real hash of a real file, that the SOURCE checksum is correct — the
    // problem is somewhere between the source and the final verification).
    //
    // `InMemoryFileSyncProvider` keeps real bytes in both directions and uses blake3 (only
    // as a test checksum — it is not the real production SHA-256, which is computed in Kotlin)
    // to prove that what leaves one side arrives intact on the other, including the checksum
    // value itself crossing the wire inside `FileHeader`. Same spirit as the stress tests of
    // `acerola-p2p` (`transport_stress.rs`: real payload + real hash + comparison), but
    // exercising the whole `sync-files` protocol (manifest -> want-list -> transfer)
    // instead of the raw transport.

    /// `read_chapter_chunk`/`write_chapter_chunk`/`close_read_handle` are reused unchanged
    /// for both chapter and extra (see the trait docs in `callbacks.rs`) — so
    /// both handle kinds must live in the SAME handle/map space here, exactly
    /// like the real Kotlin implementation (`FileSyncProviderImpl`) will need to. The only thing
    /// that differs between them is what `WriteTarget` holds and which list (`finalized` vs.
    /// `finalized_extras`) the result goes to.
    enum WriteTarget {
        Chapter { comic_name: String, chapter: String },
        Extra { comic_name: String, kind: String },
    }

    struct WriteState {
        target: WriteTarget,
        expected_checksum: String,
        buffer: Vec<u8>,
    }

    /// A chapter already finalized on the receiving side — keeps both the checksum
    /// declared by the sender (`expected_checksum`, what came in the `FileHeader`) and the
    /// one this provider recomputed from the bytes actually received (`actual_checksum`),
    /// plus the raw bytes — so both the hash and the byte-by-byte content can be compared.
    struct FinalizedChapter {
        comic_name: String,
        chapter: String,
        expected_checksum: String,
        actual_checksum: String,
        bytes: Vec<u8>,
    }

    /// Same pair (declared x recomputed checksum) as `FinalizedChapter`, for an extra item
    /// (cover/banner/`ComicInfo.xml`) instead of a chapter.
    struct FinalizedExtra {
        comic_name: String,
        kind: String,
        expected_checksum: String,
        actual_checksum: String,
        bytes: Vec<u8>,
    }

    struct InMemoryFileSyncProvider {
        readable: HashMap<(String, String), (String, Vec<u8>)>,
        read_handles: Mutex<HashMap<i64, (Vec<u8>, usize)>>,
        write_handles: Mutex<HashMap<i64, WriteState>>,
        finalized: Mutex<Vec<FinalizedChapter>>,
        readable_extras: HashMap<(String, String), (String, Vec<u8>)>,
        finalized_extras: Mutex<Vec<FinalizedExtra>>,
        next_handle: AtomicI64,
    }

    impl InMemoryFileSyncProvider {
        fn with_readable(entries: Vec<(String, String, String, Vec<u8>)>) -> Self {
            Self::with_readable_and_extras(entries, vec![])
        }

        /// `extras`: `(comic_name, kind, file_name, bytes)`.
        fn with_readable_and_extras(
            entries: Vec<(String, String, String, Vec<u8>)>,
            extras: Vec<(String, String, String, Vec<u8>)>,
        ) -> Self {
            let readable = entries
                .into_iter()
                .map(|(comic_name, chapter, file_name, bytes)| {
                    ((comic_name, chapter), (file_name, bytes))
                })
                .collect();
            let readable_extras = extras
                .into_iter()
                .map(|(comic_name, kind, file_name, bytes)| {
                    ((comic_name, kind), (file_name, bytes))
                })
                .collect();
            Self {
                readable,
                read_handles: Mutex::new(HashMap::new()),
                write_handles: Mutex::new(HashMap::new()),
                finalized: Mutex::new(Vec::new()),
                readable_extras,
                finalized_extras: Mutex::new(Vec::new()),
                next_handle: AtomicI64::new(1),
            }
        }
    }

    impl FileSyncProvider for InMemoryFileSyncProvider {
        fn get_file_manifest(&self) -> Vec<FfiFileManifestEntry> {
            self.readable
                .iter()
                .map(
                    |((comic_name, chapter), (file_name, bytes))| FfiFileManifestEntry {
                        comic_name: comic_name.clone(),
                        chapter: chapter.clone(),
                        file_name: file_name.clone(),
                        checksum: blake3::hash(bytes).to_hex().to_string(),
                        size_bytes: bytes.len() as u64,
                    },
                )
                .collect()
        }

        fn get_library_summary(&self) -> Vec<crate::callbacks::FfiComicSummaryEntry> {
            vec![]
        }

        fn open_chapter_for_read(&self, comic_name: String, chapter: String) -> i64 {
            let Some((_, bytes)) = self.readable.get(&(comic_name, chapter)) else {
                return -1;
            };
            let handle = self.next_handle.fetch_add(1, Ordering::SeqCst);
            self.read_handles
                .lock()
                .unwrap()
                .insert(handle, (bytes.clone(), 0));
            handle
        }

        fn read_chapter_chunk(&self, handle: i64, chunk_size: u32) -> Vec<u8> {
            let mut handles = self.read_handles.lock().unwrap();
            let Some((bytes, pos)) = handles.get_mut(&handle) else {
                return vec![];
            };
            let end = (*pos + chunk_size as usize).min(bytes.len());
            if *pos >= end {
                return vec![];
            }
            let chunk = bytes[*pos..end].to_vec();
            *pos = end;
            chunk
        }

        fn close_read_handle(&self, handle: i64) {
            self.read_handles.lock().unwrap().remove(&handle);
        }

        fn begin_chapter_write(
            &self,
            comic_name: String,
            chapter: String,
            _file_name: String,
            expected_checksum: String,
            _size_bytes: u64,
        ) -> i64 {
            let handle = self.next_handle.fetch_add(1, Ordering::SeqCst);
            self.write_handles.lock().unwrap().insert(
                handle,
                WriteState {
                    target: WriteTarget::Chapter {
                        comic_name,
                        chapter,
                    },
                    expected_checksum,
                    buffer: Vec::new(),
                },
            );
            handle
        }

        fn write_chapter_chunk(&self, handle: i64, bytes: Vec<u8>) -> bool {
            let mut handles = self.write_handles.lock().unwrap();
            let Some(state) = handles.get_mut(&handle) else {
                return false;
            };
            state.buffer.extend_from_slice(&bytes);
            true
        }

        fn finalize_chapter_write(&self, handle: i64) -> bool {
            let Some(state) = self.write_handles.lock().unwrap().remove(&handle) else {
                return false;
            };
            let WriteTarget::Chapter {
                comic_name,
                chapter,
            } = state.target
            else {
                return false;
            };
            let actual_checksum = blake3::hash(&state.buffer).to_hex().to_string();
            let succeeded = actual_checksum == state.expected_checksum;
            self.finalized.lock().unwrap().push(FinalizedChapter {
                comic_name,
                chapter,
                expected_checksum: state.expected_checksum,
                actual_checksum,
                bytes: state.buffer,
            });
            succeeded
        }

        fn abort_chapter_write(&self, handle: i64) {
            self.write_handles.lock().unwrap().remove(&handle);
        }

        fn get_extras_manifest(&self) -> Vec<crate::callbacks::FfiExtraManifestEntry> {
            self.readable_extras
                .iter()
                .map(|((comic_name, kind), (file_name, bytes))| {
                    crate::callbacks::FfiExtraManifestEntry {
                        comic_name: comic_name.clone(),
                        kind: kind.clone(),
                        file_name: file_name.clone(),
                        checksum: blake3::hash(bytes).to_hex().to_string(),
                        size_bytes: bytes.len() as u64,
                    }
                })
                .collect()
        }

        // Reuses `read_handles` (the SAME map used by `open_chapter_for_read`) — there is
        // no separate `open_extra_for_read` in the map, only in the trait: the handle returned here
        // is later consumed by the usual `read_chapter_chunk`/`close_read_handle`.
        fn open_extra_for_read(&self, comic_name: String, kind: String) -> i64 {
            let Some((_, bytes)) = self.readable_extras.get(&(comic_name, kind)) else {
                return -1;
            };
            let handle = self.next_handle.fetch_add(1, Ordering::SeqCst);
            self.read_handles
                .lock()
                .unwrap()
                .insert(handle, (bytes.clone(), 0));
            handle
        }

        // Same idea as `open_extra_for_read`: inserts into `write_handles`, the map shared
        // with chapters — `write_chapter_chunk` (reused) writes here without knowing the difference.
        fn begin_extra_write(
            &self,
            comic_name: String,
            kind: String,
            _file_name: String,
            expected_checksum: String,
            _size_bytes: u64,
        ) -> i64 {
            let handle = self.next_handle.fetch_add(1, Ordering::SeqCst);
            self.write_handles.lock().unwrap().insert(
                handle,
                WriteState {
                    target: WriteTarget::Extra { comic_name, kind },
                    expected_checksum,
                    buffer: Vec::new(),
                },
            );
            handle
        }

        fn finalize_extra_write(&self, handle: i64) -> bool {
            let Some(state) = self.write_handles.lock().unwrap().remove(&handle) else {
                return false;
            };
            let WriteTarget::Extra { comic_name, kind } = state.target else {
                return false;
            };
            let actual_checksum = blake3::hash(&state.buffer).to_hex().to_string();
            let succeeded = actual_checksum == state.expected_checksum;
            self.finalized_extras.lock().unwrap().push(FinalizedExtra {
                comic_name,
                kind,
                expected_checksum: state.expected_checksum,
                actual_checksum,
                bytes: state.buffer,
            });
            succeeded
        }

        fn abort_extra_write(&self, handle: i64) {
            self.write_handles.lock().unwrap().remove(&handle);
        }
    }

    /// Deterministic payload generator — varies with `seed` so different chapters do not
    /// have the same content (avoids masking a swap/mix-up bug between chapters).
    fn make_payload(size: usize, seed: u8) -> Vec<u8> {
        (0..size)
            .map(|i| ((i as u64).wrapping_mul(31).wrapping_add(seed as u64) % 251) as u8)
            .collect()
    }

    /// Round-trip with real bytes in both directions, sizes deliberately NOT
    /// multiples of `CHUNK_SIZE` (64KB) to exercise the loop's last partial chunk in
    /// `stream_chapter_out`/`receive_chapter_bytes`. Checks three independent things:
    /// (1) the checksum declared by the sender survives the wire; (2) the checksum recomputed
    /// on the receiving side matches the declared one; (3) the received bytes are identical,
    /// byte by byte, to what was sent — not just the hash matching (a matching hash does not prove
    /// absence of a bug if the mock were also wrong the same way on both sides).
    #[tokio::test]
    async fn run_exchange_transfers_real_bytes_with_correct_checksum_round_trip() {
        let outbound_payload = make_payload(1_500_037, 7);
        let inbound_payload = make_payload(900_321, 42);

        let outbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(vec![(
            "Comic A".to_string(),
            "Ch. 1".to_string(),
            "ch1.cbz".to_string(),
            outbound_payload.clone(),
        )]));
        let inbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(vec![(
            "Comic B".to_string(),
            "Ch. 1".to_string(),
            "ch1.cbz".to_string(),
            inbound_payload.clone(),
        )]));

        let (client_io, server_io) = tokio::io::duplex(256 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let peer = make_peer("peer-x");
        let emit = no_op_emitter();
        let (outbound_transfer, inbound_transfer) = test_transfer_pair();

        let outbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&outbound_provider) as Arc<dyn FileSyncProvider>;
        let inbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&inbound_provider) as Arc<dyn FileSyncProvider>;

        let outbound_fut = run_exchange(
            true,
            &peer,
            &emit,
            &outbound_dyn,
            &outbound_transfer,
            Box::new(client_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(client_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );
        let inbound_fut = run_exchange(
            false,
            &peer,
            &emit,
            &inbound_dyn,
            &inbound_transfer,
            Box::new(server_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(server_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );

        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        outbound_result.expect("outbound should complete without error");
        inbound_result.expect("inbound should complete without error");

        let outbound_received = outbound_provider.finalized.lock().unwrap();
        assert_eq!(
            outbound_received.len(),
            1,
            "outbound should have received exactly 1 chapter from B"
        );
        let received_from_b = &outbound_received[0];
        assert_eq!(received_from_b.comic_name, "Comic B");
        assert_eq!(
            received_from_b.expected_checksum, received_from_b.actual_checksum,
            "checksum declared by B does not match the one recomputed by A after receiving"
        );
        assert_eq!(
            received_from_b.bytes, inbound_payload,
            "bytes received by A do not match byte by byte what B sent"
        );

        let inbound_received = inbound_provider.finalized.lock().unwrap();
        assert_eq!(
            inbound_received.len(),
            1,
            "inbound should have received exactly 1 chapter from A"
        );
        let received_from_a = &inbound_received[0];
        assert_eq!(received_from_a.comic_name, "Comic A");
        assert_eq!(
            received_from_a.expected_checksum, received_from_a.actual_checksum,
            "checksum declared by A does not match the one recomputed by B after receiving"
        );
        assert_eq!(
            received_from_a.bytes, outbound_payload,
            "bytes received by B do not match byte by byte what A sent"
        );
    }

    /// Same comic/chapter on both sides, different content (checksum) — real scenario of the
    /// reported bug ("conflict is neither detected nor reported" when syncing a comic already
    /// present on both devices). Proves `FileSyncStats::conflicts_count` reflects that on
    /// both sides, with the session completing normally (overwrite behavior
    /// unchanged — see `missing_from`).
    #[tokio::test]
    async fn run_exchange_reports_a_real_conflict_when_both_sides_have_the_same_chapter_with_different_content(
    ) {
        let outbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(vec![(
            "Test".to_string(),
            "Cap 1".to_string(),
            "Cap 1.cbz".to_string(),
            b"versao do outbound".to_vec(),
        )]));
        let inbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(vec![(
            "Test".to_string(),
            "Cap 1".to_string(),
            "Cap 1.cbz".to_string(),
            b"versao do inbound, diferente".to_vec(),
        )]));

        let (client_io, server_io) = tokio::io::duplex(256 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let peer = make_peer("peer-conflict");
        let emit = no_op_emitter();
        let (outbound_transfer, inbound_transfer) = test_transfer_pair();

        let outbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&outbound_provider) as Arc<dyn FileSyncProvider>;
        let inbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&inbound_provider) as Arc<dyn FileSyncProvider>;

        let outbound_fut = run_exchange(
            true,
            &peer,
            &emit,
            &outbound_dyn,
            &outbound_transfer,
            Box::new(client_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(client_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );
        let inbound_fut = run_exchange(
            false,
            &peer,
            &emit,
            &inbound_dyn,
            &inbound_transfer,
            Box::new(server_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(server_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );

        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        let outbound_stats =
            outbound_result.expect("outbound should not fail even with a conflict");
        let inbound_stats =
            inbound_result.expect("inbound should not fail even with a conflict");

        assert_eq!(
            outbound_stats.conflicts_count, 1,
            "outbound already had 'Cap 1' with a checksum different from the peer's"
        );
        assert_eq!(
            inbound_stats.conflicts_count, 1,
            "inbound already had 'Cap 1' with a checksum different from the peer's"
        );
    }

    /// Test double that measures how many `fetch_reader` calls are in flight at once (observed
    /// peak) — the only way to prove `receive_files` really runs fetches in parallel,
    /// no longer one chapter at a time as before this change. Mirrors Desktop's
    /// `ConcurrencyTrackingTransfer` (`infra/sync/protocol/comic_handler.rs`).
    struct ConcurrencyTrackingTransfer {
        latency: Duration,
        fetch_delay: Duration,
        in_flight: AtomicUsize,
        max_observed: AtomicUsize,
    }

    impl ConcurrencyTrackingTransfer {
        fn new(latency: Duration, fetch_delay: Duration) -> Self {
            Self {
                latency,
                fetch_delay,
                in_flight: AtomicUsize::new(0),
                max_observed: AtomicUsize::new(0),
            }
        }

        fn max_observed(&self) -> usize {
            self.max_observed.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl ChapterTransfer for ConcurrencyTrackingTransfer {
        async fn publish(&self, _bytes: Vec<u8>) -> Result<String, P2pError> {
            unimplemented!("this test only exercises the receiving side (fetch_reader)")
        }

        async fn fetch_reader(
            &self,
            _blob_hash: &str,
            _peer: &PeerIdentity,
        ) -> Result<Box<dyn AsyncRead + Send + Unpin>, P2pError> {
            let current = self.in_flight.fetch_add(1, Ordering::SeqCst) + 1;
            self.max_observed.fetch_max(current, Ordering::SeqCst);
            tokio::time::sleep(self.fetch_delay).await;
            self.in_flight.fetch_sub(1, Ordering::SeqCst);
            Ok(Box::new(tokio::io::empty()))
        }

        async fn latency(&self, _peer: &PeerIdentity) -> Option<Duration> {
            Some(self.latency)
        }
    }

    /// Core regression of this change: before, `receive_files` read a header, blocked on the
    /// WHOLE fetch of that chapter, and only then read the next one — concurrency peak always 1,
    /// regardless of latency. With a simulated RTT of 500ms (> 200ms, ceiling of 6 concurrent
    /// via `chapter_fetch_concurrency`) and 6 expected chapters, if the fetch really ran
    /// in parallel the observed peak must exceed 1 — proving that reading the NEXT
    /// header is no longer blocked behind the previous chapter's fetch.
    #[tokio::test]
    async fn receive_files_fetches_chapters_concurrently_on_a_high_latency_connection() {
        let provider: Arc<dyn FileSyncProvider> =
            Arc::new(InMemoryFileSyncProvider::with_readable(vec![]));
        let emit = no_op_emitter();
        let peer = make_peer("peer-concurrency");

        // Kept as the concrete type (not `Arc<dyn ChapterTransfer>`) just to be able to read
        // `max_observed()` later — `receive_files` gets a reference via implicit coercion
        // at the call site itself, no downcast needed.
        let transfer = Arc::new(ConcurrencyTrackingTransfer::new(
            Duration::from_millis(500),
            Duration::from_millis(60),
        ));

        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let (_client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, _server_send) = tokio::io::split(server_io);
        let mut writer: Writer =
            FramedWrite::new(Box::new(client_send), LengthDelimitedCodec::new());
        let mut reader: Recv = FramedRead::new(Box::new(server_recv), LengthDelimitedCodec::new());

        // `fetch_reader` always returns an empty reader — the expected checksum must match
        // the hash of an empty buffer (`InMemoryFileSyncProvider::finalize_chapter_write` really
        // recomputes it), otherwise the 6 "concurrent" chapters become checksum failures instead of
        // successes — the test would still measure concurrency correctly, but `received_count`
        // would stay at 0, masking a real integrity regression behind an unrelated
        // metric (parallelism != correctness).
        let empty_checksum = blake3::hash(&[]).to_hex().to_string();

        const CHAPTER_COUNT: usize = 6;
        let writer_task = tokio::spawn(async move {
            for i in 0..CHAPTER_COUNT {
                write_json(
                    &mut writer,
                    &FileHeader {
                        comic_name: "Test".to_string(),
                        chapter: format!("Cap {i}"),
                        file_name: format!("Cap {i}.cbz"),
                        size: 10,
                        checksum: Some(empty_checksum.clone()),
                        blob_hash: Some(format!("hash-{i}")),
                    },
                )
                .await
                .unwrap();
            }
            // Dropping `writer` (end of this task's scope) closes the duplex's write half —
            // without it the receiving side's `FramedRead` would never see EOF after the last header.
        });

        let mut stats = FileSyncStats::default();
        receive_files(
            &mut reader,
            CHAPTER_COUNT,
            &provider,
            &emit,
            &peer,
            &mut stats,
            &(Arc::clone(&transfer) as Arc<dyn ChapterTransfer>),
        )
        .await
        .unwrap();
        writer_task.await.unwrap();

        assert_eq!(
            stats.received_count, CHAPTER_COUNT as u32,
            "all 6 chapters should have been persisted successfully"
        );
        assert_eq!(stats.failed_count, 0);
        let observed = transfer.max_observed();
        assert!(
            observed >= 2,
            "expected overlapping fetches (peak >= 2) with high RTT, only observed a peak of {observed} \
             — receive_files went back to processing one chapter at a time"
        );
    }

    /// Test double that injects specific failures by `blob_hash` — allows mixing success,
    /// generic fetch failure and task panic in the SAME concurrent batch. Only exists to back
    /// the test below (`receive_files_isolates_failures_within_a_concurrent_batch...`); unlike
    /// `ConcurrencyTrackingTransfer`, it does not measure the concurrency peak, it only decides the
    /// fetch result from the hash prefix.
    struct FaultyTransfer {
        blobs: HashMap<String, Vec<u8>>,
    }

    impl FaultyTransfer {
        fn new(blobs: HashMap<String, Vec<u8>>) -> Self {
            Self { blobs }
        }
    }

    #[async_trait]
    impl ChapterTransfer for FaultyTransfer {
        async fn publish(&self, _bytes: Vec<u8>) -> Result<String, P2pError> {
            unimplemented!("this test only exercises the receiving side (fetch_reader)")
        }

        async fn fetch_reader(
            &self,
            blob_hash: &str,
            _peer: &PeerIdentity,
        ) -> Result<Box<dyn AsyncRead + Send + Unpin>, P2pError> {
            if blob_hash.starts_with("fail-") {
                return Err(P2pError::StreamFailed("simulated fetch failure".into()));
            }
            if blob_hash.starts_with("panic-") {
                panic!("simulated task panic for blob {blob_hash}");
            }

            let bytes = self.blobs.get(blob_hash).cloned().unwrap_or_default();
            use tokio::io::AsyncWriteExt;
            let (mut writer, reader) = tokio::io::duplex(bytes.len().max(1));
            tokio::spawn(async move {
                let _ = writer.write_all(&bytes).await;
                let _ = writer.shutdown().await;
            });
            Ok(Box::new(reader))
        }

        async fn latency(&self, _peer: &PeerIdentity) -> Option<Duration> {
            // High enough for `chapter_fetch_concurrency` to allow the ceiling of 6 — all the
            // headers in this test (5) fit in a single concurrent batch.
            Some(Duration::from_millis(500))
        }
    }

    /// Regression of this change's core behavior: BEFORE, any failure in
    /// `begin_chapter_write`/`receive_chapter_from_blob`/`finalize_chapter_write` propagated via
    /// `?` and aborted the WHOLE session — one bad chapter in the middle of the batch took down all
    /// the others, including those that had already arrived. Now each chapter is isolated
    /// (`ChapterOutcome::Failed`), so a batch with success + generic fetch failure +
    /// checksum mismatch + missing `blob_hash` + a task that literally PANICKED must
    /// finish the whole session with `Ok(())` and correct counts, with no side effect
    /// on the sibling chapters that were in flight at the same time.
    #[tokio::test]
    async fn receive_files_isolates_failures_within_a_concurrent_batch_instead_of_aborting_the_session(
    ) {
        let ok_payload_1 = make_payload(64, 1);
        let ok_payload_2 = make_payload(128, 2);
        let wrong_payload = make_payload(32, 3);

        let mut blobs = HashMap::new();
        blobs.insert("ok-1".to_string(), ok_payload_1.clone());
        blobs.insert("ok-2".to_string(), ok_payload_2.clone());
        blobs.insert("mismatch-1".to_string(), wrong_payload);

        let transfer: Arc<dyn ChapterTransfer> = Arc::new(FaultyTransfer::new(blobs));
        let provider: Arc<dyn FileSyncProvider> =
            Arc::new(InMemoryFileSyncProvider::with_readable(vec![]));

        let emit_log: Arc<Mutex<Vec<(String, String)>>> = Arc::new(Mutex::new(Vec::new()));
        let emit: EventEmitter = {
            let emit_log = Arc::clone(&emit_log);
            Arc::new(move |event: &str, payload: String| {
                emit_log.lock().unwrap().push((event.to_string(), payload));
            })
        };
        let peer = make_peer("peer-faulty");

        let headers = vec![
            FileHeader {
                comic_name: "Test".to_string(),
                chapter: "ok-1".to_string(),
                file_name: "ok-1.cbz".to_string(),
                size: 10,
                checksum: Some(blake3::hash(&ok_payload_1).to_hex().to_string()),
                blob_hash: Some("ok-1".to_string()),
            },
            FileHeader {
                comic_name: "Test".to_string(),
                chapter: "fetch-fails".to_string(),
                file_name: "fetch-fails.cbz".to_string(),
                size: 10,
                checksum: None,
                blob_hash: Some("fail-network".to_string()),
            },
            FileHeader {
                comic_name: "Test".to_string(),
                chapter: "checksum-mismatch".to_string(),
                file_name: "checksum-mismatch.cbz".to_string(),
                size: 10,
                // Deliberately NOT the checksum of `wrong_payload` — forces
                // `finalize_chapter_write` to report a mismatch instead of success.
                checksum: Some(blake3::hash(b"not the real content").to_hex().to_string()),
                blob_hash: Some("mismatch-1".to_string()),
            },
            FileHeader {
                comic_name: "Test".to_string(),
                chapter: "ok-2".to_string(),
                file_name: "ok-2.cbz".to_string(),
                size: 10,
                checksum: Some(blake3::hash(&ok_payload_2).to_hex().to_string()),
                blob_hash: Some("ok-2".to_string()),
            },
            FileHeader {
                comic_name: "Test".to_string(),
                chapter: "task-panics".to_string(),
                file_name: "task-panics.cbz".to_string(),
                size: 10,
                checksum: None,
                blob_hash: Some("panic-boom".to_string()),
            },
            FileHeader {
                comic_name: "Test".to_string(),
                chapter: "missing-blob-hash".to_string(),
                file_name: "missing-blob-hash.cbz".to_string(),
                size: 10,
                checksum: None,
                blob_hash: None,
            },
        ];
        let header_count = headers.len();

        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let (_client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, _server_send) = tokio::io::split(server_io);
        let mut writer: Writer =
            FramedWrite::new(Box::new(client_send), LengthDelimitedCodec::new());
        let mut reader: Recv = FramedRead::new(Box::new(server_recv), LengthDelimitedCodec::new());

        let writer_task = tokio::spawn(async move {
            for header in headers {
                write_json(&mut writer, &header).await.unwrap();
            }
        });

        let mut stats = FileSyncStats::default();
        let result = receive_files(
            &mut reader,
            header_count,
            &provider,
            &emit,
            &peer,
            &mut stats,
            &transfer,
        )
        .await;
        writer_task.await.unwrap();

        result.expect("an isolated chapter failure must not abort the whole session");
        assert_eq!(
            stats.received_count, 2,
            "the 2 valid chapters should have been persisted even with failures alongside"
        );
        assert_eq!(
            stats.failed_count, 4,
            "generic fetch + checksum mismatch + task panic + missing blob_hash = 4 isolated failures"
        );

        let events = emit_log.lock().unwrap();
        let failed_events = events
            .iter()
            .filter(|(name, _)| name == "sync:files:chapter_failed")
            .count();
        let complete_events = events
            .iter()
            .filter(|(name, _)| name == "sync:files:chapter_complete")
            .count();
        // Only 3, not 4: the task that PANICKED never runs the body of `receive_one_chapter`
        // to the end (it is the `JoinSet` that catches the panic as `Err(join_err)` in `receive_files`),
        // so there is nowhere to emit `chapter_failed` from — only `tracing::error!` + a count in
        // `stats.failed_count`. Same behavior as Desktop (`transfer.rs::receive_files`)
        // for the `Err(join_err)` branch: a panic is a real bug, not an item-level network failure.
        assert_eq!(
            failed_events, 3,
            "generic fetch + checksum mismatch + missing blob_hash emit chapter_failed; \
             the task panic does not (only counted in stats.failed_count)"
        );
        assert_eq!(complete_events, 2);
    }

    /// The parallelism cut (`permits_to_forget`) may only happen ONCE per session, even
    /// if several concurrent tasks observe `timed_out: true` at the same time — without the
    /// `compare_exchange`, a batch with several simultaneous timeouts would shrink the pool several times
    /// (`forget_permits` is cumulative, never returns permits), reducing concurrency far
    /// beyond what is intended. Tested in isolation from the real `receive_files` logic (which depends on
    /// the real scheduler for task execution order, not deterministic enough for
    /// a reliable test) — only the guard mechanics themselves, copied exactly from the call site.
    #[test]
    fn backoff_guard_only_fires_once_even_if_several_tasks_time_out_at_once() {
        let backed_off = AtomicBool::new(false);
        let mut fired = 0;
        for _ in 0..5 {
            if backed_off
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                fired += 1;
            }
        }
        assert_eq!(
            fired, 1,
            "the guard should allow only ONE firing even with several concurrent timeouts"
        );
    }

    /// Same spirit as the chapter round-trip above, but for extra items (cover/banner/
    /// `ComicInfo.xml`) — proves that `send_extras`/`receive_extras` (which reuse the same
    /// read/write handles used for chapters) deliver the right bytes to the right comic,
    /// with the right `kind`, without mixing with the chapter transfer running in the same
    /// session.
    #[tokio::test]
    async fn run_exchange_transfers_extras_round_trip() {
        let cover_bytes = make_payload(2048, 11);
        let comic_info_bytes = b"<ComicInfo><Title>Berserk</Title></ComicInfo>".to_vec();

        let outbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable_and_extras(
            vec![],
            vec![(
                "Comic A".to_string(),
                EXTRA_KIND_COVER.to_string(),
                "cover.jpg".to_string(),
                cover_bytes.clone(),
            )],
        ));
        let inbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable_and_extras(
            vec![],
            vec![(
                "Comic A".to_string(),
                EXTRA_KIND_COMIC_INFO.to_string(),
                "ComicInfo.xml".to_string(),
                comic_info_bytes.clone(),
            )],
        ));

        let (client_io, server_io) = tokio::io::duplex(256 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let peer = make_peer("peer-extras");
        let emit = no_op_emitter();
        let (outbound_transfer, inbound_transfer) = test_transfer_pair();

        let outbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&outbound_provider) as Arc<dyn FileSyncProvider>;
        let inbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&inbound_provider) as Arc<dyn FileSyncProvider>;

        let outbound_fut = run_exchange(
            true,
            &peer,
            &emit,
            &outbound_dyn,
            &outbound_transfer,
            Box::new(client_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(client_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );
        let inbound_fut = run_exchange(
            false,
            &peer,
            &emit,
            &inbound_dyn,
            &inbound_transfer,
            Box::new(server_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(server_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );

        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        outbound_result.expect("outbound should complete without error");
        inbound_result.expect("inbound should complete without error");

        let outbound_received = outbound_provider.finalized_extras.lock().unwrap();
        assert_eq!(
            outbound_received.len(),
            1,
            "outbound should have received B's ComicInfo.xml"
        );
        assert_eq!(outbound_received[0].comic_name, "Comic A");
        assert_eq!(outbound_received[0].kind, EXTRA_KIND_COMIC_INFO);
        assert_eq!(outbound_received[0].bytes, comic_info_bytes);
        assert_eq!(
            outbound_received[0].expected_checksum,
            outbound_received[0].actual_checksum
        );

        let inbound_received = inbound_provider.finalized_extras.lock().unwrap();
        assert_eq!(
            inbound_received.len(),
            1,
            "inbound should have received A's cover"
        );
        assert_eq!(inbound_received[0].comic_name, "Comic A");
        assert_eq!(inbound_received[0].kind, EXTRA_KIND_COVER);
        assert_eq!(inbound_received[0].bytes, cover_bytes);
        assert_eq!(
            inbound_received[0].expected_checksum,
            inbound_received[0].actual_checksum
        );
    }

    /// Stress: many chapters of varied sizes transferred SEQUENTIALLY in a single
    /// session (same `send_files`/`receive_files` loop). Specific target: the real production
    /// bug fails DIFFERENT chapters on each run, spread across several comics
    /// — the classic signature of a handle being incorrectly reused or of state
    /// (buffer, read position) leaking between consecutive chapters, not of a
    /// deterministic calculation bug. If that kind of leak existed on the Rust side, one
    /// file would receive another's bytes and the checksum would mismatch only on that one — this test
    /// catches it by comparing each received chapter, individually, against its exact original
    /// payload.
    #[tokio::test]
    async fn run_exchange_stress_many_sequential_chapters_preserve_checksum_integrity() {
        const CHAPTER_COUNT: usize = 40;

        let mut outbound_entries = Vec::with_capacity(CHAPTER_COUNT);
        for i in 0..CHAPTER_COUNT {
            let size = 10_000 + (i * 37_123) % 500_000;
            outbound_entries.push((
                "Stress Comic".to_string(),
                format!("Ch. {i}"),
                format!("ch{i}.cbz"),
                make_payload(size, i as u8),
            ));
        }

        let outbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(
            outbound_entries.clone(),
        ));
        let inbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(vec![]));

        let (client_io, server_io) = tokio::io::duplex(1024 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let peer = make_peer("peer-stress");
        let emit = no_op_emitter();
        let (outbound_transfer, inbound_transfer) = test_transfer_pair();

        let outbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&outbound_provider) as Arc<dyn FileSyncProvider>;
        let inbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&inbound_provider) as Arc<dyn FileSyncProvider>;

        let outbound_fut = run_exchange(
            true,
            &peer,
            &emit,
            &outbound_dyn,
            &outbound_transfer,
            Box::new(client_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(client_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );
        let inbound_fut = run_exchange(
            false,
            &peer,
            &emit,
            &inbound_dyn,
            &inbound_transfer,
            Box::new(server_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(server_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );

        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        outbound_result.expect("outbound should complete without error");
        inbound_result.expect("inbound should complete without error");

        let received = inbound_provider.finalized.lock().unwrap();
        assert_eq!(
            received.len(),
            CHAPTER_COUNT,
            "not all chapters arrived"
        );

        let by_chapter: HashMap<&str, &FinalizedChapter> =
            received.iter().map(|f| (f.chapter.as_str(), f)).collect();

        for (comic_name, chapter, _file_name, expected_bytes) in &outbound_entries {
            let got = by_chapter
                .get(chapter.as_str())
                .unwrap_or_else(|| panic!("chapter {chapter} never arrived on the receiving side"));
            assert_eq!(
                &got.comic_name, comic_name,
                "chapter {chapter}: comic_name diverged"
            );
            assert_eq!(
                got.expected_checksum, got.actual_checksum,
                "chapter {chapter}: declared checksum does not match the recomputed one"
            );
            assert_eq!(
                &got.bytes, expected_bytes,
                "chapter {chapter}: received bytes diverge from the original (possible state leak between sequential chapters)"
            );
        }
    }

    // --- `run_exchange_scoped` tests (protocol `acerola/sync-comic/1`) ---
    //
    // They reuse the same `InMemoryFileSyncProvider` above: the only thing that changes is that the
    // outbound has MORE THAN ONE comic in the local manifest, and the test proves that only the
    // requested `comic_name` crosses over — not the whole library (which is already covered by the
    // `run_exchange` tests above).

    /// Push: outbound has two comics, but scopes only one. Proves that `ComicSyncScope`
    /// (phase 0) really filters the local manifest before the exchange — without filtering, the inbound
    /// would receive both comics, not just the requested one.
    #[tokio::test]
    async fn run_exchange_scoped_transfers_only_the_target_comic() {
        let payload_a = make_payload(50_000, 3);
        let payload_b = make_payload(70_000, 9);

        let outbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(vec![
            (
                "Comic A".to_string(),
                "Ch. 1".to_string(),
                "ch1.cbz".to_string(),
                payload_a.clone(),
            ),
            (
                "Comic B".to_string(),
                "Ch. 1".to_string(),
                "ch1.cbz".to_string(),
                payload_b.clone(),
            ),
        ]));
        let inbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(vec![]));

        let (client_io, server_io) = tokio::io::duplex(256 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let peer = make_peer("peer-scoped");
        let emit = no_op_emitter();
        let (outbound_transfer, inbound_transfer) = test_transfer_pair();

        let outbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&outbound_provider) as Arc<dyn FileSyncProvider>;
        let inbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&inbound_provider) as Arc<dyn FileSyncProvider>;

        let outbound_fut = run_exchange_scoped(
            true,
            &peer,
            &emit,
            &outbound_dyn,
            &outbound_transfer,
            Some(("Comic A".to_string(), SyncDirection::Push, vec![])),
            Box::new(client_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(client_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );
        let inbound_fut = run_exchange_scoped(
            false,
            &peer,
            &emit,
            &inbound_dyn,
            &inbound_transfer,
            None,
            Box::new(server_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(server_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );

        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        outbound_result.expect("outbound scoped exchange should complete without error");
        inbound_result.expect("inbound scoped exchange should complete without error");

        let received = inbound_provider.finalized.lock().unwrap();
        assert_eq!(
            received.len(),
            1,
            "only the scoped comic should have been transferred, not the whole library"
        );
        assert_eq!(received[0].comic_name, "Comic A");
        assert_eq!(received[0].bytes, payload_a);
    }

    /// Proves the use case of the chapter selection's "Send" button: scoping the session to a
    /// subset of chapter labels (`ComicSyncScope.chapters`) delivers ONLY the scoped
    /// chapters — even to a peer that does not have the comic yet, which must be created at the
    /// destination containing only what was requested, not the outbound's whole collection.
    #[tokio::test]
    async fn run_exchange_scoped_to_chapters_only_sends_the_requested_ones() {
        let payload_a = make_payload(20_000, 1);
        let payload_b = make_payload(25_000, 2);

        let outbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(vec![
            (
                "Test".to_string(),
                "Cap A".to_string(),
                "Cap A.cbz".to_string(),
                payload_a.clone(),
            ),
            (
                "Test".to_string(),
                "Cap B".to_string(),
                "Cap B.cbz".to_string(),
                payload_b.clone(),
            ),
        ]));
        // Inbound does NOT have "Test" yet — proves the comic is created from scratch with only the
        // scoped chapter, not with the outbound's whole collection.
        let inbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(vec![]));

        let (client_io, server_io) = tokio::io::duplex(256 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let peer = make_peer("peer-scoped-chapters");
        let emit = no_op_emitter();
        let (outbound_transfer, inbound_transfer) = test_transfer_pair();

        let outbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&outbound_provider) as Arc<dyn FileSyncProvider>;
        let inbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&inbound_provider) as Arc<dyn FileSyncProvider>;

        // Only scopes "Cap A" — "Cap B" is left out.
        let outbound_fut = run_exchange_scoped(
            true,
            &peer,
            &emit,
            &outbound_dyn,
            &outbound_transfer,
            Some(("Test".to_string(), SyncDirection::Push, vec!["Cap A".to_string()])),
            Box::new(client_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(client_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );
        let inbound_fut = run_exchange_scoped(
            false,
            &peer,
            &emit,
            &inbound_dyn,
            &inbound_transfer,
            None,
            Box::new(server_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(server_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );

        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        outbound_result.expect("push scoped to chapters should not fail");
        inbound_result.expect("push scoped to chapters should not fail");

        let received = inbound_provider.finalized.lock().unwrap();
        assert_eq!(
            received.len(),
            1,
            "comic should have been created at the destination with only the scoped chapter"
        );
        assert_eq!(received[0].comic_name, "Test");
        assert_eq!(received[0].chapter, "Cap A");
        assert_eq!(received[0].bytes, payload_a);
    }

    /// Pull: the requested comic only exists on the peer (inbound), the outbound has nothing of it
    /// locally. Proves the same symmetric exchange that covers push also covers pull without
    /// any extra code — the outbound's filtered local manifest is empty, so
    /// `missing_from` already resolves "I want everything the peer has of this comic" on its own.
    #[tokio::test]
    async fn run_exchange_scoped_pulls_a_comic_that_only_exists_on_the_peer() {
        let payload = make_payload(30_000, 5);

        let outbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(vec![]));
        let inbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(vec![(
            "Comic Only On Peer".to_string(),
            "Ch. 1".to_string(),
            "ch1.cbz".to_string(),
            payload.clone(),
        )]));

        let (client_io, server_io) = tokio::io::duplex(128 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let peer = make_peer("peer-pull");
        let emit = no_op_emitter();
        let (outbound_transfer, inbound_transfer) = test_transfer_pair();

        let outbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&outbound_provider) as Arc<dyn FileSyncProvider>;
        let inbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&inbound_provider) as Arc<dyn FileSyncProvider>;

        let outbound_fut = run_exchange_scoped(
            true,
            &peer,
            &emit,
            &outbound_dyn,
            &outbound_transfer,
            Some(("Comic Only On Peer".to_string(), SyncDirection::Pull, vec![])),
            Box::new(client_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(client_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );
        let inbound_fut = run_exchange_scoped(
            false,
            &peer,
            &emit,
            &inbound_dyn,
            &inbound_transfer,
            None,
            Box::new(server_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(server_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );

        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        outbound_result.expect("pull should complete without error");
        inbound_result.expect("offering side should complete without error");

        let received = outbound_provider.finalized.lock().unwrap();
        assert_eq!(
            received.len(),
            1,
            "the requester should have received the chapter that only existed on the peer"
        );
        assert_eq!(received[0].comic_name, "Comic Only On Peer");
        assert_eq!(received[0].bytes, payload);
    }

    /// Proves `SyncDirection::Push` is respected even when the regular diff would show the
    /// outbound as "missing" the peer's chapter: outbound delivers "Cap A" (which only it has)
    /// but NEVER requests "Cap B" back, even though it genuinely lacks that chapter. Unlike
    /// the two tests above (where one side has NOTHING of the comic, so the
    /// direction is never really tested — the diff was already zero), here both sides
    /// have real, different content of the SAME comic.
    #[tokio::test]
    async fn run_exchange_scoped_push_never_requests_what_it_is_missing() {
        let payload_a = make_payload(40_000, 3);
        let payload_b = make_payload(60_000, 11);

        let outbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(vec![(
            "Shared Comic".to_string(),
            "Cap A".to_string(),
            "capA.cbz".to_string(),
            payload_a.clone(),
        )]));
        let inbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(vec![(
            "Shared Comic".to_string(),
            "Cap B".to_string(),
            "capB.cbz".to_string(),
            payload_b.clone(),
        )]));

        let (client_io, server_io) = tokio::io::duplex(256 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let peer = make_peer("peer-push");
        let emit = no_op_emitter();
        let (outbound_transfer, inbound_transfer) = test_transfer_pair();

        let outbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&outbound_provider) as Arc<dyn FileSyncProvider>;
        let inbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&inbound_provider) as Arc<dyn FileSyncProvider>;

        let outbound_fut = run_exchange_scoped(
            true,
            &peer,
            &emit,
            &outbound_dyn,
            &outbound_transfer,
            Some(("Shared Comic".to_string(), SyncDirection::Push, vec![])),
            Box::new(client_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(client_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );
        let inbound_fut = run_exchange_scoped(
            false,
            &peer,
            &emit,
            &inbound_dyn,
            &inbound_transfer,
            None,
            Box::new(server_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(server_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );

        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        outbound_result.expect("push should not fail");
        inbound_result.expect("push should not fail");

        assert!(
            outbound_provider.finalized.lock().unwrap().is_empty(),
            "outbound (sender) should never have requested 'Cap B' back"
        );
        let received = inbound_provider.finalized.lock().unwrap();
        assert_eq!(
            received.len(),
            1,
            "inbound (receiver) should have received 'Cap A'"
        );
        assert_eq!(received[0].comic_name, "Shared Comic");
        assert_eq!(received[0].chapter, "Cap A");
        assert_eq!(received[0].bytes, payload_a);
    }

    /// Mirror of the test above for `SyncDirection::Pull` — outbound pulls "Cap B" from the peer
    /// without ever sending "Cap A", even though the inbound genuinely lacks that chapter.
    #[tokio::test]
    async fn run_exchange_scoped_pull_never_sends_what_the_peer_is_missing() {
        let payload_a = make_payload(40_000, 3);
        let payload_b = make_payload(60_000, 11);

        let outbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(vec![(
            "Shared Comic".to_string(),
            "Cap A".to_string(),
            "capA.cbz".to_string(),
            payload_a.clone(),
        )]));
        let inbound_provider = Arc::new(InMemoryFileSyncProvider::with_readable(vec![(
            "Shared Comic".to_string(),
            "Cap B".to_string(),
            "capB.cbz".to_string(),
            payload_b.clone(),
        )]));

        let (client_io, server_io) = tokio::io::duplex(256 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let peer = make_peer("peer-pull");
        let emit = no_op_emitter();
        let (outbound_transfer, inbound_transfer) = test_transfer_pair();

        let outbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&outbound_provider) as Arc<dyn FileSyncProvider>;
        let inbound_dyn: Arc<dyn FileSyncProvider> =
            Arc::clone(&inbound_provider) as Arc<dyn FileSyncProvider>;

        let outbound_fut = run_exchange_scoped(
            true,
            &peer,
            &emit,
            &outbound_dyn,
            &outbound_transfer,
            Some(("Shared Comic".to_string(), SyncDirection::Pull, vec![])),
            Box::new(client_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(client_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );
        let inbound_fut = run_exchange_scoped(
            false,
            &peer,
            &emit,
            &inbound_dyn,
            &inbound_transfer,
            None,
            Box::new(server_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(server_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );

        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        outbound_result.expect("pull should not fail");
        inbound_result.expect("pull should not fail");

        assert!(
            inbound_provider.finalized.lock().unwrap().is_empty(),
            "inbound (sender) should never have requested 'Cap A' back"
        );
        let received = outbound_provider.finalized.lock().unwrap();
        assert_eq!(
            received.len(),
            1,
            "outbound (receiver) should have pulled 'Cap B'"
        );
        assert_eq!(received[0].comic_name, "Shared Comic");
        assert_eq!(received[0].chapter, "Cap B");
        assert_eq!(received[0].bytes, payload_b);
    }

    /// Without `comic_name` on the outbound side (a wiring bug — `P2PNode::sync_comic` always stores the
    /// pending scope before connecting, so this should not happen in production), the session
    /// fails immediately instead of hanging while waiting for the peer to send something that will never come.
    #[tokio::test]
    async fn run_exchange_scoped_outbound_without_comic_name_fails_fast() {
        let provider: Arc<dyn FileSyncProvider> =
            Arc::new(InMemoryFileSyncProvider::with_readable(vec![]));
        let (client_io, _server_io) = tokio::io::duplex(4096);
        let (recv, send) = tokio::io::split(client_io);
        let peer = make_peer("peer-no-scope");
        let emit = no_op_emitter();

        let result = run_exchange_scoped(
            true,
            &peer,
            &emit,
            &provider,
            &test_transfer_pair().0,
            None,
            Box::new(send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(recv) as Box<dyn AsyncRead + Send + Unpin>,
        )
        .await;

        assert!(
            result.is_err(),
            "outbound without comic_name should fail without hanging on the peer"
        );
    }

    /// Real regression (05/09/2026): the dialing side (outbound) was already busy with another
    /// session for the same peer and replied with `SessionBusy` instead of the regular `ComicSyncScope` —
    /// exactly what `write_session_busy` produces (see `run_and_report_scoped`/
    /// `ComicSyncOutbound::handle` on Desktop, mirrored here). The inbound side (`outbound_role:
    /// false`) must recognize this as "peer busy", not try to decode `{"error":
    /// "busy", "reason": "..."}` as `ComicSyncScope` — before this fix, `exchange_comic_scope`
    /// read phase 0 with raw `read_json`, without checking busy, and broke with "missing field
    /// `comic_name`" (seen live: WINDOWS peer showing that raw error on the sync screen).
    #[tokio::test]
    async fn run_exchange_scoped_inbound_reports_clean_busy_error_instead_of_decode_failure() {
        let provider: Arc<dyn FileSyncProvider> =
            Arc::new(InMemoryFileSyncProvider::with_readable(vec![]));
        let (client_io, server_io) = tokio::io::duplex(4096);
        let (_client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);
        let peer = make_peer("peer-busy-scope");
        let emit = no_op_emitter();

        // Simulates the other side (the dialer) already busy: the first thing it sends in phase
        // 0 of `acerola/sync-comic/1` is `SessionBusy`, not `ComicSyncScope`.
        write_session_busy(
            Box::new(client_send) as Box<dyn AsyncWrite + Send + Unpin>,
            "sync-comic session already in progress for this peer",
        )
        .await
        .unwrap();

        let result = run_exchange_scoped(
            false,
            &peer,
            &emit,
            &provider,
            &test_transfer_pair().0,
            None,
            Box::new(server_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(server_recv) as Box<dyn AsyncRead + Send + Unpin>,
        )
        .await;

        let err = result.expect_err("should report an error, not hang or succeed");
        let message = err.to_string();
        assert!(
            message.contains("peer busy"),
            "expected a recognized 'peer busy' error, got: {message}"
        );
        assert!(
            !message.contains("missing field"),
            "should no longer try to decode SessionBusy as ComicSyncScope: {message}"
        );
    }

    /// Same steps and limits as the Desktop equivalent
    /// (`infra/sync/protocol/transfer.rs::chapter_fetch_concurrency`) — both sides must
    /// agree on sizing so a sync between different platforms behaves
    /// predictably under the same network condition.
    #[test]
    fn chapter_fetch_concurrency_scales_with_measured_latency() {
        assert_eq!(
            chapter_fetch_concurrency(None),
            2,
            "without a measurement, stays at the most conservative"
        );
        assert_eq!(chapter_fetch_concurrency(Some(Duration::from_millis(0))), 2);
        assert_eq!(
            chapter_fetch_concurrency(Some(Duration::from_millis(20))),
            2,
            "exact limit of the low step, still within it"
        );
        assert_eq!(
            chapter_fetch_concurrency(Some(Duration::from_millis(21))),
            4,
            "1ms above the low step already moves to the middle one"
        );
        assert_eq!(
            chapter_fetch_concurrency(Some(Duration::from_millis(200))),
            4,
            "exact limit of the middle step, still within it"
        );
        assert_eq!(
            chapter_fetch_concurrency(Some(Duration::from_millis(201))),
            6,
            "1ms above the middle step already moves to the ceiling"
        );
        assert_eq!(
            chapter_fetch_concurrency(Some(Duration::from_secs(2))),
            6,
            "very high RTT (bad relay) never exceeds the ceiling of 6"
        );
    }

    #[test]
    fn permits_to_forget_halves_the_pool_but_never_below_one_permit_left() {
        assert_eq!(permits_to_forget(6), 3, "6 -> keeps 3, forgets 3");
        assert_eq!(permits_to_forget(4), 2, "4 -> keeps 2, forgets 2");
        assert_eq!(permits_to_forget(2), 1, "2 -> keeps max(1,1)=1, forgets 1");
        assert_eq!(
            permits_to_forget(1),
            0,
            "1 -> keeps max(1,0)=1, nothing to forget"
        );
        assert_eq!(
            permits_to_forget(0),
            0,
            "0 -> should never happen in practice, but must not overflow (saturating_sub)"
        );
    }
}
