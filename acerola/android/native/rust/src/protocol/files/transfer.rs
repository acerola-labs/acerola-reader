//! Abstraction for transferring a chapter's bytes between peers, used by phase 3 of
//! `exchange.rs` (`publish_chapter_to_blob`/`receive_chapter_from_blob`). Exists as a trait
//! (instead of a direct `Arc<BlobContext>`) only to keep the `exchange.rs` tests fast and free of
//! a real network — `InMemoryChapterTransfer` simulates the publish/fetch pair with a `HashMap`
//! shared between both sides of the test, same spirit as the `InMemoryFileSyncProvider`
//! already used there. In production, `BlobChapterTransfer` is the only implementation used.

use std::sync::Arc;

use acerola_p2p::api::{error::P2pError, peer::PeerIdentity};
use async_trait::async_trait;
use tokio::io::AsyncRead;

use crate::protocol::blob_context::BlobContext;

#[async_trait]
pub(crate) trait ChapterTransfer: Send + Sync {
    /// Publishes the bytes locally (`P2pBlobStore::put`) and returns the blob hash (hex) —
    /// it goes into `FileHeader.blob_hash`, which the other side uses to fetch it back.
    async fn publish(&self, bytes: Vec<u8>) -> Result<String, P2pError>;

    /// Fetches the bytes referenced by `blob_hash` from the given `peer` (`P2pBlobStore::fetch`
    /// followed by `get`) and returns a reader — the caller decides how to consume it (chunk by
    /// chunk, for incremental progress via `FileSyncProvider::write_chapter_chunk`).
    async fn fetch_reader(
        &self,
        blob_hash: &str,
        peer: &PeerIdentity,
    ) -> Result<Box<dyn AsyncRead + Send + Unpin>, P2pError>;

    /// RTT measured with `peer` now, used only to size how many chapters to fetch in
    /// parallel in `exchange.rs::receive_files` (see docs there) — `None` when it cannot be measured
    /// (no live connection, or a test implementation that does not simulate a real network) falls
    /// back to minimum parallelism, never to an error. Mirrors Desktop's `ChapterTransfer::latency`
    /// (`infra/sync/protocol/transfer.rs`) — same contract, different platforms.
    async fn latency(&self, peer: &PeerIdentity) -> Option<std::time::Duration>;
}

pub(crate) struct BlobChapterTransfer {
    context: Arc<BlobContext>,
}

impl BlobChapterTransfer {
    pub(crate) fn new(context: Arc<BlobContext>) -> Self {
        Self { context }
    }
}

#[async_trait]
impl ChapterTransfer for BlobChapterTransfer {
    async fn publish(&self, bytes: Vec<u8>) -> Result<String, P2pError> {
        let store = self.context.blob_store().await?;
        // `P2pBlobStore::put` already returns `ConnectionError` (= `P2pError`, same type — see
        // `acerola_p2p::api::error`), already correctly classified at the source
        // (`classify_connection_error`/`classify_get_error` in `lib/p2p`). Propagate it directly
        // instead of flattening into `StreamFailed(err.to_string())` — that threw away the
        // classification and forced `classify_sync_error` (`mod.rs`) to guess the cause via
        // substring matching on the text, fragile and with no guarantee of covering real cases.
        let hash = store.put(bytes).await?;
        Ok(hash.to_string())
    }

    async fn fetch_reader(
        &self,
        blob_hash: &str,
        peer: &PeerIdentity,
    ) -> Result<Box<dyn AsyncRead + Send + Unpin>, P2pError> {
        let store = self.context.blob_store().await?;
        // The hash format is local/static (does not come from the network) — no counterpart in
        // `ConnectionError` beyond the generic `StreamFailed`, so `.map_err` is kept only here.
        let hash = blob_hash
            .parse()
            .map_err(|_| P2pError::StreamFailed(format!("invalid blob hash: {blob_hash}")))?;
        let addr = self.context.resolve_addr(peer).await?;

        // See the comment in `publish` — propagates the structured `ConnectionError` directly.
        store.fetch(&hash, &addr).await?;
        store.get(&hash).await
    }

    async fn latency(&self, peer: &PeerIdentity) -> Option<std::time::Duration> {
        self.context.latency(peer).await
    }
}

#[cfg(test)]
pub(crate) struct InMemoryChapterTransfer {
    blobs: std::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>,
    /// `None` by default (same meaning as "no real connection to measure" in the trait docs) —
    /// tests exercising latency-based parallelism sizing use `set_latency`
    /// to simulate a specific RTT without needing a real network.
    latency: std::sync::Mutex<Option<std::time::Duration>>,
}

#[cfg(test)]
impl InMemoryChapterTransfer {
    /// Creates a pair of handles sharing the same internal `HashMap` — simulates two distinct
    /// sides publishing/fetching from the SAME "network store" (one side's hash is already
    /// available to the other, exactly as real content-addressed content would be).
    pub(crate) fn shared_pair() -> (Arc<dyn ChapterTransfer>, Arc<dyn ChapterTransfer>) {
        let shared = Arc::new(InMemoryChapterTransfer {
            blobs: std::sync::Mutex::new(std::collections::HashMap::new()),
            latency: std::sync::Mutex::new(None),
        });
        (
            Arc::clone(&shared) as Arc<dyn ChapterTransfer>,
            shared as Arc<dyn ChapterTransfer>,
        )
    }

    pub(crate) fn set_latency(&self, latency: std::time::Duration) {
        *self.latency.lock().unwrap() = Some(latency);
    }
}

#[cfg(test)]
#[async_trait]
impl ChapterTransfer for InMemoryChapterTransfer {
    async fn publish(&self, bytes: Vec<u8>) -> Result<String, P2pError> {
        let hash = blake3::hash(&bytes).to_hex().to_string();
        self.blobs.lock().unwrap().insert(hash.clone(), bytes);
        Ok(hash)
    }

    async fn fetch_reader(
        &self,
        blob_hash: &str,
        _peer: &PeerIdentity,
    ) -> Result<Box<dyn AsyncRead + Send + Unpin>, P2pError> {
        let bytes = self
            .blobs
            .lock()
            .unwrap()
            .get(blob_hash)
            .cloned()
            .ok_or_else(|| {
                P2pError::StreamFailed(format!("unknown blob hash in test transfer: {blob_hash}"))
            })?;

        // `tokio::io::duplex` already implements `AsyncRead`/`AsyncWrite` — simpler than
        // implementing a manual `poll_read` just for tests. Writes everything and closes (EOF) before
        // returning the read end; the buffer is sized to the payload's exact size
        // so the write does not block.
        use tokio::io::AsyncWriteExt;
        let (mut writer, reader) = tokio::io::duplex(bytes.len().max(1));
        tokio::spawn(async move {
            let _ = writer.write_all(&bytes).await;
            let _ = writer.shutdown().await;
        });
        Ok(Box::new(reader))
    }

    async fn latency(&self, _peer: &PeerIdentity) -> Option<std::time::Duration> {
        *self.latency.lock().unwrap()
    }
}
