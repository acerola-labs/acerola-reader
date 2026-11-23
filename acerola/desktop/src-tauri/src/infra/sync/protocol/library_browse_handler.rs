use acerola_p2p::api::{
    error::P2pError,
    peer::PeerIdentity,
    protocol::{EventEmitter, Handler},
};
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{
    core::services::sync::file_sync::FileSyncService,
    infra::sync::{
        framing::{
            framed_reader, framed_writer, read_json, write_json, FramedReader, FramedWriter,
        },
        messages::{LibraryBrowseRequest, LibrarySummary},
        protocol::transfer::{classify_sync_error, sync_error_payload},
    },
};

/// Single round-trip (no `FileSyncSessionGuard`: it only lists titles, transfers nothing, same
/// cost profile as `history_handler.rs`) to find out what exists in the remote library
/// before deciding to sync a specific comic (`acerola/sync-comic/1`). Does not persist
/// to `sync_history_log` — it is a browsing query, not a content sync.
///
/// The request itself carries no data (see `LibraryBrowseRequest`) — but it must be
/// WRITTEN to the wire by WHOEVER DIALS (`open_bi()`), not just "exist as an intent". A rule of
/// `quinn` itself: whoever calls `open_bi()` must write to the `SendStream` before the accepting
/// side can `accept_bi()` — without that the inbound handler is never even invoked. That was
/// exactly the bug reopened on 2026-08-22 (30s timeout on both sides): this function only
/// read, it never wrote.
///
/// Important: this rule is about MATERIALIZING the stream for the accepting side, not about the
/// accepting side having to READ what was written before answering — that is a separate
/// protocol decision, and `LibraryBrowseInbound` deliberately does NOT read this marker (see the
/// docs there). A previous fix, in this same round, tried to make the inbound drain the marker
/// before answering — that broke the whole session whenever the stream materialization
/// was delayed (slow NAT traversal: the connection "establishes" at handshake level well before the
/// data path is ready to carry the marker), because it coupled the inbound's answer to that
/// delivery, when the inbound does not need it for anything.
pub struct LibraryBrowseOutbound {
    emit: EventEmitter,
}

impl LibraryBrowseOutbound {
    pub fn new(emit: EventEmitter) -> Self {
        Self { emit }
    }

    async fn run(
        &self, writer: &mut FramedWriter, reader: &mut FramedReader,
    ) -> Result<LibrarySummary, P2pError> {
        write_json(writer, &LibraryBrowseRequest::default()).await?;
        let response: LibrarySummary = read_json(reader).await?;
        Ok(response)
    }
}

#[async_trait]
impl Handler for LibraryBrowseOutbound {
    async fn handle(
        &self, peer: &PeerIdentity, send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        let mut writer = framed_writer(send);
        let mut reader = framed_reader(recv);

        match self.run(&mut writer, &mut reader).await {
            Ok(response) => {
                // `ComicSummaryEntry` (wire message, `snake_case`, shared with
                // Android) is not the same format as the event payload for the frontend
                // (`camelCase`, same convention as `ConnectedPeerPayload`/`PairedPeerPayload` in
                // `cmd/events/network`) — builds the event JSON by hand instead of serializing the
                // wire struct directly, the same pattern already used by this module's `sync:*:error`
                // events (ad-hoc `serde_json::json!`, no struct shared between
                // `infra` and `cmd`).
                let comics: Vec<serde_json::Value> = response
                    .comics
                    .iter()
                    .map(|comic| {
                        serde_json::json!({
                            "comicName": comic.comic_name,
                            "chapterCount": comic.chapter_count,
                            "coverVersion": comic.cover_version,
                        })
                    })
                    .collect();

                (self.emit)(
                    "library:query:result",
                    serde_json::json!({ "peerId": peer.id, "comics": comics }).to_string(),
                );
                Ok(())
            },
            Err(error) => {
                let message = error.to_string();
                let code = classify_sync_error(&error);
                tracing::warn!(peer = %peer.id, ?code, error = %message, "[LibraryBrowse] query failed");
                (self.emit)(
                    "library:query:error",
                    sync_error_payload(&peer.id, &message, code, None),
                );
                Err(error)
            },
        }
    }
}

/// Side that ANSWERS: builds the local library summary from the same
/// `FileSyncService::build_manifest()` used by file sync (without duplicating the logic of
/// listing comics/chapters) and writes as soon as the connection is accepted — it does NOT wait for nor read the
/// marker `LibraryBrowseOutbound` writes (see the docs there): that marker only exists to
/// satisfy `quinn`'s rule on the dialing side, it is not something this side needs
/// to consume. Waiting for it here would couple the answer to how long the P2P path (NAT
/// traversal/multipath) takes to be ready to carry real data — which can be much
/// longer than the connection's initial handshake (the "connection established" in the logs) — and it was
/// exactly that unnecessary wait that broke the whole session whenever the other side
/// timed out waiting for an answer that never came because this handler was stuck
/// waiting to read something it did not need.
pub struct LibraryBrowseInbound {
    service: FileSyncService,
}

