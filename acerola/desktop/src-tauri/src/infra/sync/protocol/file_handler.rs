use std::sync::Arc;

use acerola_p2p::api::{
    error::P2pError,
    peer::PeerIdentity,
    protocol::{EventEmitter, Handler},
};
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    core::services::{metadata::MetadataService, sync::file_sync::FileSyncService},
    data::{models::sync::SyncHistoryLogEntry, repositories::sync::SyncHistoryLogRepository},
    infra::sync::{
        framing::{
            framed_reader, framed_writer, read_json, write_json, FramedReader, FramedWriter,
        },
        messages::FileWantList,
        protocol::{
            file_session_guard::FileSyncSessionGuard,
            transfer::{
                classify_sync_error, emit_busy, read_or_busy, receive_extras, receive_files,
                send_extras, send_files, sync_error_payload, write_session_busy, ChapterTransfer,
                SyncErrorCode,
            },
        },
    },
};

const LOG_KIND: &str = "files";
const STARTED_EVENT: &str = "sync:files:started";
const PROGRESS_EVENT: &str = "sync:files:progress";
const COMPLETE_EVENT: &str = "sync:files:complete";
const ERROR_EVENT: &str = "sync:files:error";

/// Side that STARTS the file sync session. Sequence (each step alternates who
/// writes/reads, never both writing to the stream at the same time):
/// 1. write local manifest → 2. read peer's manifest → 3. write what it wants → 4. read
///    what the peer wants → 5. read the files it requested (the peer writes first in this phase) →
///    6. write the files the peer requested.
pub struct FileSyncOutbound {
    emit: EventEmitter,
    service: FileSyncService,
    metadata_service: Arc<MetadataService>,
    log_repo: SyncHistoryLogRepository,
    guard: Arc<FileSyncSessionGuard>,
    transfer: Arc<dyn ChapterTransfer>,
}

impl FileSyncOutbound {
    pub fn new(
        emit: EventEmitter, service: FileSyncService, metadata_service: Arc<MetadataService>,
        log_repo: SyncHistoryLogRepository, guard: Arc<FileSyncSessionGuard>,
        transfer: Arc<dyn ChapterTransfer>,
    ) -> Self {
        Self { emit, service, metadata_service, log_repo, guard, transfer }
    }

    /// See the docs of `receive_files`/`send_files` (`transfer.rs`) — the first returned `usize` is
    /// the sum of chapters/extras that were not actually transferred across the four phases,
    /// used by `handle()` to avoid reporting the session as "complete" when something was
    /// missing. The second `usize` is how many of those chapters already existed locally with a
    /// checksum different from the peer's — a real conflict, not just "I didn't have it yet" (see
    /// `FileSyncService::diff_wanted`).
    async fn run(
        &self, peer: &PeerIdentity, writer: &mut FramedWriter, reader: &mut FramedReader,
    ) -> Result<(usize, usize), P2pError> {
        let local_manifest = self.service.build_manifest().await?;
        write_json(writer, &local_manifest).await?;

        let peer_manifest = read_or_busy(reader).await?;

        let (my_wanted, my_wanted_extras, my_conflicts) =
            self.service.diff_wanted(&peer_manifest).await?;
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
            my_conflicts.len(),
        ))
    }
}

