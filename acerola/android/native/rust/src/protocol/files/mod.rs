mod exchange;
mod model;
mod session_guard;
pub(crate) mod transfer;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use acerola_p2p::api::{
    error::P2pError,
    peer::PeerIdentity,
    protocol::{EventEmitter, Handler},
};
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{callbacks::FileSyncProvider, protocol::sync_error::classify_sync_error};
use model::FileSyncStats;
pub(crate) use model::SyncDirection;
pub(crate) use session_guard::FileSyncSessionGuard;
pub(crate) use transfer::{BlobChapterTransfer, ChapterTransfer};

pub(crate) const FILE_SYNC_ALPN: &[u8] = b"acerola/sync-files/1";

/// ALPN of the `acerola/sync-comic/1` protocol — same state machine as `sync-files`
/// (manifest -> want-list -> transfer), but filtered to a single comic by an extra phase 0
/// (`ComicSyncScope`, see `exchange::run_exchange_scoped`). A dedicated ALPN (instead of
/// reusing `FILE_SYNC_ALPN`) because the other side must route to the right handler before
/// knowing the session is scoped — the only way to express that without touching the
/// transport lib (`acerola-p2p`, which routes by exact ALPN and carries no payload in `connect()`).
pub(crate) const COMIC_SYNC_ALPN: &[u8] = b"acerola/sync-comic/1";

/// `peer_id -> (comic_name, direction, chapters)` chosen by the user in the
/// `P2PNode::sync_comic` FFI call, read and removed by `ComicSyncOutbound::handle` as soon as the
/// outbound session starts. The only way to carry that choice (which only exists on the Kotlin side)
/// to the `Handler` the transport lib invokes without per-call context — see the comment on
/// `COMIC_SYNC_ALPN`. `chapters` (labels, empty = whole comic) scopes the session to a
/// subset of chapters — same idea as Desktop (`PendingComicSyncRegistry`).
pub(crate) type PendingComicScope = Arc<Mutex<HashMap<String, (String, SyncDirection, Vec<String>)>>>;

/// Outbound role of the `acerola/sync-files/1` protocol — this side started the connection.
pub(crate) struct FileSyncOutbound {
    emit: EventEmitter,
    provider: Arc<dyn FileSyncProvider>,
    session_guard: Arc<FileSyncSessionGuard>,
    transfer: Arc<dyn ChapterTransfer>,
}

impl FileSyncOutbound {
    pub(crate) fn new(
        emit: EventEmitter,
        provider: Arc<dyn FileSyncProvider>,
        session_guard: Arc<FileSyncSessionGuard>,
        transfer: Arc<dyn ChapterTransfer>,
    ) -> Self {
        Self {
            emit,
            provider,
            session_guard,
            transfer,
        }
    }
}

#[async_trait]
impl Handler for FileSyncOutbound {
    async fn handle(
        &self,
        peer: &PeerIdentity,
        send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        run_and_report(
            true,
            peer,
            &self.emit,
            &self.provider,
            &self.session_guard,
            &self.transfer,
            send,
            recv,
        )
        .await
    }
}

/// Inbound role of the `acerola/sync-files/1` protocol — the peer started the connection.
pub(crate) struct FileSyncInbound {
    emit: EventEmitter,
    provider: Arc<dyn FileSyncProvider>,
    session_guard: Arc<FileSyncSessionGuard>,
    transfer: Arc<dyn ChapterTransfer>,
}

impl FileSyncInbound {
    pub(crate) fn new(
        emit: EventEmitter,
        provider: Arc<dyn FileSyncProvider>,
        session_guard: Arc<FileSyncSessionGuard>,
        transfer: Arc<dyn ChapterTransfer>,
    ) -> Self {
        Self {
            emit,
            provider,
            session_guard,
            transfer,
        }
    }
}

#[async_trait]
impl Handler for FileSyncInbound {
    async fn handle(
        &self,
        peer: &PeerIdentity,
        send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        run_and_report(
            false,
            peer,
            &self.emit,
            &self.provider,
            &self.session_guard,
            &self.transfer,
            send,
            recv,
        )
        .await
    }
}

