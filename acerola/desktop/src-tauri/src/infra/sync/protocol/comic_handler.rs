use std::sync::Arc;

use acerola_p2p::api::{
    error::P2pError,
    peer::PeerIdentity,
    protocol::{EventEmitter, Handler},
};
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    core::services::{
        metadata::MetadataService,
        sync::file_sync::{restrict_manifest_to_chapters, FileSyncService},
    },
    data::{models::sync::SyncHistoryLogEntry, repositories::sync::SyncHistoryLogRepository},
    infra::sync::{
        framing::{
            framed_reader, framed_writer, read_json, write_json, FramedReader, FramedWriter,
        },
        messages::{ComicSyncRequest, FileManifest, FileWantList, SyncDirection},
        protocol::{
            comic_sync_registry::PendingComicSyncRegistry,
            file_session_guard::FileSyncSessionGuard,
            transfer::{
                classify_sync_error, emit_busy, read_or_busy, receive_extras, receive_files,
                send_extras, send_files, sync_error_payload, write_session_busy, ChapterTransfer,
                SyncErrorCode, NO_PENDING_SCOPE_REASON,
            },
        },
    },
};

const LOG_KIND: &str = "comic";
const STARTED_EVENT: &str = "sync:comic:started";
const PROGRESS_EVENT: &str = "sync:comic:progress";
const COMPLETE_EVENT: &str = "sync:comic:complete";
const ERROR_EVENT: &str = "sync:comic:error";

/// Side that STARTS an individual sync of ONE comic (`acerola/sync-comic/1`). Reuses the
/// same diff/want-list/transfer mechanism as `acerola/sync-files/1`
/// (`file_handler.rs`/`transfer.rs`), only swapping the whole-library manifest for a
/// manifest scoped to one `comic_name` — see the design comment on
/// `FileSyncService::build_manifest_for_comic`. This solves push and pull with the same code:
/// if I have the comic and the peer doesn't, its manifest comes empty and its `diff_wanted` asks
/// me for everything; if it is the other way around, mine comes empty and I am the one asking for everything.
///
/// Sequence (one message more than `FileSyncOutbound`, at the start, to say WHICH
/// comic — without it the answering side would have no way to scope when my manifest
/// comes empty, the "pull" case):
/// 1. write `ComicSyncRequest{comic_name}` → 2. write scoped local manifest → 3. read
///    peer's manifest (scoped) → 4. write what it wants → 5. read what the peer wants → 6. read
///    the files it requested → 7. write the files the peer requested.
pub struct ComicSyncOutbound {
    emit: EventEmitter,
    service: FileSyncService,
    metadata_service: Arc<MetadataService>,
    log_repo: SyncHistoryLogRepository,
    guard: Arc<FileSyncSessionGuard>,
    registry: Arc<PendingComicSyncRegistry>,
    transfer: Arc<dyn ChapterTransfer>,
}

impl ComicSyncOutbound {
    pub fn new(
        emit: EventEmitter, service: FileSyncService, metadata_service: Arc<MetadataService>,
        log_repo: SyncHistoryLogRepository, guard: Arc<FileSyncSessionGuard>,
        registry: Arc<PendingComicSyncRegistry>, transfer: Arc<dyn ChapterTransfer>,
    ) -> Self {
        Self { emit, service, metadata_service, log_repo, guard, registry, transfer }
    }

    /// Returns `(skipped, comic_name, conflicts)` — `skipped` is how many chapters/extras (from
    /// either phase) were not actually transferred, used by `handle()` to
    /// decide whether the session can really be reported as "complete" (see the docs of
    /// `receive_files`/`send_files` in `transfer.rs`); `comic_name` travels back only so
    /// `handle()` can include it in the `sync:comic:complete`/`error` event payload, since only
    /// `run()` knows which comic this session was scoped to; `conflicts` is how many of those
    /// chapters already existed locally with a checksum different from the peer's (see
    /// `FileSyncService::diff_wanted`) — only filled on the side that actually computed the diff
    /// (`is_sender == false` below), it stays 0 on the side that only serves what was requested.
    async fn run(
        &self, peer: &PeerIdentity, writer: &mut FramedWriter, reader: &mut FramedReader,
    ) -> Result<(usize, String, usize), P2pError> {
        let (comic_name, direction, chapter_ids) = self
            .registry
            .take(&peer.id)
            .ok_or_else(|| P2pError::StreamFailed(NO_PENDING_SCOPE_REASON.into()))?;
        let chapters = self.service.resolve_chapter_labels(&comic_name, &chapter_ids).await?;

        write_json(
            writer,
            &ComicSyncRequest { comic_name: comic_name.clone(), direction, chapters: chapters.clone() },
        )
        .await?;

        let local_manifest = self.service.build_manifest_for_comic(&comic_name).await?;
        let local_manifest = restrict_manifest_to_chapters(local_manifest, &chapters);
        write_json(writer, &local_manifest).await?;

        let peer_manifest: FileManifest = read_or_busy(reader).await?;

        // Outbound is the "sender" when the chosen direction was Push — in that case it requests nothing
        // back, it only serves what the peer requests (see the docs of `SyncDirection`). The diff is still
        // computed normally on the other side (inbound), which decides what to request from me without
        // knowing (or needing to know) the direction.
        let is_sender = direction == SyncDirection::Push;
        let (my_wanted, my_wanted_extras, my_conflicts) = if is_sender {
            (Vec::new(), Vec::new(), Vec::new())
        } else {
            self.service.diff_wanted(&peer_manifest).await?
        };
        write_json(
            writer,
            &FileWantList { wanted: my_wanted.clone(), wanted_extras: my_wanted_extras.clone() },
        )
        .await?;

        let their_wanted: FileWantList = read_json(reader).await?;

        // Phase 1: the peer first sends what I requested (chapters, then extras).
        let received_skipped = receive_files(
            reader,
            my_wanted.len(),
            &self.service,
            &self.emit,
            PROGRESS_EVENT,
            ERROR_EVENT,
            peer,
            &self.transfer,
        )
        .await?;
        let received_extras_skipped = receive_extras(
            reader,
            my_wanted_extras.len(),
            &self.service,
            &self.metadata_service,
            &self.emit,
            PROGRESS_EVENT,
            ERROR_EVENT,
            peer,
            &self.transfer,
        )
        .await?;

        // Phase 2: I send what the peer requested (chapters, then extras).
        let sent_skipped = send_files(
            writer,
            &their_wanted.wanted,
            &self.service,
            &self.emit,
            PROGRESS_EVENT,
            &self.transfer,
        )
        .await?;
        let sent_extras_skipped = send_extras(
            writer,
            &their_wanted.wanted_extras,
            &self.service,
            &self.emit,
            PROGRESS_EVENT,
            &self.transfer,
        )
        .await?;

        Ok((
            received_skipped + received_extras_skipped + sent_skipped + sent_extras_skipped,
            comic_name,
            my_conflicts.len(),
        ))
    }
}

