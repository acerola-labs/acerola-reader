use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

/// The Tauri command `query_remote_cover` writes `(comic_name, known_version)` for the peer BEFORE
/// calling `connect()`, and `CoverBrowseOutbound::handle` consumes (`take`) that value as soon as
/// the outbound session starts — same idea as `comic_sync_registry.rs::PendingComicSyncRegistry`,
/// but with one important difference: `use-remote-library.svelte.ts::fetchCoversFor` fires ONE
/// `query_remote_cover` per comic of the remote library, all in parallel (`Promise.all`),
/// for the SAME peer. A single slot per peer (previous version, with `insert`/`remove`) let
/// one call's `push` overwrite another's before the matching `CoverBrowseOutbound::handle`
/// ran — most sessions found the slot empty and never got to write
/// the `CoverRequest` to the stream, resulting in `StreamFailed("no pending cover request registered
/// for this peer")` on Desktop and `StreamFailed("stream closed before cover message")` on the
/// accepting side (Android), with only the last one surviving. Reported/reproduced live on 2026-08-22.
///
/// Per-peer queue (FIFO) instead of a single slot: `push` enqueues, `take` dequeues the oldest.
/// It does not matter which physical connection carries which request — it only matters that each
/// `push` has exactly one matching `take`, with no overwrite and no loss. `CoverBrowseOutbound` is a
/// singleton with no other way to know `comic_name` per `connect()` call, so it cannot be
/// indexed by `(peer_id, comic_name)` on the `take` side — whoever calls `take` only has `peer_id`.
///
/// Only the outbound side needs this — the inbound side receives everything through the
/// `CoverRequest` itself on the wire.
type PendingCoverRequest = (String, Option<i64>);

#[derive(Default)]
pub struct PendingCoverRequestRegistry {
    pending: Mutex<HashMap<String, VecDeque<PendingCoverRequest>>>,
}

impl PendingCoverRequestRegistry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Enqueues a pending request for the peer — never overwrites an already queued request.
    pub fn push(&self, peer_id: String, comic_name: String, known_version: Option<i64>) {
        self.pending
            .lock()
            .expect("pending cover request registry mutex poisoned")
            .entry(peer_id)
            .or_default()
            .push_back((comic_name, known_version));
    }

    /// Takes the oldest still-pending request for the peer (FIFO). `None` if there is none.
    pub fn take(&self, peer_id: &str) -> Option<PendingCoverRequest> {
        let mut pending =
            self.pending.lock().expect("pending cover request registry mutex poisoned");
        let queue = pending.get_mut(peer_id)?;
        let next = queue.pop_front();
        if queue.is_empty() {
            pending.remove(peer_id);
        }
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_removes_the_pending_entry() {
        let registry = PendingCoverRequestRegistry::new();
        registry.push("peer-1".to_string(), "Berserk".to_string(), Some(42));

        assert_eq!(registry.take("peer-1"), Some(("Berserk".to_string(), Some(42))));
        assert_eq!(registry.take("peer-1"), None);
    }

    #[test]
    fn take_without_a_pending_entry_returns_none() {
        let registry = PendingCoverRequestRegistry::new();
        assert_eq!(registry.take("peer-unknown"), None);
    }

    /// Regression for the bug reported on 2026-08-22: `fetchCoversFor` fires N `query_remote_cover`
    /// in parallel for the same peer — with the old single slot, the second `push` (then `set`)
    /// wiped the first before any `take` ran, and one of the two sessions always found
    /// the registry empty. With the queue, both requests survive and come out in the order they went in.
    #[test]
    fn concurrent_pushes_for_the_same_peer_do_not_overwrite_each_other() {
        let registry = PendingCoverRequestRegistry::new();
        registry.push("peer-1".to_string(), "Berserk".to_string(), None);
        registry.push("peer-1".to_string(), "Vinland Saga".to_string(), Some(3));

        assert_eq!(registry.take("peer-1"), Some(("Berserk".to_string(), None)));
        assert_eq!(registry.take("peer-1"), Some(("Vinland Saga".to_string(), Some(3))));
        assert_eq!(registry.take("peer-1"), None);
    }

    /// Different peers have independent queues — pending requests for peer A do not leak into
    /// peer B's `take` nor are consumed by it.
    #[test]
    fn different_peers_have_independent_queues() {
        let registry = PendingCoverRequestRegistry::new();
        registry.push("peer-a".to_string(), "A".to_string(), None);
        registry.push("peer-b".to_string(), "B".to_string(), None);

        assert_eq!(registry.take("peer-b"), Some(("B".to_string(), None)));
        assert_eq!(registry.take("peer-a"), Some(("A".to_string(), None)));
    }
}
