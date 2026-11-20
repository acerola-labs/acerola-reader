//! Bridge between the application protocols (`sync-files`, `sync-comic`, later
//! `browse-cover`) and the node's blob capability. The lib's `Handler` (`acerola-p2p`) only
//! delivers the caller's `PeerIdentity`, without a network address — but `P2pBlobStore::fetch`
//! needs a full `PeerAddr` (it opens its own QUIC connection, independent of the protocol
//! stream itself) and the `Arc<dyn P2pBlobStore>` itself, both only available via `AcerolaP2p`
//! methods (`node.blobs()`, `node.known_peers()`).
//!
//! The node only exists AFTER `AcerolaP2p::builder(...).build()` returns, but the handlers are
//! passed TO the builder before that — `Arc<AcerolaP2p>` cannot be injected directly when
//! building them. `BlobContext` solves this by holding a `Weak<AcerolaP2p>` (`set_node`), right
//! after `.build()` in `bios/network.rs`, before any connection can trigger a handler (none
//! exists until the node is returned) — and again on every `NetworkService::restart()`, since
//! the same handlers (registered only once, on the first boot) stay alive across
//! multiple nodes over the process lifetime.

use std::sync::{Arc, RwLock, Weak};

use acerola_p2p::api::{
    blobs::P2pBlobStore,
    error::P2pError,
    peer::{PeerAddr, PeerIdentity},
    AcerolaP2p,
};

#[derive(Default)]
pub struct BlobContext {
    node: RwLock<Option<Weak<AcerolaP2p>>>,
}

impl BlobContext {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Re-points to the current node. It used to use `OnceLock` (only accepted one successful `set()`
    /// forever) — a P2P module restart (`NetworkService::restart`) calling this again
    /// would silently have no effect, leaving the sync-files/sync-comic handlers
    /// stuck to the OLD (already shut down) node until the process ends. `RwLock<Option<...>>` can
    /// be re-pointed as many times as needed.
    pub fn set_node(&self, node: &Arc<AcerolaP2p>) {
        *self.node.write().unwrap_or_else(|poison| poison.into_inner()) =
            Some(Arc::downgrade(node));
    }

    fn node(&self) -> Result<Arc<AcerolaP2p>, P2pError> {
        self.node
            .read()
            .unwrap_or_else(|poison| poison.into_inner())
            .as_ref()
            .and_then(Weak::upgrade)
            .ok_or_else(|| P2pError::StreamFailed("p2p node not initialized yet".into()))
    }

    pub async fn blob_store(&self) -> Result<Arc<dyn P2pBlobStore>, P2pError> {
        self.node()?
            .blobs()
            .await
            .ok_or_else(|| P2pError::StreamFailed("blob store not configured on this node".into()))
    }

    /// Resolves the most recent known `PeerAddr` for `peer` — cache fed by the
    /// handshake (`acerola/handshake/1`) and by explicit calls (`connect`/`sync_comic`).
    pub async fn resolve_addr(&self, peer: &PeerIdentity) -> Result<PeerAddr, P2pError> {
        self.node()?
            .known_peers()
            .await
            .into_iter()
            .find(|(id, _, _)| id == peer)
            .map(|(_, addr, _)| addr)
            .ok_or_else(|| P2pError::StreamFailed(format!("no known address for peer {}", peer.id)))
    }

    /// RTT measured with `peer` right now, only used to size blob fetch parallelism
    /// (see `receive_files`/`fetch_covers_for`) — never critical enough to become `Err`.
    /// `None` both without a live connection and without a node (in that case the caller will fail
    /// right after in `resolve_addr`/`blob_store`, so it is not worth duplicating the error here).
    pub async fn latency(&self, peer: &PeerIdentity) -> Option<std::time::Duration> {
        self.node().ok()?.latency(peer).await
    }
}
