//! Bridge between the application protocols (`sync-files`, `sync-comic`, eventually
//! `browse-cover`) and the node's blob capability. The lib's `Handler` (`acerola-p2p`) only
//! hands over the caller's `PeerIdentity`, with no network address — but `P2pBlobStore::fetch`
//! needs a full `PeerAddr` (it opens its own QUIC connection, independent of the protocol's
//! stream itself) and the `Arc<dyn P2pBlobStore>` itself, both only available via
//! `AcerolaP2p` methods (`node.blobs()`, `node.known_peers()`).
//!
//! The node only exists AFTER `AcerolaP2p::builder(...).build()` returns, but the handlers are
//! passed TO the builder before that — `Arc<AcerolaP2p>` cannot be injected directly when
//! building them. `BlobContext` solves this by holding a `Weak<AcerolaP2p>` (`set_node`), filled
//! right after each `.build()` in `api.rs` — both at boot and on every `P2PNode::restart`
//! — before any connection can trigger a handler. `RwLock` (not `OnceLock`) because a
//! restart calls `set_node` again: with `OnceLock` the second call would be a silent no-op
//! and the handlers would be stuck forever on the old, already shut-down node.

use std::sync::{Arc, RwLock, Weak};

use acerola_p2p::api::{
    blobs::P2pBlobStore,
    error::P2pError,
    peer::{PeerAddr, PeerIdentity},
    AcerolaP2p,
};

#[derive(Default)]
pub(crate) struct BlobContext {
    node: RwLock<Option<Weak<AcerolaP2p>>>,
}

impl BlobContext {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub(crate) fn set_node(&self, node: &Arc<AcerolaP2p>) {
        *self
            .node
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Arc::downgrade(node));
    }

    fn node(&self) -> Result<Arc<AcerolaP2p>, P2pError> {
        self.node
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .and_then(Weak::upgrade)
            .ok_or_else(|| P2pError::StreamFailed("p2p node not initialized yet".into()))
    }

    pub(crate) async fn blob_store(&self) -> Result<Arc<dyn P2pBlobStore>, P2pError> {
        self.node()?
            .blobs()
            .await
            .ok_or_else(|| P2pError::StreamFailed("blob store not configured on this node".into()))
    }

    /// Resolves the most recent known `PeerAddr` for `peer` — cache fed by the
    /// handshake (`acerola/handshake/1`) and by explicit calls (`connect`/`sync_comic`).
    pub(crate) async fn resolve_addr(&self, peer: &PeerIdentity) -> Result<PeerAddr, P2pError> {
        self.node()?
            .known_peers()
            .await
            .into_iter()
            .find(|(id, _, _)| id == peer)
            .map(|(_, addr, _)| addr)
            .ok_or_else(|| P2pError::StreamFailed(format!("no known address for peer {}", peer.id)))
    }

    /// RTT measured with `peer` now, used only to size blob fetch parallelism (see
    /// `exchange.rs::receive_files`) — never critical enough to become an `Err`. `None` both
    /// without a live connection and without a node (in that case the caller will fail right after in
    /// `resolve_addr`/`blob_store`, so it is not worth duplicating the error here). Mirrors
    /// Desktop's `BlobContext::latency` (`infra/sync/blob_context.rs`) — same concurrency
    /// contract, different platforms.
    pub(crate) async fn latency(&self, peer: &PeerIdentity) -> Option<std::time::Duration> {
        self.node().ok()?.latency(peer).await
    }
}