#[async_trait]
impl Handler for ComicSyncOutbound {
    async fn handle(
        &self, peer: &PeerIdentity, send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        let Some(_lease) = self.guard.try_acquire(&peer.id) else {
            write_session_busy(send).await;
            emit_busy(&self.emit, ERROR_EVENT, &peer.id);
            return Ok(());
        };

        let mut writer = framed_writer(send);
        let mut reader = framed_reader(recv);

        (self.emit)(STARTED_EVENT, peer.id.clone());

        match self.run(peer, &mut writer, &mut reader).await {
            Ok((0, comic_name, conflicts)) => {
                (self.emit)(
                    COMPLETE_EVENT,
                    serde_json::json!({
                        "peerId": peer.id,
                        "comicName": comic_name,
                        "conflicts": conflicts,
                    })
                    .to_string(),
                );
                self.log_repo
                    .base
                    .insert(&SyncHistoryLogEntry::new(&peer.id, LOG_KIND, "complete", None))
                    .await
                    .ok();
                Ok(())
            },
            // The session ended without a protocol error, but not every chapter arrived —
            // reporting it as "complete" here would hide exactly the kind of silent data
            // loss that motivated this change (see the docs of `receive_files`).
            Ok((skipped, comic_name, _conflicts)) => {
                let message =
                    format!("{skipped} chapter(s) not synced — see backend log for details");
                tracing::warn!(peer = %peer.id, skipped, "[ComicSync] session finished with missing chapters");
                (self.emit)(
                    ERROR_EVENT,
                    sync_error_payload(
                        &peer.id,
                        &message,
                        Some(SyncErrorCode::PartialSync),
                        Some(&comic_name),
                    ),
                );
                self.log_repo
                    .base
                    .insert(&SyncHistoryLogEntry::new(&peer.id, LOG_KIND, "error", Some(&message)))
                    .await
                    .ok();
                Err(P2pError::StreamFailed(message))
            },
            Err(error) => {
                let message = error.to_string();
                let code = classify_sync_error(&error);
                tracing::warn!(peer = %peer.id, ?code, error = %message, "[ComicSync] session failed");
                (self.emit)(ERROR_EVENT, sync_error_payload(&peer.id, &message, code, None));
                self.log_repo
                    .base
                    .insert(&SyncHistoryLogEntry::new(&peer.id, LOG_KIND, "error", Some(&message)))
                    .await
                    .ok();
                Err(error)
            },
        }
    }
}

/// Side that ANSWERS the individual sync session — same logical sequence as the outbound, with
/// the read/write roles inverted at each step. The `comic_name` to scope comes from the
/// received `ComicSyncRequest`, not from a local registry (only the starting side needs
/// `PendingComicSyncRegistry`).
pub struct ComicSyncInbound {
    emit: EventEmitter,
    service: FileSyncService,
    metadata_service: Arc<MetadataService>,
    log_repo: SyncHistoryLogRepository,
    guard: Arc<FileSyncSessionGuard>,
    transfer: Arc<dyn ChapterTransfer>,
}

impl ComicSyncInbound {
    pub fn new(
        emit: EventEmitter, service: FileSyncService, metadata_service: Arc<MetadataService>,
        log_repo: SyncHistoryLogRepository, guard: Arc<FileSyncSessionGuard>,
        transfer: Arc<dyn ChapterTransfer>,
    ) -> Self {
        Self { emit, service, metadata_service, log_repo, guard, transfer }
    }

    /// See the equivalent docs in `ComicSyncOutbound::run` — same return contract
    /// `(skipped, comic_name, conflicts)`.
    async fn run(
        &self, peer: &PeerIdentity, writer: &mut FramedWriter, reader: &mut FramedReader,
    ) -> Result<(usize, String, usize), P2pError> {
        let request: ComicSyncRequest = read_or_busy(reader).await?;

        let peer_manifest: FileManifest = read_json(reader).await?;

        let local_manifest = self.service.build_manifest_for_comic(&request.comic_name).await?;
        let local_manifest = restrict_manifest_to_chapters(local_manifest, &request.chapters);
        write_json(writer, &local_manifest).await?;

        let their_wanted: FileWantList = read_json(reader).await?;

        // Inbound is the "sender" when the direction chosen by the outbound was Pull (outbound
        // pulls from me) — the role is always the opposite of the outbound's (see the docs of `SyncDirection` and
        // `ComicSyncOutbound::run`), so it requests nothing back, it only serves what was requested.
        let is_sender = request.direction == SyncDirection::Pull;
        let (my_wanted, my_wanted_extras, my_conflicts) = if is_sender {
            (Vec::new(), Vec::new(), Vec::new())
        } else {
            self.service.diff_wanted(&peer_manifest).await?
        };
        write_json(
            writer,
            &FileWantList { wanted: my_wanted.clone(), wanted_extras: my_wanted_extras.clone() },
        )
        .await?;

        // Phase 1: I first send what the peer (outbound) requested (chapters, then extras).
        let sent_skipped = send_files(
            writer,
            &their_wanted.wanted,
            &self.service,
            &self.emit,
            PROGRESS_EVENT,
            &self.transfer,
        )
        .await?;
        let sent_extras_skipped = send_extras(
            writer,
            &their_wanted.wanted_extras,
            &self.service,
            &self.emit,
            PROGRESS_EVENT,
            &self.transfer,
        )
        .await?;

        // Phase 2: I receive what I requested (chapters, then extras).
        let received_skipped = receive_files(
            reader,
            my_wanted.len(),
            &self.service,
            &self.emit,
            PROGRESS_EVENT,
            ERROR_EVENT,
            peer,
            &self.transfer,
        )
        .await?;
        let received_extras_skipped = receive_extras(
            reader,
            my_wanted_extras.len(),
            &self.service,
            &self.metadata_service,
            &self.emit,
            PROGRESS_EVENT,
            ERROR_EVENT,
            peer,
            &self.transfer,
        )
        .await?;

        Ok((
            sent_skipped + sent_extras_skipped + received_skipped + received_extras_skipped,
            request.comic_name,
            my_conflicts.len(),
        ))
    }
}

