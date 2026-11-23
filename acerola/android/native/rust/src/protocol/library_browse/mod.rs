mod exchange;
mod model;

use std::sync::Arc;

use acerola_p2p::api::{
    error::P2pError,
    peer::PeerIdentity,
    protocol::{EventEmitter, Handler},
};
use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{callbacks::FileSyncProvider, protocol::sync_error::classify_sync_error};
use model::LibrarySummary;

/// Lists only the comic names of a peer (with chapter count) — without transferring
/// anything. No session/guard: it is a single short read, with no risk of duplicating heavy
/// I/O work the way `acerola/sync-files/1`/`acerola/sync-comic/1` do (see
/// `protocol::files::FileSyncSessionGuard`).
pub(crate) const LIBRARY_BROWSE_ALPN: &[u8] = b"acerola/browse-library/1";

/// Inbound role: replies with the local library summary as soon as the connection is accepted — it
/// does not wait for nor read the marker the outbound writes (see docs in `exchange::run_inbound`), so
/// the reply is not coupled to how long the P2P path takes to be ready for real data.
pub(crate) struct LibraryBrowseInbound {
    provider: Arc<dyn FileSyncProvider>,
}

impl LibraryBrowseInbound {
    pub(crate) fn new(provider: Arc<dyn FileSyncProvider>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl Handler for LibraryBrowseInbound {
    async fn handle(
        &self,
        _peer: &PeerIdentity,
        send: Box<dyn AsyncWrite + Send + Unpin>,
        _recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        exchange::run_inbound(&self.provider, send).await
    }
}

/// Outbound role: writes the request marker, reads the reply and emits the result as an event
/// to Kotlin — whoever triggered `P2PNode::browse_library` has no synchronous return (same
/// fire-and-forget pattern as `connect`), so the result only arrives via
/// `browse:library:result`/`browse:library:error`.
pub(crate) struct LibraryBrowseOutbound {
    emit: EventEmitter,
}

impl LibraryBrowseOutbound {
    pub(crate) fn new(emit: EventEmitter) -> Self {
        Self { emit }
    }
}

#[async_trait]
impl Handler for LibraryBrowseOutbound {
    async fn handle(
        &self,
        peer: &PeerIdentity,
        send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        match exchange::run_outbound(send, recv).await {
            Ok(summary) => {
                (self.emit)("browse:library:result", result_payload(peer, &summary));
                Ok(())
            }
            Err(err) => {
                let code = classify_sync_error(&err);
                tracing::warn!(peer = %peer.id, ?code, error = %err, "[LibraryBrowse] query failed");
                (self.emit)(
                    "browse:library:error",
                    serde_json::json!({ "peerId": peer.id, "message": err.to_string(), "code": code }).to_string(),
                );
                Err(err)
            }
        }
    }
}

fn result_payload(peer: &PeerIdentity, summary: &LibrarySummary) -> String {
    let comics: Vec<serde_json::Value> = summary
        .comics
        .iter()
        .map(|entry| {
            serde_json::json!({
                "comicName": entry.comic_name,
                "chapterCount": entry.chapter_count,
                "coverVersion": entry.cover_version,
            })
        })
        .collect();

    serde_json::json!({ "peerId": peer.id, "comics": comics }).to_string()
}
