mod exchange;
mod model;

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

use crate::{callbacks::HistorySyncProvider, protocol::sync_error::classify_sync_error};

pub(crate) const HISTORY_SYNC_ALPN: &[u8] = b"acerola/sync-history/1";
/// Individual push of ONE history entry, without exchanging the whole library manifest —
/// see `exchange::run_entry_exchange`. Complements `HISTORY_SYNC_ALPN`, does not replace it.
pub(crate) const HISTORY_ENTRY_SYNC_ALPN: &[u8] = b"acerola/sync-history-entry/1";

/// `peer_id` -> `(comic_name, chapter_sorts)` pending between the `sync_history_entry` FFI call
/// and `HistoryEntrySyncOutbound::handle` — same technique as `protocol::files::PendingComicScope`.
pub(crate) type PendingHistoryEntryScope = Arc<Mutex<HashMap<String, (String, Vec<String>)>>>;

/// Outbound role of the `acerola/sync-history/1` protocol — used when this side is the one that
/// started the connection via `AcerolaP2p::connect`. Writes its manifest first (rule 4).
pub(crate) struct HistorySyncOutbound {
    emit: EventEmitter,
    provider: Arc<dyn HistorySyncProvider>,
}

impl HistorySyncOutbound {
    pub(crate) fn new(emit: EventEmitter, provider: Arc<dyn HistorySyncProvider>) -> Self {
        Self { emit, provider }
    }
}

#[async_trait]
impl Handler for HistorySyncOutbound {
    async fn handle(
        &self,
        peer: &PeerIdentity,
        send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        exchange::run_exchange(true, peer, &self.emit, &self.provider, send, recv)
            .await
            .inspect_err(|err| (self.emit)("sync:history:error", error_payload(peer, err)))
    }
}

/// Inbound role of the `acerola/sync-history/1` protocol — used when the peer is the one that
/// started the connection. Reads the peer's manifest first (rule 4).
pub(crate) struct HistorySyncInbound {
    emit: EventEmitter,
    provider: Arc<dyn HistorySyncProvider>,
}

impl HistorySyncInbound {
    pub(crate) fn new(emit: EventEmitter, provider: Arc<dyn HistorySyncProvider>) -> Self {
        Self { emit, provider }
    }
}

#[async_trait]
impl Handler for HistorySyncInbound {
    async fn handle(
        &self,
        peer: &PeerIdentity,
        send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        exchange::run_exchange(false, peer, &self.emit, &self.provider, send, recv)
            .await
            .inspect_err(|err| (self.emit)("sync:history:error", error_payload(peer, err)))
    }
}

fn error_payload(peer: &PeerIdentity, err: &P2pError) -> String {
    let code = classify_sync_error(err);
    tracing::warn!(peer = %peer.id, ?code, error = %err, "[HistorySync] session failed");
    serde_json::json!({ "peerId": peer.id, "message": err.to_string(), "code": code }).to_string()
}

/// Outbound role of the `acerola/sync-history-entry/1` protocol — `comic_name` comes from
/// `PendingHistoryEntryScope`, stored by the `sync_history_entry` FFI call before
/// `connect()` (same technique as `ComicSyncOutbound`/`PendingComicScope`).
pub(crate) struct HistoryEntrySyncOutbound {
    emit: EventEmitter,
    provider: Arc<dyn HistorySyncProvider>,
    pending_scope: PendingHistoryEntryScope,
}

impl HistoryEntrySyncOutbound {
    pub(crate) fn new(
        emit: EventEmitter,
        provider: Arc<dyn HistorySyncProvider>,
        pending_scope: PendingHistoryEntryScope,
    ) -> Self {
        Self {
            emit,
            provider,
            pending_scope,
        }
    }
}

#[async_trait]
impl Handler for HistoryEntrySyncOutbound {
    async fn handle(
        &self,
        peer: &PeerIdentity,
        send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        let scope = self
            .pending_scope
            .lock()
            .expect("pending history entry scope mutex poisoned")
            .remove(&peer.id);

        exchange::run_entry_exchange(true, scope, peer, &self.emit, &self.provider, send, recv)
            .await
            .inspect_err(|err| (self.emit)("sync:history-entry:error", error_payload(peer, err)))
    }
}

/// Inbound role of the `acerola/sync-history-entry/1` protocol — `comic_name` already comes inside
/// the received entry itself, so no prior scope is needed.
pub(crate) struct HistoryEntrySyncInbound {
    emit: EventEmitter,
    provider: Arc<dyn HistorySyncProvider>,
}

impl HistoryEntrySyncInbound {
    pub(crate) fn new(emit: EventEmitter, provider: Arc<dyn HistorySyncProvider>) -> Self {
        Self { emit, provider }
    }
}

#[async_trait]
impl Handler for HistoryEntrySyncInbound {
    async fn handle(
        &self,
        peer: &PeerIdentity,
        send: Box<dyn AsyncWrite + Send + Unpin>,
        recv: Box<dyn AsyncRead + Send + Unpin>,
    ) -> Result<(), P2pError> {
        exchange::run_entry_exchange(false, None, peer, &self.emit, &self.provider, send, recv)
            .await
            .inspect_err(|err| (self.emit)("sync:history-entry:error", error_payload(peer, err)))
    }
}