#[async_trait]
impl Handler for ComicSyncInbound {
    async fn handle(
        &self, peer: &PeerIdentity, send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        let Some(_lease) = self.guard.try_acquire(&peer.id) else {
            write_session_busy(send).await;
            emit_busy(&self.emit, ERROR_EVENT, &peer.id);
            return Ok(());
        };

        let mut writer = framed_writer(send);
        let mut reader = framed_reader(recv);

        (self.emit)(STARTED_EVENT, peer.id.clone());

        match self.run(peer, &mut writer, &mut reader).await {
            Ok((0, comic_name, conflicts)) => {
                (self.emit)(
                    COMPLETE_EVENT,
                    serde_json::json!({
                        "peerId": peer.id,
                        "comicName": comic_name,
                        "conflicts": conflicts,
                    })
                    .to_string(),
                );
                self.log_repo
                    .base
                    .insert(&SyncHistoryLogEntry::new(&peer.id, LOG_KIND, "complete", None))
                    .await
                    .ok();
                Ok(())
            },
            Ok((skipped, comic_name, _conflicts)) => {
                let message =
                    format!("{skipped} chapter(s) not synced — see backend log for details");
                tracing::warn!(peer = %peer.id, skipped, "[ComicSync] session finished with missing chapters");
                (self.emit)(
                    ERROR_EVENT,
                    sync_error_payload(
                        &peer.id,
                        &message,
                        Some(SyncErrorCode::PartialSync),
                        Some(&comic_name),
                    ),
                );
                self.log_repo
                    .base
                    .insert(&SyncHistoryLogEntry::new(&peer.id, LOG_KIND, "error", Some(&message)))
                    .await
                    .ok();
                Err(P2pError::StreamFailed(message))
            },
            Err(error) => {
                let message = error.to_string();
                let code = classify_sync_error(&error);
                tracing::warn!(peer = %peer.id, ?code, error = %message, "[ComicSync] session failed");
                (self.emit)(ERROR_EVENT, sync_error_payload(&peer.id, &message, code, None));
                self.log_repo
                    .base
                    .insert(&SyncHistoryLogEntry::new(&peer.id, LOG_KIND, "error", Some(&message)))
                    .await
                    .ok();
                Err(error)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use acerola_p2p::api::peer::PeerIdentity;

    use super::*;
    use crate::infra::sync::protocol::transfer::InMemoryChapterTransfer;

    type RecordedEvents = Arc<Mutex<Vec<(String, String)>>>;

    fn test_transfer() -> Arc<dyn ChapterTransfer> {
        InMemoryChapterTransfer::shared_pair().0
    }

    fn mock_emitter() -> (EventEmitter, RecordedEvents) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let events_clone = Arc::clone(&events);
        let emit: EventEmitter =
            Arc::new(move |name, data| events_clone.lock().unwrap().push((name.to_string(), data)));
        (emit, events)
    }

    async fn setup(
    ) -> (SyncHistoryLogRepository, FileSyncService, Arc<MetadataService>, tempfile::TempDir) {
        let pool = crate::tests::utils::setup_test_db::setup_test_db().await;
        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path().to_path_buf();
        let service = FileSyncService::new(pool.clone(), move || root.clone());
        let metadata_service = Arc::new(MetadataService::new(pool.clone()));
        let log_repo = SyncHistoryLogRepository::new(pool);
        (log_repo, service, metadata_service, temp_dir)
    }

    /// Without a `comic_name` registered for the peer, the outbound aborts the session without ever
    /// writing anything to the stream — avoids triggering the rest of the protocol with an undefined
    /// scope, which would happen if the Tauri command `sync_comic` forgot to call
    /// `registry.set(...)` before `connect()`.
    #[tokio::test]
    async fn outbound_fails_fast_when_no_comic_is_pending_for_the_peer() {
        let (log_repo, service, metadata_service, _dir) = setup().await;
        let guard = FileSyncSessionGuard::new();
        let registry = PendingComicSyncRegistry::new();
        let (emit, events) = mock_emitter();

        let peer = PeerIdentity { id: "peer-no-scope".to_string(), device_id: None };
        let outbound = ComicSyncOutbound::new(
            emit,
            service,
            metadata_service,
            log_repo,
            guard,
            registry,
            test_transfer(),
        );

        let result = outbound
            .handle(&peer, Box::new(tokio::io::empty()), Box::new(tokio::io::empty()))
            .await;

        assert!(result.is_err());
        let recorded = events.lock().unwrap();
        assert!(recorded.iter().any(|(name, _)| name == "sync:comic:error"));
    }

    /// Same race documented in `file_handler.rs`, applied to the individual protocol: a
    /// second session for the same peer while one is already active is rejected without touching the
    /// service, reusing the `FileSyncSessionGuard` shared with `FILE_SYNC_ALPN`.
    #[tokio::test]
    async fn inbound_rejects_second_session_for_same_peer_without_touching_the_service() {
        let (log_repo, service, metadata_service, _dir) = setup().await;
        let guard = FileSyncSessionGuard::new();
        let (emit, events) = mock_emitter();

        let peer = PeerIdentity { id: "peer-busy".to_string(), device_id: None };
        let _held_lease = guard.try_acquire(&peer.id).expect("first reservation should succeed");

        let inbound = ComicSyncInbound::new(
            emit,
            service,
            metadata_service,
            log_repo,
            guard,
            test_transfer(),
        );

        let result =
            inbound.handle(&peer, Box::new(tokio::io::empty()), Box::new(tokio::io::empty())).await;

        assert!(result.is_ok());
        let recorded = events.lock().unwrap();
        assert_eq!(recorded.len(), 1, "expected only the busy event, got: {recorded:?}");
        assert_eq!(recorded[0].0, "sync:comic:error");
        assert!(recorded[0].1.contains("already in progress"));
        let payload: serde_json::Value = serde_json::from_str(&recorded[0].1).unwrap();
        assert_eq!(payload["code"], "busy", "frontend uses this code to translate the UI message");
    }

    /// Locks the new `sync:comic:complete` payload — it used to carry only the raw peer id
    /// string (`peer.id.clone()`), now a JSON with `comicName` (the comic this session was
    /// scoped to) so the frontend knows WHETHER it needs to refresh the open comic page without
    /// having to guess from which peer/kind it came (see `use-network-sync.svelte.ts`).
    #[tokio::test]
    async fn complete_event_payload_carries_the_scoped_comic_name() {
        let (outbound_log, outbound_service, outbound_metadata, _outbound_dir) = setup().await;
        let (inbound_log, inbound_service, inbound_metadata, _inbound_dir) = setup().await;

        // `FileSyncSessionGuard` is LOCAL state of each device (protects against two concurrent
        // sessions for the SAME peer as seen by that device) — outbound and inbound here
        // simulate two DIFFERENT devices in the same session (via `tokio::io::duplex`), so each
        // one needs its own instance. Sharing a single one (as this test used to do)
        // creates a real race: `tokio::join!` polls `outbound_fut` first, which acquires the
        // lease synchronously before `inbound_fut` is even polled once — inbound always
        // loses the race, falls into the "busy" branch and closes the stream immediately, and the
        // outbound's next write (which had not read that yet) breaks with "broken pipe". Never
        // caught before because plain `cargo test` hung with STATUS_ENTRYPOINT_NOT_FOUND in this
        // crate on Windows (environment) — only via `cargo make test` (nextest, with the Windows
        // manifest/DLL setup) did this test actually run for the first time.
        let outbound_guard = FileSyncSessionGuard::new();
        let inbound_guard = FileSyncSessionGuard::new();
        let registry = PendingComicSyncRegistry::new();
        let (outbound_emit, outbound_events) = mock_emitter();
        let (inbound_emit, inbound_events) = mock_emitter();

        let peer = PeerIdentity { id: "peer-complete".to_string(), device_id: None };
        registry.set(
            peer.id.clone(),
            "Quadrinho Compartilhado".to_string(),
            SyncDirection::Push,
            vec![],
        );

        let outbound = ComicSyncOutbound::new(
            outbound_emit,
            outbound_service,
            outbound_metadata,
            outbound_log,
            outbound_guard,
            registry,
            test_transfer(),
        );
        let inbound = ComicSyncInbound::new(
            inbound_emit,
            inbound_service,
            inbound_metadata,
            inbound_log,
            inbound_guard,
            test_transfer(),
        );

        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let outbound_fut = outbound.handle(&peer, Box::new(client_send), Box::new(client_recv));
        let inbound_fut = inbound.handle(&peer, Box::new(server_send), Box::new(server_recv));

        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        outbound_result
            .expect("outbound should complete without error (no chapter on either side)");
        inbound_result.expect("inbound should complete without error");

        let outbound_recorded = outbound_events.lock().unwrap();
        let complete_event = outbound_recorded
            .iter()
            .find(|(name, _)| name == "sync:comic:complete")
            .expect("should have emitted sync:comic:complete");
        let payload: serde_json::Value = serde_json::from_str(&complete_event.1).unwrap();
        assert_eq!(payload["comicName"], "Quadrinho Compartilhado");
        assert_eq!(payload["peerId"], "peer-complete");
        assert_eq!(payload["conflicts"], 0, "no chapter involved, no conflict possible");

        let inbound_recorded = inbound_events.lock().unwrap();
        let inbound_complete = inbound_recorded
            .iter()
            .find(|(name, _)| name == "sync:comic:complete")
            .expect("inbound should also have emitted sync:comic:complete");
        let inbound_payload: serde_json::Value = serde_json::from_str(&inbound_complete.1).unwrap();
        assert_eq!(inbound_payload["comicName"], "Quadrinho Compartilhado");
    }

    /// Same comic/chapter on both sides, different content (checksum) — the real scenario of the
    /// reported bug ("conflict is neither detected nor reported" when syncing a comic that already
    /// exists on both devices). Proves `sync:comic:complete` carries `conflicts >= 1` on the
    /// side that computed the diff (the `Pull`, which is the one calling `diff_wanted`).
    #[tokio::test]
    async fn comic_complete_event_payload_carries_a_real_conflict_count() {
        let outbound_pool = crate::tests::utils::setup_test_db::setup_test_db().await;
        let inbound_pool = crate::tests::utils::setup_test_db::setup_test_db().await;
        let outbound_dir = tempfile::tempdir().unwrap();
        let inbound_dir = tempfile::tempdir().unwrap();

        crate::tests::utils::setup_test_db::insert_comic_directory(
            &outbound_pool,
            1,
            "Test",
            outbound_dir.path().join("Test").to_str().unwrap(),
        )
        .await;
        crate::tests::utils::setup_test_db::insert_comic_directory(
            &inbound_pool,
            1,
            "Test",
            inbound_dir.path().join("Test").to_str().unwrap(),
        )
        .await;

        let outbound_path = outbound_dir.path().join("Cap 1.cbz");
        let outbound_bytes = b"versao do outbound";
        tokio::fs::write(&outbound_path, outbound_bytes).await.unwrap();
        sqlx::query(
            "INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, checksum, comic_directory_fk, last_modified) VALUES (1, 'Cap 1', ?, '1', 0, ?, 1, 0)",
        )
        .bind(outbound_path.to_str().unwrap())
        .bind(sha256_hex(outbound_bytes))
        .execute(&outbound_pool)
        .await
        .unwrap();

        let inbound_path = inbound_dir.path().join("Cap 1.cbz");
        let inbound_bytes = b"versao do inbound, diferente";
        tokio::fs::write(&inbound_path, inbound_bytes).await.unwrap();
        sqlx::query(
            "INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, checksum, comic_directory_fk, last_modified) VALUES (1, 'Cap 1', ?, '1', 0, ?, 1, 0)",
        )
        .bind(inbound_path.to_str().unwrap())
        .bind(sha256_hex(inbound_bytes))
        .execute(&inbound_pool)
        .await
        .unwrap();

        let outbound_root = outbound_dir.path().to_path_buf();
        let inbound_root = inbound_dir.path().to_path_buf();
        let outbound_service =
            FileSyncService::new(outbound_pool.clone(), move || outbound_root.clone());
        let inbound_service =
            FileSyncService::new(inbound_pool.clone(), move || inbound_root.clone());
        let outbound_metadata = Arc::new(MetadataService::new(outbound_pool.clone()));
        let inbound_metadata = Arc::new(MetadataService::new(inbound_pool.clone()));

        let outbound_guard = FileSyncSessionGuard::new();
        let inbound_guard = FileSyncSessionGuard::new();
        let registry = PendingComicSyncRegistry::new();
        let (outbound_emit, outbound_events) = mock_emitter();
        let (inbound_emit, _inbound_events) = mock_emitter();

        let peer = PeerIdentity { id: "peer-comic-conflict".to_string(), device_id: None };
        // Pull: outbound is the one computing the diff (`is_sender == false`) and detecting the conflict.
        registry.set(peer.id.clone(), "Test".to_string(), SyncDirection::Pull, vec![]);

        let (outbound_transfer, inbound_transfer) = InMemoryChapterTransfer::shared_pair();

        let outbound = ComicSyncOutbound::new(
            outbound_emit,
            outbound_service,
            outbound_metadata,
            SyncHistoryLogRepository::new(outbound_pool.clone()),
            outbound_guard,
            registry,
            outbound_transfer,
        );
        let inbound = ComicSyncInbound::new(
            inbound_emit,
            inbound_service,
            inbound_metadata,
            SyncHistoryLogRepository::new(inbound_pool.clone()),
            inbound_guard,
            inbound_transfer,
        );

        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let (outbound_result, inbound_result) = tokio::join!(
            outbound.handle(&peer, Box::new(client_send), Box::new(client_recv)),
            inbound.handle(&peer, Box::new(server_send), Box::new(server_recv)),
        );
        outbound_result.expect("pull should not fail even with a conflict");
        inbound_result.expect("pull should not fail even with a conflict");

        let recorded = outbound_events.lock().unwrap();
        let complete = recorded
            .iter()
            .find(|(name, _)| name == "sync:comic:complete")
            .expect("outbound should have emitted sync:comic:complete");
        let payload: serde_json::Value = serde_json::from_str(&complete.1).unwrap();
        assert_eq!(
            payload["conflicts"], 1,
            "a different checksum for the same chapter on both sides is a real conflict"
        );
    }

    fn sha256_hex(bytes: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        format!("{:x}", Sha256::digest(bytes))
    }

    struct DirectedScenario {
        outbound_pool: sqlx::SqlitePool,
        inbound_pool: sqlx::SqlitePool,
        outbound_service: FileSyncService,
        inbound_service: FileSyncService,
        outbound_metadata: Arc<MetadataService>,
        inbound_metadata: Arc<MetadataService>,
        _outbound_dir: tempfile::TempDir,
        _inbound_dir: tempfile::TempDir,
    }

    /// Sets up two devices with the SAME comic ("Test"), each with a real chapter (file
    /// on disk + a real sha256 checksum) the other does not have — the minimal scenario to prove
    /// that it is the chosen `direction`, not the diff, that decides who receives what. Uses
    /// `insert_comic_directory` (path inside the `tempfile::TempDir` itself) instead of
    /// `setup_test_db_with_comic` (which writes the fixed path `/test`) because `resolve_comic_dir`
    /// really calls `create_dir_all` on that path — a test should not create directories outside
    /// its own sandbox.
    async fn setup_directed_scenario() -> DirectedScenario {
        let outbound_pool = crate::tests::utils::setup_test_db::setup_test_db().await;
        let inbound_pool = crate::tests::utils::setup_test_db::setup_test_db().await;

        let outbound_dir = tempfile::tempdir().unwrap();
        let inbound_dir = tempfile::tempdir().unwrap();

        crate::tests::utils::setup_test_db::insert_comic_directory(
            &outbound_pool,
            1,
            "Test",
            outbound_dir.path().join("Test").to_str().unwrap(),
        )
        .await;
        crate::tests::utils::setup_test_db::insert_comic_directory(
            &inbound_pool,
            1,
            "Test",
            inbound_dir.path().join("Test").to_str().unwrap(),
        )
        .await;

        let outbound_chapter_path = outbound_dir.path().join("Cap A.cbz");
        let outbound_bytes = b"chapter A bytes";
        tokio::fs::write(&outbound_chapter_path, outbound_bytes).await.unwrap();
        sqlx::query(
            "INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, checksum, comic_directory_fk, last_modified) VALUES (1, 'Cap A', ?, '1', 0, ?, 1, 0)",
        )
        .bind(outbound_chapter_path.to_str().unwrap())
        .bind(sha256_hex(outbound_bytes))
        .execute(&outbound_pool)
        .await
        .unwrap();

        let inbound_chapter_path = inbound_dir.path().join("Cap B.cbz");
        let inbound_bytes = b"chapter B bytes";
        tokio::fs::write(&inbound_chapter_path, inbound_bytes).await.unwrap();
        sqlx::query(
            "INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, checksum, comic_directory_fk, last_modified) VALUES (1, 'Cap B', ?, '1', 0, ?, 1, 0)",
        )
        .bind(inbound_chapter_path.to_str().unwrap())
        .bind(sha256_hex(inbound_bytes))
        .execute(&inbound_pool)
        .await
        .unwrap();

        let outbound_root = outbound_dir.path().to_path_buf();
        let inbound_root = inbound_dir.path().to_path_buf();
        let outbound_service =
            FileSyncService::new(outbound_pool.clone(), move || outbound_root.clone());
        let inbound_service =
            FileSyncService::new(inbound_pool.clone(), move || inbound_root.clone());
        let outbound_metadata = Arc::new(MetadataService::new(outbound_pool.clone()));
        let inbound_metadata = Arc::new(MetadataService::new(inbound_pool.clone()));

        DirectedScenario {
            outbound_pool,
            inbound_pool,
            outbound_service,
            inbound_service,
            outbound_metadata,
            inbound_metadata,
            _outbound_dir: outbound_dir,
            _inbound_dir: inbound_dir,
        }
    }

    async fn chapters_of(pool: &sqlx::SqlitePool) -> Vec<String> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT chapter FROM chapter_archive WHERE comic_directory_fk = (SELECT id FROM comic_directory WHERE name = 'Test') ORDER BY chapter",
        )
        .fetch_all(pool)
        .await
        .unwrap();
        rows.into_iter().map(|(chapter,)| chapter).collect()
    }