#[allow(clippy::too_many_arguments)]
async fn run_and_report(
    outbound_role: bool,
    peer: &PeerIdentity,
    emit: &EventEmitter,
    provider: &Arc<dyn FileSyncProvider>,
    session_guard: &Arc<FileSyncSessionGuard>,
    transfer: &Arc<dyn ChapterTransfer>,
    send: Box<dyn AsyncWrite + Send + Unpin>,
    recv: Box<dyn AsyncRead + Send + Unpin>,
) -> Result<(), P2pError> {
    let result = match session_guard.try_acquire(&peer.id) {
        Some(_lease) => {
            exchange::run_exchange(outbound_role, peer, emit, provider, transfer, send, recv).await
        }
        None => {
            let reason = format!(
                "sync-files session already in progress for peer {}",
                peer.id
            );
            // Best effort: tells the other side of the stream the rejection was due to being busy,
            // not a connection failure. If this write fails (peer already gone, stream already closed),
            // ignore it — the session was already rejected locally anyway, the error below is
            // returned all the same.
            let _ = exchange::write_session_busy(send, &reason).await;
            Err(P2pError::StreamFailed(reason))
        }
    };

    match result {
        Ok(stats) => {
            emit("sync:files:complete", complete_payload(peer, &stats));
            Ok(())
        }
        Err(err) => {
            let reason = err.to_string();
            let code = classify_sync_error(&err);
            // Technical log always in English, regardless of `code` — it is what goes to logs/
            // support; the translation (`code`) is only for the UI.
            tracing::warn!(peer = %peer.id, ?code, error = %reason, "[FileSync] session failed");
            emit(
                "sync:files:chapter_failed",
                serde_json::json!({
                    "peerId": peer.id,
                    "comicName": "",
                    "chapter": "",
                    "reason": reason,
                    "code": code,
                })
                .to_string(),
            );
            Err(err)
        }
    }
}

/// Outbound role of the `acerola/sync-comic/1` protocol — this side dialed. Retrieves (and removes)
/// the `comic_name` the user chose, stored in `pending_scope` by `P2PNode::sync_comic`
/// right before calling `connect()` — the only way for that choice, which only exists on the
/// Kotlin side, to get here (see `COMIC_SYNC_ALPN`).
pub(crate) struct ComicSyncOutbound {
    emit: EventEmitter,
    provider: Arc<dyn FileSyncProvider>,
    session_guard: Arc<FileSyncSessionGuard>,
    transfer: Arc<dyn ChapterTransfer>,
    pending_scope: PendingComicScope,
}

impl ComicSyncOutbound {
    pub(crate) fn new(
        emit: EventEmitter,
        provider: Arc<dyn FileSyncProvider>,
        session_guard: Arc<FileSyncSessionGuard>,
        transfer: Arc<dyn ChapterTransfer>,
        pending_scope: PendingComicScope,
    ) -> Self {
        Self {
            emit,
            provider,
            session_guard,
            transfer,
            pending_scope,
        }
    }
}

#[async_trait]
impl Handler for ComicSyncOutbound {
    async fn handle(
        &self,
        peer: &PeerIdentity,
        send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        let pending = self
            .pending_scope
            .lock()
            .expect("pending comic scope mutex poisoned")
            .remove(&peer.id);
        run_and_report_scoped(
            true,
            peer,
            &self.emit,
            &self.provider,
            &self.session_guard,
            &self.transfer,
            pending,
            send,
            recv,
        )
        .await
    }
}

/// Inbound role of the `acerola/sync-comic/1` protocol — the peer dialed requesting (or offering) a
/// specific comic. The `comic_name` arrives over the wire (phase 0, `ComicSyncScope`), it is not
/// known locally before the session starts — that is why it needs no `pending_scope`.
pub(crate) struct ComicSyncInbound {
    emit: EventEmitter,
    provider: Arc<dyn FileSyncProvider>,
    session_guard: Arc<FileSyncSessionGuard>,
    transfer: Arc<dyn ChapterTransfer>,
}

impl ComicSyncInbound {
    pub(crate) fn new(
        emit: EventEmitter,
        provider: Arc<dyn FileSyncProvider>,
        session_guard: Arc<FileSyncSessionGuard>,
        transfer: Arc<dyn ChapterTransfer>,
    ) -> Self {
        Self {
            emit,
            provider,
            session_guard,
            transfer,
        }
    }
}

