use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

/// Prevents two `acerola/sync-files/1` sessions from competing for the same peer — the same
/// race the Android side already observes in production (user retry, automatic reconnection,
/// or the transport delivering two physical connections for the same peer, see fix 1/3 in
/// `acerola-p2p`). Shared between `FileSyncOutbound` and `FileSyncInbound`: an inbound and
/// an outbound for the same peer block each other, not only sessions of the same role.
///
/// History sync does not need this — it is a short JSON round-trip, the overlap window
/// is negligible compared to transferring file bytes.
#[derive(Default)]
pub struct FileSyncSessionGuard {
    active_peers: Mutex<HashSet<String>>,
}

impl FileSyncSessionGuard {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Tries to reserve the session for the peer. `None` if there is already an active session
    /// (inbound or outbound) for the same peer — the caller must refuse the connection without
    /// opening the streams or touching the sync service.
    pub fn try_acquire(self: &Arc<Self>, peer_id: &str) -> Option<FileSyncSessionLease> {
        let mut active_peers = self.active_peers.lock().expect("file sync guard mutex poisoned");
        if !active_peers.insert(peer_id.to_string()) {
            return None;
        }

        Some(FileSyncSessionLease { guard: Arc::clone(self), peer_id: peer_id.to_string() })
    }
}

/// RAII: releases the peer from the guard when the session ends, whether by success, error, or
/// early drop — never needs a manual release path.
pub struct FileSyncSessionLease {
    guard: Arc<FileSyncSessionGuard>,
    peer_id: String,
}

impl Drop for FileSyncSessionLease {
    fn drop(&mut self) {
        self.guard
            .active_peers
            .lock()
            .expect("file sync guard mutex poisoned")
            .remove(&self.peer_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_second_attempt_for_same_peer_is_rejected_while_first_is_active() {
        let guard = FileSyncSessionGuard::new();

        let first_lease = guard.try_acquire("peer-1").expect("first acquisition should succeed");
        assert!(guard.try_acquire("peer-1").is_none());

        drop(first_lease);
        assert!(guard.try_acquire("peer-1").is_some());
    }

    #[test]
    fn test_different_peers_do_not_block_each_other() {
        let guard = FileSyncSessionGuard::new();

        let _lease_a = guard.try_acquire("peer-a").expect("peer-a should acquire");
        assert!(guard.try_acquire("peer-b").is_some());
    }
}