    /// Proves `direction: Push` is respected even when the normal diff would show the outbound
    /// as "missing" the peer's chapter: outbound delivers "Cap A" (which only it has) but NEVER
    /// requests "Cap B" back, even though it genuinely lacks that chapter.
    #[tokio::test]
    async fn push_direction_sends_without_ever_requesting_what_it_is_missing() {
        let scenario = setup_directed_scenario().await;
        let outbound_guard = FileSyncSessionGuard::new();
        let inbound_guard = FileSyncSessionGuard::new();
        let registry = PendingComicSyncRegistry::new();
        let (outbound_emit, _outbound_events) = mock_emitter();
        let (inbound_emit, _inbound_events) = mock_emitter();

        let peer = PeerIdentity { id: "peer-push".to_string(), device_id: None };
        registry.set(peer.id.clone(), "Test".to_string(), SyncDirection::Push, vec![]);

        // Take both handles from the SAME pair (`shared_pair`), not `test_transfer()` twice —
        // each `test_transfer()` call creates a NEW isolated pair, discarding the leftover
        // half; without the same "network store" on both sides, a `publish` on one side is never
        // visible to the other's `fetch_reader` (the bug that made this test fail in its first
        // version, with "1 chapter not synced").
        let (outbound_transfer, inbound_transfer) = InMemoryChapterTransfer::shared_pair();

        let outbound = ComicSyncOutbound::new(
            outbound_emit,
            scenario.outbound_service,
            scenario.outbound_metadata,
            SyncHistoryLogRepository::new(scenario.outbound_pool.clone()),
            outbound_guard,
            registry,
            outbound_transfer,
        );
        let inbound = ComicSyncInbound::new(
            inbound_emit,
            scenario.inbound_service,
            scenario.inbound_metadata,
            SyncHistoryLogRepository::new(scenario.inbound_pool.clone()),
            inbound_guard,
            inbound_transfer,
        );

        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let (outbound_result, inbound_result) = tokio::join!(
            outbound.handle(&peer, Box::new(client_send), Box::new(client_recv)),
            inbound.handle(&peer, Box::new(server_send), Box::new(server_recv)),
        );
        outbound_result.expect("push should not fail");
        inbound_result.expect("push should not fail");

        assert_eq!(
            chapters_of(&scenario.outbound_pool).await,
            vec!["Cap A".to_string()],
            "outbound (sender) should never have requested 'Cap B' back"
        );
        assert_eq!(
            chapters_of(&scenario.inbound_pool).await,
            vec!["Cap A".to_string(), "Cap B".to_string()],
            "inbound (receiver) should have received 'Cap A' normally, keeping 'Cap B'"
        );
    }

