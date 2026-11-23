use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// Scope of ONE history push: the comic and the chapters (`chapter_archive_id`, the same
/// unit used by the rest of the local UI, e.g. `markChaptersReadBatch`) selected by the
/// user to send — the exchanged manifest (`acerola/sync-history-entry/1`) is restricted to
/// that, never the whole library. `HistorySyncService::build_manifest_for_chapters` resolves
/// the IDs to `chapter_sort` (the key comparable across devices) before building the manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntryScope {
    pub comic_name: String,
    pub chapter_ids: Vec<i64>,
}

/// Same side channel as `PendingComicSyncRegistry`, but for the history push restricted to
/// selected chapter(s) (`acerola/sync-history-entry/1`): the Tauri command
/// `sync_history_entry` writes the [`HistoryEntryScope`] before calling `connect()`, and
/// `HistoryEntrySyncOutbound` consumes (`take`) that value as soon as the session starts.
#[derive(Default)]
pub struct PendingHistoryEntryRegistry {
    pending: Mutex<HashMap<String, HistoryEntryScope>>,
}

impl PendingHistoryEntryRegistry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn set(&self, peer_id: String, scope: HistoryEntryScope) {
        self.pending
            .lock()
            .expect("pending history entry registry mutex poisoned")
            .insert(peer_id, scope);
    }

    /// Consumes (removes) the pending scope for the peer — each `connect()` call only serves
    /// one session.
    pub fn take(&self, peer_id: &str) -> Option<HistoryEntryScope> {
        self.pending.lock().expect("pending history entry registry mutex poisoned").remove(peer_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scope(comic_name: &str, chapter_ids: &[i64]) -> HistoryEntryScope {
        HistoryEntryScope { comic_name: comic_name.to_string(), chapter_ids: chapter_ids.to_vec() }
    }

    #[test]
    fn take_removes_the_pending_entry() {
        let registry = PendingHistoryEntryRegistry::new();
        registry.set("peer-1".to_string(), scope("Berserk", &[1, 2]));

        assert_eq!(registry.take("peer-1"), Some(scope("Berserk", &[1, 2])));
        assert_eq!(registry.take("peer-1"), None);
    }

    #[test]
    fn take_without_a_pending_entry_returns_none() {
        let registry = PendingHistoryEntryRegistry::new();
        assert_eq!(registry.take("peer-unknown"), None);
    }
}