impl LibraryBrowseInbound {
    pub fn new(service: FileSyncService) -> Self {
        Self { service }
    }

    /// Own timeout, much shorter than the other side's `FRAME_TIMEOUT` (30s) — if the query hangs
    /// (e.g. contention in the SQLite connection pool due to another sync session in progress),
    /// it fails fast and logs the cause instead of letting the outbound blow the frame timeout with
    /// no clue about what happened on this side. See the note on the bug reopened on 2026-08-22: the
    /// handler never logged anything, so there was no way to tell whether it hung here or on the write.
    const LIBRARY_SUMMARY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

    async fn run(&self, writer: &mut FramedWriter) -> Result<(), P2pError> {
        // Pure SQL (`get_library_summary`), no disk access — `build_manifest()` does a
        // `tokio::fs::metadata()` per chapter just to fill `size`, which this answer does not even use,
        // and that blew the protocol timeout on large libraries/slow disks.
        tracing::debug!(layer = "library_browse", "querying local library summary");
        let comics = match tokio::time::timeout(
            Self::LIBRARY_SUMMARY_TIMEOUT,
            self.service.get_library_summary(),
        )
        .await
        {
            Ok(result) => result?,
            Err(_) => {
                tracing::warn!(
                    layer = "library_browse",
                    "get_library_summary did not return within {:?} — DB pool likely contended by another sync session",
                    Self::LIBRARY_SUMMARY_TIMEOUT
                );
                return Err(P2pError::StreamFailed(
                    "timed out querying local library summary".into(),
                ));
            },
        };

        tracing::debug!(
            layer = "library_browse",
            comics = comics.len(),
            "writing library summary response"
        );
        write_json(writer, &LibrarySummary { comics }).await?;
        tracing::debug!(layer = "library_browse", "library summary response written");
        Ok(())
    }
}

#[async_trait]
impl Handler for LibraryBrowseInbound {
    async fn handle(
        &self, peer: &PeerIdentity, send: Box<dyn AsyncWrite + Send + Unpin>,
        _recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        tracing::debug!(layer = "library_browse", peer = %peer.id, "inbound connection accepted");
        let mut writer = framed_writer(send);
        let result = self.run(&mut writer).await;
        if let Err(error) = &result {
            tracing::warn!(layer = "library_browse", peer = %peer.id, %error, "inbound run failed");
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_service() -> (FileSyncService, tempfile::TempDir) {
        let pool = crate::tests::utils::setup_test_db::setup_test_db_with_comic().await;
        crate::tests::utils::setup_test_db::insert_chapter_archive_with_checksum(
            &pool,
            1,
            1,
            "Cap 1",
            "p/Cap 1.cbz",
            "abc",
        )
        .await;

        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path().to_path_buf();
        let service = FileSyncService::new(pool, move || root.clone());
        (service, temp_dir)
    }

    /// Real round-trip over a `tokio::io::duplex`: proves the outbound writes the
    /// `LibraryBrowseRequest` (satisfies `quinn`'s rule on the dialing side) and still reads the
    /// answer correctly, even with the inbound NEVER reading that marker — the inbound answers
    /// directly, without depending on decoding anything from the other side first (see the docs of
    /// `LibraryBrowseInbound`). Regression for a previous fix in this same round that made
    /// the inbound drain the marker before answering, which hung the whole session whenever
    /// that marker's delivery was delayed (slow NAT traversal).
    #[tokio::test]
    async fn outbound_writes_request_but_inbound_never_reads_it_and_still_responds() {
        let (service, _dir) = setup_service().await;
        let inbound = LibraryBrowseInbound::new(service);

        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (_server_recv, server_send) = tokio::io::split(server_io);

        let mut writer = framed_writer(Box::new(server_send) as Box<dyn AsyncWrite + Send + Unpin>);
        inbound.run(&mut writer).await.unwrap();

        let mut outbound_writer =
            framed_writer(Box::new(client_send) as Box<dyn AsyncWrite + Send + Unpin>);
        let mut outbound_reader =
            framed_reader(Box::new(client_recv) as Box<dyn AsyncRead + Send + Unpin>);
        let outbound = LibraryBrowseOutbound::new(std::sync::Arc::new(|_, _| {}));
        let response = outbound.run(&mut outbound_writer, &mut outbound_reader).await.unwrap();

        assert_eq!(response.comics.len(), 1);
        assert_eq!(response.comics[0].comic_name, "Test");
        assert_eq!(response.comics[0].chapter_count, 1);
    }
}