    /// Proves the use case of the chapter selection "Send" button: scoping the session to a
    /// subset of local `chapter_archive_id`s (`registry.set(..., vec![1])`, only "Cap A")
    /// delivers ONLY the scoped chapter — even to a peer that does not have the comic yet, which
    /// must be created on the destination containing only what was requested, not the outbound's
    /// whole collection.
    #[tokio::test]
    async fn push_direction_scoped_to_chapters_only_sends_the_requested_ones() {
        let outbound_pool = crate::tests::utils::setup_test_db::setup_test_db().await;
        let inbound_pool = crate::tests::utils::setup_test_db::setup_test_db().await;

        let outbound_dir = tempfile::tempdir().unwrap();
        let inbound_dir = tempfile::tempdir().unwrap();

        crate::tests::utils::setup_test_db::insert_comic_directory(
            &outbound_pool,
            1,
            "Test",
            outbound_dir.path().join("Test").to_str().unwrap(),
        )
        .await;
        // Inbound does NOT have "Test" yet — proves the comic is created from scratch with only the
        // scoped chapter, not with the outbound's whole collection.

        let cap_a_path = outbound_dir.path().join("Cap A.cbz");
        let cap_a_bytes = b"chapter A bytes";
        tokio::fs::write(&cap_a_path, cap_a_bytes).await.unwrap();
        sqlx::query(
            "INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, checksum, comic_directory_fk, last_modified) VALUES (1, 'Cap A', ?, '1', 0, ?, 1, 0)",
        )
        .bind(cap_a_path.to_str().unwrap())
        .bind(sha256_hex(cap_a_bytes))
        .execute(&outbound_pool)
        .await
        .unwrap();

        let cap_b_path = outbound_dir.path().join("Cap B.cbz");
        let cap_b_bytes = b"chapter B bytes";
        tokio::fs::write(&cap_b_path, cap_b_bytes).await.unwrap();
        sqlx::query(
            "INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, checksum, comic_directory_fk, last_modified) VALUES (2, 'Cap B', ?, '2', 0, ?, 1, 0)",
        )
        .bind(cap_b_path.to_str().unwrap())
        .bind(sha256_hex(cap_b_bytes))
        .execute(&outbound_pool)
        .await
        .unwrap();

        let outbound_root = outbound_dir.path().to_path_buf();
        let inbound_root = inbound_dir.path().to_path_buf();
        let outbound_service =
            FileSyncService::new(outbound_pool.clone(), move || outbound_root.clone());
        let inbound_service =
            FileSyncService::new(inbound_pool.clone(), move || inbound_root.clone());
        let outbound_metadata = Arc::new(MetadataService::new(outbound_pool.clone()));
        let inbound_metadata = Arc::new(MetadataService::new(inbound_pool.clone()));

        let outbound_guard = FileSyncSessionGuard::new();
        let inbound_guard = FileSyncSessionGuard::new();
        let registry = PendingComicSyncRegistry::new();
        let (outbound_emit, _outbound_events) = mock_emitter();
        let (inbound_emit, _inbound_events) = mock_emitter();

        let peer = PeerIdentity { id: "peer-scoped-push".to_string(), device_id: None };
        // Only scope chapter_archive_id=1 ("Cap A") — "Cap B" (id=2) is left out.
        registry.set(peer.id.clone(), "Test".to_string(), SyncDirection::Push, vec![1]);

        let (outbound_transfer, inbound_transfer) = InMemoryChapterTransfer::shared_pair();

        let outbound = ComicSyncOutbound::new(
            outbound_emit,
            outbound_service,
            outbound_metadata,
            SyncHistoryLogRepository::new(outbound_pool.clone()),
            outbound_guard,
            registry,
            outbound_transfer,
        );
        let inbound = ComicSyncInbound::new(
            inbound_emit,
            inbound_service,
            inbound_metadata,
            SyncHistoryLogRepository::new(inbound_pool.clone()),
            inbound_guard,
            inbound_transfer,
        );

        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let (outbound_result, inbound_result) = tokio::join!(
            outbound.handle(&peer, Box::new(client_send), Box::new(client_recv)),
            inbound.handle(&peer, Box::new(server_send), Box::new(server_recv)),
        );
        outbound_result.expect("scoped push should not fail");
        inbound_result.expect("scoped push should not fail");

        assert_eq!(
            chapters_of(&inbound_pool).await,
            vec!["Cap A".to_string()],
            "the comic should have been created on the destination with only the scoped chapter"
        );

        let comic_row: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM comic_directory WHERE name = 'Test'")
                .fetch_one(&inbound_pool)
                .await
                .unwrap();
        assert_eq!(comic_row.0, 1, "the comic should have been created on the destination");
    }

