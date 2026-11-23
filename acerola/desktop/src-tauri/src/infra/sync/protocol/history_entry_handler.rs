use acerola_p2p::api::{
    error::P2pError,
    peer::PeerIdentity,
    protocol::{EventEmitter, Handler},
};
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite};

use super::history_entry_registry::PendingHistoryEntryRegistry;
use crate::{
    core::services::sync::history_sync::{ApplyManifestStats, HistorySyncService},
    infra::sync::{
        framing::{
            framed_reader, framed_writer, read_json, write_json, FramedReader, FramedWriter,
        },
        messages::{HistoryEntryAck, HistoryEntryRequest, HistoryManifest},
        protocol::transfer::{
            classify_sync_error, sync_error_payload, NO_PENDING_SCOPE_REASON,
            PEER_COMIC_NOT_FOUND_REASON,
        },
    },
};

const LOG_KIND: &str = "history-entry";

/// ONE-WAY push of a `HistoryManifest` restricted to the selected chapter(s) of ONE
/// comic — exists so a full `acerola/sync-history/1` session (which exchanges the
/// whole library in both directions) is not needed just to carry the progress/"read" marker
/// of the chapter(s) the user chose to send.
///
/// Two-message contract (standardized with `ComicSyncOutbound`): first a
/// `HistoryEntryRequest` (comic + scoped chapters, always present even if the manifest
/// that follows is empty), then the `HistoryManifest`. The answer is a `HistoryEntryAck`
/// with the real result (not an empty `{}`) — if `comic_known` comes back `false`, the peer did
/// not have that comic and nothing was applied; the outbound treats this as an error, not silent success.
///
/// The scope (comic + `chapter_ids`) comes from `PendingHistoryEntryRegistry` (same technique as
/// `ComicSyncOutbound`/`PendingComicSyncRegistry`), written by the Tauri command
/// `sync_history_entry` before calling `connect()`.
pub struct HistoryEntrySyncOutbound {
    emit: EventEmitter,
    service: HistorySyncService,
    registry: std::sync::Arc<PendingHistoryEntryRegistry>,
}

impl HistoryEntrySyncOutbound {
    pub fn new(
        emit: EventEmitter, service: HistorySyncService,
        registry: std::sync::Arc<PendingHistoryEntryRegistry>,
    ) -> Self {
        Self { emit, service, registry }
    }

    async fn run(
        &self, peer_id: &str, writer: &mut FramedWriter, reader: &mut FramedReader,
    ) -> Result<(), P2pError> {
        let scope = self
            .registry
            .take(peer_id)
            .ok_or_else(|| P2pError::StreamFailed(NO_PENDING_SCOPE_REASON.into()))?;

        let chapter_sorts =
            self.service.resolve_chapter_sorts(&scope.comic_name, &scope.chapter_ids).await?;
        write_json(
            writer,
            &HistoryEntryRequest {
                comic_name: scope.comic_name.clone(),
                chapter_sorts: chapter_sorts.clone(),
            },
        )
        .await?;

        let manifest =
            self.service.build_manifest_for_chapters(&scope.comic_name, &scope.chapter_ids).await?;
        write_json(writer, &manifest).await?;

        let ack: HistoryEntryAck = read_json(reader).await?;
        if !ack.comic_known {
            return Err(P2pError::StreamFailed(PEER_COMIC_NOT_FOUND_REASON.into()));
        }

        tracing::info!(
            peer = %peer_id,
            comic = %scope.comic_name,
            chapters = chapter_sorts.len(),
            entries_applied = ack.entries_applied,
            markers_applied = ack.markers_applied,
            "[HistoryEntrySync] push applied by peer",
        );

        Ok(())
    }
}

#[async_trait]
impl Handler for HistoryEntrySyncOutbound {
    async fn handle(
        &self, peer: &PeerIdentity, send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        let mut writer = framed_writer(send);
        let mut reader = framed_reader(recv);

        tracing::debug!(peer = %peer.id, "[HistoryEntrySync] outbound session started");
        (self.emit)("sync:history-entry:started", peer.id.clone());

        match self.run(&peer.id, &mut writer, &mut reader).await {
            Ok(()) => {
                (self.emit)("sync:history-entry:complete", peer.id.clone());
                Ok(())
            },
            Err(error) => {
                let message = error.to_string();
                let code = classify_sync_error(&error);
                tracing::warn!(peer = %peer.id, ?code, error = %message, "[HistoryEntrySync] session failed");
                (self.emit)(
                    "sync:history-entry:error",
                    sync_error_payload(&peer.id, &message, code, None),
                );
                Err(error)
            },
        }
    }
}