#[async_trait]
impl Handler for ComicSyncInbound {
    async fn handle(
        &self,
        peer: &PeerIdentity,
        send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        run_and_report_scoped(
            false,
            peer,
            &self.emit,
            &self.provider,
            &self.session_guard,
            &self.transfer,
            None,
            send,
            recv,
        )
        .await
    }
}

/// Same orchestration as `run_and_report` (guard -> exchange -> emit complete/failed), but
/// for the exchange scoped to one comic (`exchange::run_exchange_scoped`). `session_guard` is the
/// SAME instance shared with `acerola/sync-files/1` (see `api.rs::P2PNode::new`) — a
/// full sync and a single-comic sync for the same peer never run at the same time,
/// for exactly the same reason (doubling the library scan/I-O) that already applies between two
/// `sync-files` sessions. That is why the emitted events are the SAME usual `sync:files:*` —
/// there is never ambiguity about which session is active for a peer.
#[allow(clippy::too_many_arguments)]
async fn run_and_report_scoped(
    outbound_role: bool,
    peer: &PeerIdentity,
    emit: &EventEmitter,
    provider: &Arc<dyn FileSyncProvider>,
    session_guard: &Arc<FileSyncSessionGuard>,
    transfer: &Arc<dyn ChapterTransfer>,
    comic_scope_outbound: Option<(String, SyncDirection, Vec<String>)>,
    send: Box<dyn AsyncWrite + Send + Unpin>,
    recv: Box<dyn AsyncRead + Send + Unpin>,
) -> Result<(), P2pError> {
    let result = match session_guard.try_acquire(&peer.id) {
        Some(_lease) => {
            exchange::run_exchange_scoped(
                outbound_role,
                peer,
                emit,
                provider,
                transfer,
                comic_scope_outbound,
                send,
                recv,
            )
            .await
        }
        None => {
            let reason = format!(
                "sync-comic session already in progress for peer {}",
                peer.id
            );
            let _ = exchange::write_session_busy(send, &reason).await;
            Err(P2pError::StreamFailed(reason))
        }
    };

    match result {
        Ok(stats) => {
            emit("sync:files:complete", complete_payload(peer, &stats));
            Ok(())
        }
        Err(err) => {
            let reason = err.to_string();
            let code = classify_sync_error(&err);
            // Technical log always in English, regardless of `code` — it is what goes to logs/
            // support; the translation (`code`) is only for the UI.
            tracing::warn!(peer = %peer.id, ?code, error = %reason, "[FileSync] session failed");
            emit(
                "sync:files:chapter_failed",
                serde_json::json!({
                    "peerId": peer.id,
                    "comicName": "",
                    "chapter": "",
                    "reason": reason,
                    "code": code,
                })
                .to_string(),
            );
            Err(err)
        }
    }
}

fn complete_payload(peer: &PeerIdentity, stats: &FileSyncStats) -> String {
    serde_json::json!({
        "peerId": peer.id,
        "receivedCount": stats.received_count,
        "sentCount": stats.sent_count,
        "failedCount": stats.failed_count,
        "conflictsCount": stats.conflicts_count,
    })
    .to_string()
}

