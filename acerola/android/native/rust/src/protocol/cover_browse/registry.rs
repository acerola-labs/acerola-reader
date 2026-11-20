use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};

/// `P2PNode::browse_cover` (FFI) stores `(comic_name, known_version)` for the peer BEFORE calling
/// `connect()`, and `CoverBrowseOutbound::handle` consumes (`take`) that value as soon as the
/// outbound session starts — same pattern as `protocol::files::PendingComicScope`, but
/// `SyncViewModel::fetchCoversFor` fires ONE `browse_cover` per comic of the remote library,
/// in quick succession without waiting for the previous session to close, for the SAME peer. A single
/// slot per peer (previous version, with `insert`/`remove`, see this file's history) let one
/// call's `insert` overwrite another's before the matching `CoverBrowseOutbound::handle`
/// ran — most sessions found the slot empty and never got to write the
/// `CoverRequest` to the stream, resulting in `browse:cover:error` ("no pending cover scope
/// registered for this peer") silently swallowed by the Android side (see
/// `SyncViewModel.handleEvent`'s `P2pEvent.CoverBrowseError -> Unit`), which is why almost no
/// remote cover showed up. Same bug already seen and fixed on the Desktop side — see
/// `desktop/src-tauri/src/infra/sync/protocol/cover_request_registry.rs`, whose `push`/`take`
/// this file mirrors.
///
/// A per-peer (FIFO) queue instead of a single slot: `push` enqueues, `take` dequeues the oldest.
/// It makes no difference which physical connection carries which request — all that matters is that
/// each `push` has exactly one matching `take`, with no overwrite and no loss.
type PendingCoverRequest = (String, Option<i64>);

#[derive(Default)]
pub(crate) struct PendingCoverRequestRegistry {
    pending: Mutex<HashMap<String, VecDeque<PendingCoverRequest>>>,
}

impl PendingCoverRequestRegistry {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Enqueues a pending request for the peer — never overwrites an already queued request.
    pub(crate) fn push(&self, peer_id: String, comic_name: String, known_version: Option<i64>) {
        self.pending
            .lock()
            .expect("pending cover request registry mutex poisoned")
            .entry(peer_id)
            .or_default()
            .push_back((comic_name, known_version));
    }

    /// Takes the oldest still-pending request for the peer (FIFO). `None` if there is none.
    pub(crate) fn take(&self, peer_id: &str) -> Option<PendingCoverRequest> {
        let mut pending = self
            .pending
            .lock()
            .expect("pending cover request registry mutex poisoned");
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

        assert_eq!(
            registry.take("peer-1"),
            Some(("Berserk".to_string(), Some(42)))
        );
        assert_eq!(registry.take("peer-1"), None);
    }

    #[test]
    fn take_without_a_pending_entry_returns_none() {
        let registry = PendingCoverRequestRegistry::new();
        assert_eq!(registry.take("peer-unknown"), None);
    }

    /// Regression of the bug reported 2026-08-24: `fetchCoversFor` fires N `browse_cover` in
    /// quick succession for the same peer — with the old single slot, the second `insert` erased the
    /// first before any `take` ran, and one of the two sessions always found the registry
    /// empty. With the queue, both requests survive and come out in the order they went in.
    #[test]
    fn concurrent_pushes_for_the_same_peer_do_not_overwrite_each_other() {
        let registry = PendingCoverRequestRegistry::new();
        registry.push("peer-1".to_string(), "Berserk".to_string(), None);
        registry.push("peer-1".to_string(), "Vinland Saga".to_string(), Some(3));

        assert_eq!(registry.take("peer-1"), Some(("Berserk".to_string(), None)));
        assert_eq!(
            registry.take("peer-1"),
            Some(("Vinland Saga".to_string(), Some(3)))
        );
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