    /// Mirror of the test above for `direction: Pull` — outbound pulls "Cap B" from the peer without ever
    /// sending "Cap A", even though the inbound genuinely lacks that chapter.
    #[tokio::test]
    async fn pull_direction_receives_without_ever_sending_what_the_peer_is_missing() {
        let scenario = setup_directed_scenario().await;
        let outbound_guard = FileSyncSessionGuard::new();
        let inbound_guard = FileSyncSessionGuard::new();
        let registry = PendingComicSyncRegistry::new();
        let (outbound_emit, _outbound_events) = mock_emitter();
        let (inbound_emit, _inbound_events) = mock_emitter();

        let peer = PeerIdentity { id: "peer-pull".to_string(), device_id: None };
        registry.set(peer.id.clone(), "Test".to_string(), SyncDirection::Pull, vec![]);

        let (outbound_transfer, inbound_transfer) = InMemoryChapterTransfer::shared_pair();

        let outbound = ComicSyncOutbound::new(
            outbound_emit,
            scenario.outbound_service,
            scenario.outbound_metadata,
            SyncHistoryLogRepository::new(scenario.outbound_pool.clone()),
            outbound_guard,
            registry,
            outbound_transfer,
        );
        let inbound = ComicSyncInbound::new(
            inbound_emit,
            scenario.inbound_service,
            scenario.inbound_metadata,
            SyncHistoryLogRepository::new(scenario.inbound_pool.clone()),
            inbound_guard,
            inbound_transfer,
        );

        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let (outbound_result, inbound_result) = tokio::join!(
            outbound.handle(&peer, Box::new(client_send), Box::new(client_recv)),
            inbound.handle(&peer, Box::new(server_send), Box::new(server_recv)),
        );
        outbound_result.expect("pull should not fail");
        inbound_result.expect("pull should not fail");

        assert_eq!(
            chapters_of(&scenario.outbound_pool).await,
            vec!["Cap A".to_string(), "Cap B".to_string()],
            "outbound (receiver) should have pulled 'Cap B', keeping 'Cap A'"
        );
        assert_eq!(
            chapters_of(&scenario.inbound_pool).await,
            vec!["Cap B".to_string()],
            "inbound (sender) should never have requested 'Cap A' back"
        );
    }

