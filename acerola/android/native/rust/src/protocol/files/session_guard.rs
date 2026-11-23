use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

/// Prevents two concurrent `acerola/sync-files/1` sessions with the SAME peer —
/// a real log already showed two inbound connections accepted almost simultaneously for the same
/// peer, and without this lock each one triggers its own full library scan via
/// `run_blocking` (see `protocol::ffi_blocking`), doubling the I/O/CPU load of the runtime
/// freeze bug that this lock, together with `run_blocking`, fixes. Shared
/// as the SAME instance between `FileSyncInbound` and `FileSyncOutbound` (see
/// `api.rs::P2PNode::new`) — the lock must hold for both roles at once, not only
/// within each one in isolation, otherwise simultaneous outbound and inbound sessions for the
/// same peer would still slip through.
///
/// `std::sync::Mutex` (not `tokio::sync::Mutex`) is safe here because the lock is never held
/// across an `.await` — it only protects synchronous `insert`/`remove` on the `HashSet`.
#[derive(Default)]
pub(crate) struct FileSyncSessionGuard {
    active_peers: Mutex<HashSet<String>>,
}

impl FileSyncSessionGuard {
    /// Tries to reserve a session for `peer_id`. `None` if there is already an active session for
    /// that peer — the second session is REJECTED, not queued: queuing would require a queue
    /// mechanism with timeout/limit that does not exist anywhere in the protocol today, and the
    /// rejected peer can simply try again later (reconnecting on its side is already
    /// the natural behavior of `acerola-p2p`).
    pub(crate) fn try_acquire(self: &Arc<Self>, peer_id: &str) -> Option<FileSyncSessionLease> {
        let mut active = self
            .active_peers
            .lock()
            .expect("file sync session guard poisoned");
        if !active.insert(peer_id.to_string()) {
            return None;
        }
        Some(FileSyncSessionLease {
            guard: Arc::clone(self),
            peer_id: peer_id.to_string(),
        })
    }
}

/// RAII: releases the `peer_id` reservation automatically when the session ends — success,
/// error (`?` returning early from `run_exchange`) or even panic. Without this, a session that
/// failed midway would leave the peer locked forever until the app restarts.
pub(crate) struct FileSyncSessionLease {
    guard: Arc<FileSyncSessionGuard>,
    peer_id: String,
}

impl Drop for FileSyncSessionLease {
    fn drop(&mut self) {
        self.guard
            .active_peers
            .lock()
            .expect("file sync session guard poisoned")
            .remove(&self.peer_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn second_acquire_for_same_peer_is_rejected_while_first_is_held() {
        let guard = Arc::new(FileSyncSessionGuard::default());
        let lease1 = guard.try_acquire("peer-a");
        assert!(lease1.is_some());
        assert!(guard.try_acquire("peer-a").is_none());
    }

    #[test]
    fn dropping_the_lease_releases_the_peer() {
        let guard = Arc::new(FileSyncSessionGuard::default());
        let lease1 = guard.try_acquire("peer-a").unwrap();
        drop(lease1);
        assert!(guard.try_acquire("peer-a").is_some());
    }

    #[test]
    fn different_peers_can_hold_sessions_simultaneously() {
        let guard = Arc::new(FileSyncSessionGuard::default());
        let _lease_a = guard.try_acquire("peer-a").unwrap();
        assert!(guard.try_acquire("peer-b").is_some());
    }
}