#[async_trait]
impl Handler for FileSyncOutbound {
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
            Ok((0, conflicts)) => {
                (self.emit)(
                    COMPLETE_EVENT,
                    serde_json::json!({ "peerId": peer.id, "conflicts": conflicts }).to_string(),
                );
                self.log_repo
                    .base
                    .insert(&SyncHistoryLogEntry::new(&peer.id, LOG_KIND, "complete", None))
                    .await
                    .ok();
                Ok(())
            },
            Ok((skipped, _conflicts)) => {
                let message =
                    format!("{skipped} chapter(s) not synced — see backend log for details");
                tracing::warn!(peer = %peer.id, skipped, "[FileSync] session finished with missing chapters");
                (self.emit)(
                    ERROR_EVENT,
                    sync_error_payload(&peer.id, &message, Some(SyncErrorCode::PartialSync), None),
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
                // Technical log always in English, regardless of `code` — it is what goes to the
                // log/support file; the translation (`code`) is only for the UI.
                tracing::warn!(peer = %peer.id, ?code, error = %message, "[FileSync] session failed");
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

/// Side that ANSWERS the file sync session — same logical sequence as the outbound,
/// with the read/write roles inverted at each step.
pub struct FileSyncInbound {
    emit: EventEmitter,
    service: FileSyncService,
    metadata_service: Arc<MetadataService>,
    log_repo: SyncHistoryLogRepository,
    guard: Arc<FileSyncSessionGuard>,
    transfer: Arc<dyn ChapterTransfer>,
}

impl FileSyncInbound {
    pub fn new(
        emit: EventEmitter, service: FileSyncService, metadata_service: Arc<MetadataService>,
        log_repo: SyncHistoryLogRepository, guard: Arc<FileSyncSessionGuard>,
        transfer: Arc<dyn ChapterTransfer>,
    ) -> Self {
        Self { emit, service, metadata_service, log_repo, guard, transfer }
    }

    /// See the equivalent docs in `FileSyncOutbound::run`.
    async fn run(
        &self, peer: &PeerIdentity, writer: &mut FramedWriter, reader: &mut FramedReader,
    ) -> Result<(usize, usize), P2pError> {
        let peer_manifest = read_or_busy(reader).await?;

        let local_manifest = self.service.build_manifest().await?;
        write_json(writer, &local_manifest).await?;

        let their_wanted: FileWantList = read_json(reader).await?;

        let (my_wanted, my_wanted_extras, my_conflicts) =
            self.service.diff_wanted(&peer_manifest).await?;
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
            my_conflicts.len(),
        ))
    }
}

#[async_trait]
impl Handler for FileSyncInbound {
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
            Ok((0, conflicts)) => {
                (self.emit)(
                    COMPLETE_EVENT,
                    serde_json::json!({ "peerId": peer.id, "conflicts": conflicts }).to_string(),
                );
                self.log_repo
                    .base
                    .insert(&SyncHistoryLogEntry::new(&peer.id, LOG_KIND, "complete", None))
                    .await
                    .ok();
                Ok(())
            },
            Ok((skipped, _conflicts)) => {
                let message =
                    format!("{skipped} chapter(s) not synced — see backend log for details");
                tracing::warn!(peer = %peer.id, skipped, "[FileSync] session finished with missing chapters");
                (self.emit)(
                    ERROR_EVENT,
                    sync_error_payload(&peer.id, &message, Some(SyncErrorCode::PartialSync), None),
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
                // Technical log always in English, regardless of `code` — it is what goes to the
                // log/support file; the translation (`code`) is only for the UI.
                tracing::warn!(peer = %peer.id, ?code, error = %message, "[FileSync] session failed");
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

    /// Reproduces the race documented in the fix: a second inbound session arrives for the same
    /// peer while one is already in progress. Without the guard, `handle()` would call
    /// `self.run()` (which opens the manifest via `FileSyncService`) even with another session
    /// active. Since the lease is already reserved before `handle()` runs, the assertion that
    /// only the "busy" event was emitted — never "started" — is the proof that `run()` (and,
    /// therefore, `FileSyncService`) was never touched.
    #[tokio::test]
    async fn inbound_rejects_second_session_for_same_peer_without_touching_the_service() {
        let (log_repo, service, metadata_service, _dir) = setup().await;
        let guard = FileSyncSessionGuard::new();
        let (emit, events) = mock_emitter();

        let peer = PeerIdentity { id: "peer-busy".to_string(), device_id: None };
        let _held_lease = guard.try_acquire(&peer.id).expect("first reservation should succeed");

        let inbound =
            FileSyncInbound::new(emit, service, metadata_service, log_repo, guard, test_transfer());

        let result =
            inbound.handle(&peer, Box::new(tokio::io::empty()), Box::new(tokio::io::empty())).await;

        assert!(result.is_ok());
        let recorded = events.lock().unwrap();
        assert_eq!(recorded.len(), 1, "expected only the busy event, got: {recorded:?}");
        assert_eq!(recorded[0].0, "sync:files:error");
        assert!(recorded[0].1.contains("already in progress"));
        let payload: serde_json::Value = serde_json::from_str(&recorded[0].1).unwrap();
        assert_eq!(payload["code"], "busy", "frontend uses this code to translate the UI message");
    }

    /// Same race as the test above, but crossing roles: the lease was reserved by an
    /// inbound session and the second attempt is outbound — proves the guard is shared
    /// between both roles, not only between sessions of the same type.
    #[tokio::test]
    async fn outbound_and_inbound_share_the_same_guard_per_peer() {
        let (log_repo, service, metadata_service, _dir) = setup().await;
        let guard = FileSyncSessionGuard::new();
        let (emit, events) = mock_emitter();

        let peer = PeerIdentity { id: "peer-cross".to_string(), device_id: None };
        let _held_lease = guard.try_acquire(&peer.id).expect("initial reservation should succeed");

        let outbound = FileSyncOutbound::new(
            emit,
            service,
            metadata_service,
            log_repo,
            guard,
            test_transfer(),
        );

        let result = outbound
            .handle(&peer, Box::new(tokio::io::empty()), Box::new(tokio::io::empty()))
            .await;

        assert!(result.is_ok());
        let recorded = events.lock().unwrap();
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0].0, "sync:files:error");
    }