    /// Regression (2026-09-05): before this fix, `receive_files` called
    /// `persist_received_chapter(...).await?` without catching the error — unlike the other 4
    /// per-item failure points (missing blob, fetch failed, checksum mismatch, folder
    /// unavailable), a failure HERE aborted the WHOLE session via `?`, even with other
    /// chapters still queued. Proves that the "Cap 1" failure (destination on disk blocked by a
    /// directory, not a regular file) does not prevent "Cap 2" from being received normally right
    /// after — and that the item event (`itemLevel: true`, `code: "persist_failed"`) and the
    /// final session event (`code: "partial_sync"`) are emitted separately, not one instead of the
    /// other.
    #[tokio::test]
    async fn receive_files_skips_a_chapter_that_fails_to_persist_instead_of_aborting_the_session() {
        let sender_pool = crate::tests::utils::setup_test_db::setup_test_db().await;
        let receiver_pool = crate::tests::utils::setup_test_db::setup_test_db().await;

        let sender_dir = tempfile::tempdir().unwrap();
        let receiver_dir = tempfile::tempdir().unwrap();

        crate::tests::utils::setup_test_db::insert_comic_directory(
            &sender_pool,
            1,
            "Test",
            sender_dir.path().join("Test").to_str().unwrap(),
        )
        .await;
        crate::tests::utils::setup_test_db::insert_comic_directory(
            &receiver_pool,
            1,
            "Test",
            receiver_dir.path().join("Test").to_str().unwrap(),
        )
        .await;

        let sender_comic_dir = sender_dir.path().join("Test");
        tokio::fs::create_dir_all(&sender_comic_dir).await.unwrap();
        for (id, chapter, content) in [
            (1i64, "Cap 1", b"chapter one bytes".as_slice()),
            (2, "Cap 2", b"chapter two bytes".as_slice()),
        ] {
            let path = sender_comic_dir.join(format!("{chapter}.cbz"));
            tokio::fs::write(&path, content).await.unwrap();
            sqlx::query(
                "INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, checksum, comic_directory_fk, last_modified) VALUES (?, ?, ?, '1', 0, ?, 1, 0)",
            )
            .bind(id)
            .bind(chapter)
            .bind(path.to_str().unwrap())
            .bind(sha256_hex(content))
            .execute(&sender_pool)
            .await
            .unwrap();
        }

        // Blocks "Cap 1"'s destination on the receiver with a DIRECTORY at the exact place where
        // `persist_received_chapter` would try to `rename` the received file —
        // `tokio::fs::rename` never overwrites a directory with a file (on any
        // platform), so this is the simplest and most deterministic way (without relying on a
        // real race) to make the persistence of ONE specific item fail.
        let receiver_comic_dir = receiver_dir.path().join("Test");
        tokio::fs::create_dir_all(receiver_comic_dir.join("Cap 1.cbz")).await.unwrap();

        let sender_root = sender_dir.path().to_path_buf();
        let receiver_root = receiver_dir.path().to_path_buf();
        let outbound_service =
            FileSyncService::new(sender_pool.clone(), move || sender_root.clone());
        let inbound_service =
            FileSyncService::new(receiver_pool.clone(), move || receiver_root.clone());
        let outbound_metadata = Arc::new(MetadataService::new(sender_pool.clone()));
        let inbound_metadata = Arc::new(MetadataService::new(receiver_pool.clone()));

        let outbound_guard = FileSyncSessionGuard::new();
        let inbound_guard = FileSyncSessionGuard::new();
        let registry = PendingComicSyncRegistry::new();
        let (outbound_emit, _outbound_events) = mock_emitter();
        let (inbound_emit, inbound_events) = mock_emitter();

        let peer = PeerIdentity { id: "peer-partial-persist".to_string(), device_id: None };
        registry.set(peer.id.clone(), "Test".to_string(), SyncDirection::Push, vec![]);

        let (outbound_transfer, inbound_transfer) = InMemoryChapterTransfer::shared_pair();

        let outbound = ComicSyncOutbound::new(
            outbound_emit,
            outbound_service,
            outbound_metadata,
            SyncHistoryLogRepository::new(sender_pool.clone()),
            outbound_guard,
            registry,
            outbound_transfer,
        );
        let inbound = ComicSyncInbound::new(
            inbound_emit,
            inbound_service,
            inbound_metadata,
            SyncHistoryLogRepository::new(receiver_pool.clone()),
            inbound_guard,
            inbound_transfer,
        );

        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let (outbound_result, inbound_result) = tokio::join!(
            outbound.handle(&peer, Box::new(client_send), Box::new(client_recv)),
            inbound.handle(&peer, Box::new(server_send), Box::new(server_recv)),
        );

        outbound_result.expect(
            "outbound (the sender) should not fail — the problem is only on the receiving side",
        );
        assert!(
            inbound_result.is_err(),
            "session should end reporting skipped>0 (PartialSync Err), not silent success"
        );

        assert_eq!(
            chapters_of(&receiver_pool).await,
            vec!["Cap 2".to_string()],
            "Cap 2 should have been received normally EVEN with Cap 1 failing — the session must \
             not abort on the first problematic item"
        );

        let recorded = inbound_events.lock().unwrap();
        let item_error = recorded
            .iter()
            .find(|(name, payload)| {
                name == "sync:comic:error" && payload.contains("\"itemLevel\":true")
            })
            .expect("should have emitted an ITEM error for Cap 1, not only the final session error");
        let payload: serde_json::Value = serde_json::from_str(&item_error.1).unwrap();
        assert_eq!(payload["item"], "Cap 1");
        assert_eq!(payload["code"], "persist_failed");

        let session_error = recorded
            .iter()
            .find(|(name, payload)| {
                name == "sync:comic:error" && payload.contains("\"partial_sync\"")
            })
            .expect("should also have emitted the final SESSION error reporting skipped=1");
        let session_payload: serde_json::Value = serde_json::from_str(&session_error.1).unwrap();
        assert_eq!(session_payload["comicName"], "Test");
    }