#[cfg(test)]
mod concurrency_tests {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex as StdMutex,
    };

    use super::*;
    use crate::callbacks::FfiFileManifestEntry;

    fn test_transfer() -> Arc<dyn ChapterTransfer> {
        transfer::InMemoryChapterTransfer::shared_pair().0
    }

    struct CountingProvider {
        manifest_calls: Arc<AtomicUsize>,
    }

    impl FileSyncProvider for CountingProvider {
        fn get_file_manifest(&self) -> Vec<FfiFileManifestEntry> {
            self.manifest_calls.fetch_add(1, Ordering::SeqCst);
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

    /// Proves the full wiring: with the first session's lease still held, a second
    /// attempt for the SAME peer is rejected without ever touching the provider (no library
    /// scan triggered), and emits the existing `sync:files:chapter_failed` event — without
    /// inventing a new, non-standard event.
    #[tokio::test]
    async fn second_concurrent_session_for_same_peer_is_rejected_and_emits_chapter_failed() {
        let session_guard = Arc::new(FileSyncSessionGuard::default());
        let manifest_calls = Arc::new(AtomicUsize::new(0));
        let provider: Arc<dyn FileSyncProvider> = Arc::new(CountingProvider {
            manifest_calls: Arc::clone(&manifest_calls),
        });
        let peer = PeerIdentity {
            id: "peer-x".into(),
            device_id: None,
        };

        let events: Arc<StdMutex<Vec<(String, String)>>> = Arc::new(StdMutex::new(vec![]));
        let events_clone = Arc::clone(&events);
        let emit: EventEmitter = Arc::new(move |event, data| {
            events_clone.lock().unwrap().push((event.to_string(), data));
        });

        // First session acquires the lease and "stays in progress" (kept alive manually).
        let lease = session_guard.try_acquire(&peer.id);
        assert!(lease.is_some());

        let (_client, server) = tokio::io::duplex(4096);
        let (recv, send) = tokio::io::split(server);
        let result = run_and_report(
            true,
            &peer,
            &emit,
            &provider,
            &session_guard,
            &test_transfer(),
            Box::new(send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(recv) as Box<dyn AsyncRead + Send + Unpin>,
        )
        .await;

        assert!(result.is_err());
        assert_eq!(
            manifest_calls.load(Ordering::SeqCst),
            0,
            "provider should not be touched in the rejected session"
        );

        let recorded = events.lock().unwrap();
        let (event_name, payload) = recorded.last().unwrap();
        assert_eq!(event_name, "sync:files:chapter_failed");
        assert!(payload.contains("already in progress"));
        let parsed: serde_json::Value = serde_json::from_str(payload).unwrap();
        assert_eq!(
            parsed["code"], "busy",
            "Kotlin side uses this code to translate the message in the UI"
        );
        drop(recorded);

        drop(lease); // releases the first session
        assert!(session_guard.try_acquire(&peer.id).is_some());
    }

    /// Before this fix, the rejected side wrote nothing to the stream — whoever was waiting on the
    /// other side only saw the stream close with no context (`StreamFailed("stream closed")`), just
    /// like a connection that really dropped. Now the rejection writes `SessionBusy` to the stream
    /// before returning the local error, so the reader can tell "peer busy" from "connection
    /// really dropped" by the error message.
    #[tokio::test]
    async fn peer_reading_rejected_stream_sees_busy_error_not_generic_stream_failed() {
        let session_guard = Arc::new(FileSyncSessionGuard::default());
        let manifest_calls = Arc::new(AtomicUsize::new(0));
        let provider: Arc<dyn FileSyncProvider> = Arc::new(CountingProvider {
            manifest_calls: Arc::clone(&manifest_calls),
        });
        let peer = PeerIdentity {
            id: "peer-x".into(),
            device_id: None,
        };
        let emit: EventEmitter = Arc::new(|_event, _data| {});

        // Session already in progress locally (simulates stream #1 accepted in the fix 1 race).
        let lease = session_guard.try_acquire(&peer.id);
        assert!(lease.is_some());

        // Two unidirectional pipes (not a single bidirectional `duplex`) on purpose: iroh's
        // `SendStream`/`RecvStream` are independent objects — closing the read side of one
        // does not take down the other's write. A single bidirectional `duplex` couples
        // both directions behind a shared `Arc`, so dropping the rejecter's side
        // (which happens as soon as `run_and_report` returns, still inside the same
        // `join!`) would also take down the dialer's write before it could send the
        // manifest — a mock artifact, not real transport behavior.
        let (dialer_to_server, server_reads_dialer) = tokio::io::duplex(4096);
        let (server_to_dialer, dialer_reads_server) = tokio::io::duplex(4096);
        let (_unused_read, client_send) = tokio::io::split(dialer_to_server);
        let (server_recv, _unused_write) = tokio::io::split(server_reads_dialer);
        let (_unused_read2, server_send) = tokio::io::split(server_to_dialer);
        let (client_recv, _unused_write2) = tokio::io::split(dialer_reads_server);

        let dialer_transfer = test_transfer();
        let rejecter_transfer = test_transfer();

        // Dialing side (outbound) waiting for the manifest normally.
        let dialer = exchange::run_exchange(
            true,
            &peer,
            &emit,
            &provider,
            &dialer_transfer,
            Box::new(client_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(client_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );

        // Side that is already busy (simulates the duplicated stream #2, rejected by the guard).
        let rejecter = run_and_report(
            false,
            &peer,
            &emit,
            &provider,
            &session_guard,
            &rejecter_transfer,
            Box::new(server_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(server_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );

        let (dialer_result, rejecter_result) = tokio::join!(dialer, rejecter);

        let dialer_error = dialer_result.expect_err("dialer should fail, peer was busy");
        assert!(
            dialer_error.to_string().contains("peer busy"),
            "expected a typed peer busy error, got: {dialer_error}"
        );

        let rejecter_error = rejecter_result.expect_err("local rejection must also return an error");
        assert!(rejecter_error.to_string().contains("already in progress"));
    }

    /// `acerola/sync-comic/1` shares the SAME `FileSyncSessionGuard` instance with
    /// `acerola/sync-files/1` (see `api.rs::P2PNode::new`) — a full sync session already
    /// in progress for the peer must reject a single-comic sync attempt, even though the
    /// ALPNs differ. Without this, both protocols would run at the same time for the same
    /// peer and double the library scan via `run_blocking`.
    #[tokio::test]
    async fn comic_sync_session_is_rejected_while_a_full_sync_files_session_is_active_for_the_same_peer(
    ) {
        let session_guard = Arc::new(FileSyncSessionGuard::default());
        let manifest_calls = Arc::new(AtomicUsize::new(0));
        let provider: Arc<dyn FileSyncProvider> = Arc::new(CountingProvider {
            manifest_calls: Arc::clone(&manifest_calls),
        });
        let peer = PeerIdentity {
            id: "peer-shared-guard".into(),
            device_id: None,
        };
        let emit: EventEmitter = Arc::new(|_event, _data| {});

        // Simulates a full `sync-files` session already in progress for the peer.
        let lease = session_guard.try_acquire(&peer.id);
        assert!(lease.is_some());

        let (_client, server) = tokio::io::duplex(4096);
        let (recv, send) = tokio::io::split(server);
        let result = run_and_report_scoped(
            true,
            &peer,
            &emit,
            &provider,
            &session_guard,
            &test_transfer(),
            Some(("Comic A".to_string(), SyncDirection::Push, vec![])),
            Box::new(send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(recv) as Box<dyn AsyncRead + Send + Unpin>,
        )
        .await;

        assert!(
            result.is_err(),
            "sync-comic should be rejected while sync-files is active for the same peer"
        );
        assert_eq!(
            manifest_calls.load(Ordering::SeqCst),
            0,
            "provider should not be touched in the rejected session"
        );
    }

    /// Symmetric to the test above: a `sync-comic` session already in progress also rejects a
    /// full `sync-files` attempt for the same peer, through the same shared guard.
    #[tokio::test]
    async fn full_sync_files_session_is_rejected_while_a_comic_sync_session_is_active_for_the_same_peer(
    ) {
        let session_guard = Arc::new(FileSyncSessionGuard::default());
        let manifest_calls = Arc::new(AtomicUsize::new(0));
        let provider: Arc<dyn FileSyncProvider> = Arc::new(CountingProvider {
            manifest_calls: Arc::clone(&manifest_calls),
        });
        let peer = PeerIdentity {
            id: "peer-shared-guard-2".into(),
            device_id: None,
        };
        let emit: EventEmitter = Arc::new(|_event, _data| {});

        // Simulates a `sync-comic` session already in progress for the peer.
        let lease = session_guard.try_acquire(&peer.id);
        assert!(lease.is_some());

        let (_client, server) = tokio::io::duplex(4096);
        let (recv, send) = tokio::io::split(server);
        let result = run_and_report(
            true,
            &peer,
            &emit,
            &provider,
            &session_guard,
            &test_transfer(),
            Box::new(send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(recv) as Box<dyn AsyncRead + Send + Unpin>,
        )
        .await;

        assert!(
            result.is_err(),
            "sync-files should be rejected while sync-comic is active for the same peer"
        );
        assert_eq!(
            manifest_calls.load(Ordering::SeqCst),
            0,
            "provider should not be touched in the rejected session"
        );
    }
}