    /// Without any pre-existing lease, `handle()` must proceed normally up to `run()` —
    /// confirms the guard does not block new sessions after the previous one is released.
    #[tokio::test]
    async fn inbound_proceeds_to_run_when_no_session_is_active() {
        let (log_repo, service, metadata_service, _dir) = setup().await;
        let guard = FileSyncSessionGuard::new();
        let (emit, events) = mock_emitter();

        let peer = PeerIdentity { id: "peer-free".to_string(), device_id: None };
        let inbound =
            FileSyncInbound::new(emit, service, metadata_service, log_repo, guard, test_transfer());

        let _ =
            inbound.handle(&peer, Box::new(tokio::io::empty()), Box::new(tokio::io::empty())).await;

        let recorded = events.lock().unwrap();
        assert!(recorded.iter().any(|(name, _)| name == "sync:files:started"));
    }

    fn sha256_hex(bytes: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        format!("{:x}", Sha256::digest(bytes))
    }

    /// Same comic/chapter on both sides, but with different bytes (hence checksum) — the
    /// real scenario of the reported bug ("conflict is neither detected nor reported"). Proves the
    /// `sync:files:complete` payload carries `conflicts >= 1`, with the behavior of
    /// pulling/overwriting the peer's version unchanged (see `FileSyncService::diff_wanted`).
    #[tokio::test]
    async fn complete_event_payload_carries_a_real_conflict_count() {
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

        // Same chapter label ("Cap 1") on both sides, different content (and checksum).
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
        let (outbound_emit, outbound_events) = mock_emitter();
        let (inbound_emit, inbound_events) = mock_emitter();
        let peer = PeerIdentity { id: "peer-conflict".to_string(), device_id: None };
        let (outbound_transfer, inbound_transfer) = InMemoryChapterTransfer::shared_pair();

        let outbound = FileSyncOutbound::new(
            outbound_emit,
            outbound_service,
            outbound_metadata,
            SyncHistoryLogRepository::new(outbound_pool.clone()),
            outbound_guard,
            outbound_transfer,
        );
        let inbound = FileSyncInbound::new(
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
        outbound_result.expect("session should not fail even with a conflict");
        inbound_result.expect("session should not fail even with a conflict");

        for (label, events) in [("outbound", &outbound_events), ("inbound", &inbound_events)] {
            let recorded = events.lock().unwrap();
            let complete = recorded
                .iter()
                .find(|(name, _)| name == "sync:files:complete")
                .unwrap_or_else(|| panic!("{label} should have emitted sync:files:complete"));
            let payload: serde_json::Value = serde_json::from_str(&complete.1).unwrap();
            assert_eq!(
                payload["conflicts"], 1,
                "{label}: a different checksum for the same chapter on both sides is a real conflict"
            );
        }
    }
}