    /// Test double that measures how many `fetch_reader` calls are in flight at the same time (observed
    /// peak) — the only way to prove `receive_files` really runs fetches in parallel,
    /// no longer one chapter at a time as before this change.
    struct ConcurrencyTrackingTransfer {
        latency: std::time::Duration,
        fetch_delay: std::time::Duration,
        in_flight: std::sync::atomic::AtomicUsize,
        max_observed: std::sync::atomic::AtomicUsize,
    }

    impl ConcurrencyTrackingTransfer {
        fn new(latency: std::time::Duration, fetch_delay: std::time::Duration) -> Self {
            Self {
                latency,
                fetch_delay,
                in_flight: std::sync::atomic::AtomicUsize::new(0),
                max_observed: std::sync::atomic::AtomicUsize::new(0),
            }
        }

        fn max_observed(&self) -> usize {
            self.max_observed.load(std::sync::atomic::Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl ChapterTransfer for ConcurrencyTrackingTransfer {
        async fn publish(&self, _bytes: Vec<u8>) -> Result<String, P2pError> {
            unimplemented!("this test only exercises the receiving side (fetch_reader)")
        }

        async fn fetch_reader(
            &self, _blob_hash: &str, _peer: &PeerIdentity,
        ) -> Result<Box<dyn AsyncRead + Send + Unpin>, P2pError> {
            let current = self.in_flight.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            self.max_observed.fetch_max(current, std::sync::atomic::Ordering::SeqCst);
            tokio::time::sleep(self.fetch_delay).await;
            self.in_flight.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
            Ok(Box::new(tokio::io::empty()))
        }

        async fn latency(&self, _peer: &PeerIdentity) -> Option<std::time::Duration> {
            Some(self.latency)
        }
    }

    /// Core regression of this change: previously, `receive_files` read a header, blocked on the
    /// WHOLE fetch of that chapter, and only then read the next one — peak concurrency always 1,
    /// regardless of latency. With a simulated RTT of 500ms (> 200ms, cap of 6 concurrent
    /// via `chapter_fetch_concurrency`) and 6 expected chapters, if the fetch really ran
    /// in parallel the observed peak must exceed 1 — proving that reading the NEXT
    /// header is no longer blocked behind the previous chapter's fetch.
    #[tokio::test]
    async fn receive_files_fetches_chapters_concurrently_on_a_high_latency_connection() {
        let pool = crate::tests::utils::setup_test_db::setup_test_db().await;
        let dir = tempfile::tempdir().unwrap();
        let comic_dir = dir.path().join("Test");
        tokio::fs::create_dir_all(&comic_dir).await.unwrap();
        crate::tests::utils::setup_test_db::insert_comic_directory(
            &pool,
            1,
            "Test",
            comic_dir.to_str().unwrap(),
        )
        .await;

        let root = dir.path().to_path_buf();
        let service = FileSyncService::new(pool.clone(), move || root.clone());
        let (emit, _events) = mock_emitter();
        let peer = PeerIdentity { id: "peer-concurrency".to_string(), device_id: None };

        // Kept as a concrete type (not `Arc<dyn ChapterTransfer>`) only to be able to read
        // `max_observed()` afterwards — `receive_files` takes a reference via implicit coercion
        // at the call site itself, no downcast needed.
        let transfer = Arc::new(ConcurrencyTrackingTransfer::new(
            std::time::Duration::from_millis(500),
            std::time::Duration::from_millis(60),
        ));

        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let (_client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, _server_send) = tokio::io::split(server_io);
        let mut writer = framed_writer(Box::new(client_send));
        let mut reader = framed_reader(Box::new(server_recv));

        const CHAPTER_COUNT: usize = 6;
        let writer_task = tokio::spawn(async move {
            for i in 0..CHAPTER_COUNT {
                write_json(
                    &mut writer,
                    &crate::infra::sync::messages::FileHeader {
                        comic_name: "Test".to_string(),
                        chapter: format!("Cap {i}"),
                        file_name: format!("Cap {i}.cbz"),
                        size: 10,
                        checksum: None,
                        blob_hash: Some(format!("hash-{i}")),
                    },
                )
                .await
                .unwrap();
            }
            // Dropping `writer` (end of this task's scope) closes the duplex's write half —
            // without it the receiving side's `FramedReader` would never see EOF after the last header.
        });

        let skipped = receive_files(
            &mut reader,
            CHAPTER_COUNT,
            &service,
            &emit,
            "progress",
            "error",
            &peer,
            &(Arc::clone(&transfer) as Arc<dyn ChapterTransfer>),
        )
        .await
        .unwrap();
        writer_task.await.unwrap();

        assert_eq!(skipped, 0, "all 6 chapters should have been persisted successfully");
        let observed = transfer.max_observed();
        assert!(
            observed >= 2,
            "expected overlapping fetches (peak >= 2) with high RTT, only observed a peak of {observed} \
             — receive_files went back to processing one chapter at a time"
        );
    }
}