/// Side that ANSWERS the push — reads the `HistoryEntryRequest` (declares the scope even if the
/// manifest comes empty), checks that the comic exists locally BEFORE applying anything,
/// applies the manifest only if it exists, and confirms with a `HistoryEntryAck` carrying the
/// real result — never an empty ack that would hide a missing comic as if it were
/// success.
pub struct HistoryEntrySyncInbound {
    emit: EventEmitter,
    service: HistorySyncService,
}

impl HistoryEntrySyncInbound {
    pub fn new(emit: EventEmitter, service: HistorySyncService) -> Self {
        Self { emit, service }
    }

    async fn run(
        &self, writer: &mut FramedWriter, reader: &mut FramedReader,
    ) -> Result<(), P2pError> {
        let request: HistoryEntryRequest = read_json(reader).await?;
        let manifest: HistoryManifest = read_json(reader).await?;

        let comic_known = self.service.comic_exists(&request.comic_name).await?;
        let stats = if comic_known {
            self.service.apply_manifest(&manifest).await?
        } else {
            ApplyManifestStats::default()
        };

        write_json(
            writer,
            &HistoryEntryAck {
                comic_known,
                entries_applied: stats.entries_applied,
                markers_applied: stats.markers_applied,
            },
        )
        .await?;

        tracing::info!(
            comic = %request.comic_name,
            chapters = request.chapter_sorts.len(),
            comic_known,
            entries_applied = stats.entries_applied,
            markers_applied = stats.markers_applied,
            "[HistoryEntrySync] request processed",
        );

        Ok(())
    }
}

