use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::infra::sync::messages::SyncDirection;

/// `Handler::handle()` (`acerola-p2p` trait) is called without any invocation-specific
/// parameter — the `acerola/sync-comic/1` handlers are singletons registered only once at
/// boot (`bios::network::setup_network`). So that the side that STARTS an individual sync (the
/// Tauri command `sync_comic`) can tell `ComicSyncOutbound` which `comic_name`/`direction` to use
/// for that specific `connect()` call, this registry works as a side channel: the
/// command writes the triple (peer_id, comic_name, direction) before calling `connect()`, and the
/// handler consumes (`take`) that value as soon as the session starts. Only the outbound side
/// needs this — the inbound side receives `comic_name`/`direction` through the `ComicSyncRequest`
/// itself on the wire.
///
/// `chapter_ids` (local `chapter_archive_id`s, empty = whole comic) scopes the session to a
/// subset of chapters — the same unit `HistoryEntryScope::chapter_ids` uses for the history
/// push. `ComicSyncOutbound::run` resolves these IDs to labels (`FileChapterInfo.chapter`)
/// before building the `ComicSyncRequest`, because only the outbound has the local IDs; the
/// inbound receives the labels already resolved over the wire.
/// Pending `(comic_name, direction, chapter_ids)` for a peer — extracted into an alias because
/// clippy (`type_complexity`) complains about the nested triple tuple directly in `Mutex<HashMap<...>>`.
type PendingComicSync = (String, SyncDirection, Vec<i64>);

#[derive(Default)]
pub struct PendingComicSyncRegistry {
    pending: Mutex<HashMap<String, PendingComicSync>>,
}

impl PendingComicSyncRegistry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn set(
        &self, peer_id: String, comic_name: String, direction: SyncDirection,
        chapter_ids: Vec<i64>,
    ) {
        self.pending
            .lock()
            .expect("pending comic sync registry mutex poisoned")
            .insert(peer_id, (comic_name, direction, chapter_ids));
    }

    /// Consumes (removes) the pending `(comic_name, direction, chapter_ids)` for the peer — each
    /// `connect()` call only serves one session, so there is no point keeping the value there
    /// after it is read.
    pub fn take(&self, peer_id: &str) -> Option<PendingComicSync> {
        self.pending.lock().expect("pending comic sync registry mutex poisoned").remove(peer_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_removes_the_pending_entry() {
        let registry = PendingComicSyncRegistry::new();
        registry.set("peer-1".to_string(), "Berserk".to_string(), SyncDirection::Push, vec![]);

        assert_eq!(
            registry.take("peer-1"),
            Some(("Berserk".to_string(), SyncDirection::Push, vec![]))
        );
        assert_eq!(registry.take("peer-1"), None);
    }

    #[test]
    fn take_without_a_pending_entry_returns_none() {
        let registry = PendingComicSyncRegistry::new();
        assert_eq!(registry.take("peer-unknown"), None);
    }
}