#[async_trait]
impl Handler for HistoryEntrySyncInbound {
    async fn handle(
        &self, peer: &PeerIdentity, send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        let mut writer = framed_writer(send);
        let mut reader = framed_reader(recv);

        tracing::debug!(peer = %peer.id, "[HistoryEntrySync] inbound session started");
        (self.emit)("sync:history-entry:started", peer.id.clone());

        match self.run(&mut writer, &mut reader).await {
            Ok(()) => {
                (self.emit)("sync:history-entry:complete", peer.id.clone());
                Ok(())
            },
            Err(error) => {
                let message = error.to_string();
                let code = classify_sync_error(&error);
                tracing::warn!(peer = %peer.id, ?code, error = %message, "[HistoryEntrySync] session failed");
                (self.emit)(
                    "sync:history-entry:error",
                    sync_error_payload(&peer.id, &message, code, None),
                );
                Err(error)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use acerola_p2p::api::peer::PeerIdentity;

    use super::{super::history_entry_registry::HistoryEntryScope, *};
    use crate::{
        data::repositories::history::reading_history_repo::ReadingHistoryRepository,
        tests::utils::setup_test_db::setup_test_db_with_comic,
    };

    async fn setup() -> (sqlx::SqlitePool, HistorySyncService) {
        let pool = setup_test_db_with_comic().await;
        sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified) VALUES (1, 'Cap 1', 'p', '1', 0, 1, 0)")
            .execute(&pool)
            .await
            .unwrap();
        (pool.clone(), HistorySyncService::new(pool))
    }

    fn noop_emitter() -> EventEmitter {
        Arc::new(|_, _| {})
    }

    fn run_pair(
        outbound: HistoryEntrySyncOutbound, inbound: HistoryEntrySyncInbound, peer: PeerIdentity,
    ) -> (
        impl std::future::Future<Output = Result<(), P2pError>>,
        impl std::future::Future<Output = Result<(), P2pError>>,
    ) {
        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);

        let outbound_peer = peer.clone();
        let inbound_peer = peer;
        (
            async move {
                outbound.handle(&outbound_peer, Box::new(client_send), Box::new(client_recv)).await
            },
            async move {
                inbound.handle(&inbound_peer, Box::new(server_send), Box::new(server_recv)).await
            },
        )
    }

    #[tokio::test]
    async fn outbound_pushes_only_the_scoped_comic_and_inbound_applies_it() {
        let (outbound_pool, outbound_service) = setup().await;
        let (_inbound_pool, inbound_service) = setup().await;

        ReadingHistoryRepository::new(outbound_pool)
            .upsert(&crate::data::models::history::reading_history::ReadingHistory {
                comic_directory_id: 1,
                chapter_archive_id: 1,
                last_page: 7,
                is_completed: false,
                updated_at: 5000,
            })
            .await
            .unwrap();

        let registry = PendingHistoryEntryRegistry::new();
        let peer = PeerIdentity { id: "peer-a".to_string(), device_id: None };
        registry.set(
            peer.id.clone(),
            HistoryEntryScope { comic_name: "Test".to_string(), chapter_ids: vec![1] },
        );

        let outbound = HistoryEntrySyncOutbound::new(noop_emitter(), outbound_service, registry);
        let inbound = HistoryEntrySyncInbound::new(noop_emitter(), inbound_service.clone());

        let (outbound_fut, inbound_fut) = run_pair(outbound, inbound, peer);
        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        outbound_result.expect("outbound should complete without error");
        inbound_result.expect("inbound should complete without error");

        let applied = inbound_service.build_manifest().await.unwrap();
        assert_eq!(applied.entries.len(), 1);
        assert_eq!(applied.entries[0].comic_name, "Test");
        assert_eq!(applied.entries[0].last_page, 7);
    }

    #[tokio::test]
    async fn outbound_only_sends_read_markers_for_the_selected_chapters() {
        use crate::data::repositories::history::chapter_read_repo::ChapterReadRepository;

        let (outbound_pool, outbound_service) = setup().await;
        let (inbound_pool, inbound_service) = setup().await;

        for pool in [&outbound_pool, &inbound_pool] {
            sqlx::query("INSERT INTO chapter_archive (id, chapter, path, chapter_sort, is_special, comic_directory_fk, last_modified) VALUES (2, 'Cap 2', 'p', '2', 0, 1, 0)")
                .execute(pool)
                .await
                .unwrap();
        }
        ChapterReadRepository::new(outbound_pool.clone())
            .insert_batch(1, &[1, 2], 900)
            .await
            .unwrap();

        let registry = PendingHistoryEntryRegistry::new();
        let peer = PeerIdentity { id: "peer-b".to_string(), device_id: None };
        registry.set(
            peer.id.clone(),
            HistoryEntryScope { comic_name: "Test".to_string(), chapter_ids: vec![2] },
        );

        let outbound = HistoryEntrySyncOutbound::new(noop_emitter(), outbound_service, registry);
        let inbound = HistoryEntrySyncInbound::new(noop_emitter(), inbound_service.clone());

        let (outbound_fut, inbound_fut) = run_pair(outbound, inbound, peer);
        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        outbound_result.expect("outbound should complete without error");
        inbound_result.expect("inbound should complete without error");

        let applied = inbound_service.build_manifest().await.unwrap();
        assert_eq!(applied.read_markers.len(), 1);
        assert_eq!(applied.read_markers[0].chapter, "2");
    }

    /// Core of the new contract: if the peer does not have the comic, the `HistoryEntryAck` comes
    /// back with `comic_known: false` and the outbound ENDS WITH AN ERROR — no longer the false
    /// positive of "sent successfully" when actually nothing was applied on the other side.
    #[tokio::test]
    async fn outbound_fails_when_peer_does_not_have_the_comic() {
        let (outbound_pool, outbound_service) = setup().await;
        // "Empty" inbound: database without the "Test" comic (no `setup_test_db_with_comic`).
        let inbound_pool = crate::tests::utils::setup_test_db::setup_test_db().await;
        let inbound_service = HistorySyncService::new(inbound_pool);

        ReadingHistoryRepository::new(outbound_pool)
            .upsert(&crate::data::models::history::reading_history::ReadingHistory {
                comic_directory_id: 1,
                chapter_archive_id: 1,
                last_page: 7,
                is_completed: false,
                updated_at: 5000,
            })
            .await
            .unwrap();

        let registry = PendingHistoryEntryRegistry::new();
        let peer = PeerIdentity { id: "peer-c".to_string(), device_id: None };
        registry.set(
            peer.id.clone(),
            HistoryEntryScope { comic_name: "Test".to_string(), chapter_ids: vec![1] },
        );

        let outbound = HistoryEntrySyncOutbound::new(noop_emitter(), outbound_service, registry);
        let inbound = HistoryEntrySyncInbound::new(noop_emitter(), inbound_service);

        let (outbound_fut, inbound_fut) = run_pair(outbound, inbound, peer);
        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);

        assert!(outbound_result.is_err());
        inbound_result
            .expect("inbound must still complete the session (the error is on the outbound)");
    }

    #[tokio::test]
    async fn outbound_fails_fast_when_no_scope_was_registered_for_the_peer() {
        let (_, service) = setup().await;
        let registry = PendingHistoryEntryRegistry::new();
        let outbound = HistoryEntrySyncOutbound::new(noop_emitter(), service, registry);
        let peer = PeerIdentity { id: "peer-without-scope".to_string(), device_id: None };

        let (client_io, _server_io) = tokio::io::duplex(1024);
        let (client_recv, client_send) = tokio::io::split(client_io);

        let result = outbound.handle(&peer, Box::new(client_send), Box::new(client_recv)).await;
        assert!(result.is_err());
    }
}
