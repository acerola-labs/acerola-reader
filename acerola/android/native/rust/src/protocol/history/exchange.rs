use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

use acerola_p2p::api::{error::P2pError, peer::PeerIdentity, protocol::EventEmitter};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use super::model::{HistoryEntryAck, HistoryEntryRequest, HistoryManifest, HistorySyncStats};
use crate::{
    callbacks::HistorySyncProvider,
    protocol::{ffi_blocking::run_blocking, sync_error::PEER_COMIC_NOT_FOUND_REASON},
};

const MANIFEST_READ_TIMEOUT: Duration = Duration::from_secs(15);
const ENTRY_READ_TIMEOUT: Duration = Duration::from_secs(15);

pub(super) use crate::protocol::framing::{Recv, Writer};

/// Thin layer over `framing::write_json`/`read_json` — see the equivalent comment in
/// `protocol/files/exchange.rs`.
pub(super) async fn write_manifest(
    send: &mut Writer,
    manifest: &HistoryManifest,
) -> Result<(), P2pError> {
    crate::protocol::framing::write_json(send, manifest).await
}

pub(super) async fn read_manifest(recv: &mut Recv) -> Result<HistoryManifest, P2pError> {
    crate::protocol::framing::read_json(recv, MANIFEST_READ_TIMEOUT).await
}

/// Builds the local manifest from the `provider` (Room, via Kotlin). Both FFI calls
/// fit in the same `spawn_blocking` — a single thread hop per session, not two.
pub(super) async fn build_local_manifest(
    provider: &Arc<dyn HistorySyncProvider>,
) -> Result<HistoryManifest, P2pError> {
    let provider = Arc::clone(provider);
    run_blocking(move || HistoryManifest {
        reading_progress: provider.get_reading_progress(),
        chapters_read: provider.get_chapters_read(),
    })
    .await
}

/// Peer reading progress that beats the local one — last-write-wins by `updated_at`.
/// Pure, no FFI: only decides what should be applied, does not apply it.
fn progress_entries_to_apply(
    local: &HistoryManifest,
    peer_progress: Vec<crate::callbacks::FfiReadingProgressEntry>,
) -> Vec<crate::callbacks::FfiReadingProgressEntry> {
    let local_by_key: HashMap<(&str, &str), &crate::callbacks::FfiReadingProgressEntry> = local
        .reading_progress
        .iter()
        .map(|entry| {
            (
                (entry.comic_name.as_str(), entry.chapter_sort.as_str()),
                entry,
            )
        })
        .collect();

    peer_progress
        .into_iter()
        .filter(|peer_entry| {
            let key = (
                peer_entry.comic_name.as_str(),
                peer_entry.chapter_sort.as_str(),
            );
            match local_by_key.get(&key) {
                Some(local_entry) => peer_entry.updated_at > local_entry.updated_at,
                None => true,
            }
        })
        .collect()
}

/// Read chapters the peer has and the local side does not yet — idempotent union, with no notion
/// of "most recent" (unlike progress). Pure, no FFI.
fn chapters_read_entries_to_apply(
    local: &HistoryManifest,
    peer_chapters_read: Vec<crate::callbacks::FfiChapterReadEntry>,
) -> Vec<crate::callbacks::FfiChapterReadEntry> {
    let local_keys: HashSet<(&str, &str)> = local
        .chapters_read
        .iter()
        .map(|entry| (entry.comic_name.as_str(), entry.chapter_sort.as_str()))
        .collect();

    peer_chapters_read
        .into_iter()
        .filter(|peer_entry| {
            let key = (
                peer_entry.comic_name.as_str(),
                peer_entry.chapter_sort.as_str(),
            );
            !local_keys.contains(&key)
        })
        .collect()
}

/// Applies the already decided entries via `provider` — the only part that touches FFI, runs
/// entirely in a single `spawn_blocking` (not one per entry).
async fn apply_entries(
    provider: &Arc<dyn HistorySyncProvider>,
    progress: Vec<crate::callbacks::FfiReadingProgressEntry>,
    chapters_read: Vec<crate::callbacks::FfiChapterReadEntry>,
) -> Result<HistorySyncStats, P2pError> {
    let provider = Arc::clone(provider);
    run_blocking(move || {
        let mut stats = HistorySyncStats::default();

        for entry in progress {
            if provider.apply_reading_progress(entry) {
                stats.progress_applied += 1;
            } else {
                stats.progress_skipped += 1;
            }
        }

        for entry in chapters_read {
            if provider.apply_chapter_read(entry) {
                stats.chapters_read_applied += 1;
            } else {
                stats.chapters_read_skipped += 1;
            }
        }

        stats
    })
    .await
}

/// Diffs the peer's manifest against the local manifest and applies via `provider`:
/// last-write-wins (by `updated_at`) for reading progress, idempotent union for
/// read chapters. Each side runs this function with its own local manifest — there is
/// no further network exchange after this.
pub(super) async fn diff_and_apply(
    local: &HistoryManifest,
    peer: HistoryManifest,
    provider: &Arc<dyn HistorySyncProvider>,
) -> Result<HistorySyncStats, P2pError> {
    let progress = progress_entries_to_apply(local, peer.reading_progress);
    let chapters_read = chapters_read_entries_to_apply(local, peer.chapters_read);
    apply_entries(provider, progress, chapters_read).await
}

/// Runs the manifest exchange for one side of the connection and applies the diff.
///
/// `outbound_role` decides the read/write order (rule 4): the outbound side always
/// writes first at each step, the inbound always reads first — never "whoever arrives first
/// writes". Shared by both handlers since the only difference between them is this order.
pub(super) async fn run_exchange(
    outbound_role: bool,
    peer: &PeerIdentity,
    emit: &EventEmitter,
    provider: &Arc<dyn HistorySyncProvider>,
    send: Box<dyn AsyncWrite + Send + Unpin>,
    recv: Box<dyn AsyncRead + Send + Unpin>,
) -> Result<(), P2pError> {
    let mut writer: Writer = FramedWrite::new(send, LengthDelimitedCodec::new());
    let mut reader: Recv = FramedRead::new(recv, LengthDelimitedCodec::new());

    emit("sync:history:started", started_payload(peer));

    let local_manifest = build_local_manifest(provider).await?;

    let peer_manifest = if outbound_role {
        write_manifest(&mut writer, &local_manifest).await?;
        read_manifest(&mut reader).await?
    } else {
        let peer_manifest = read_manifest(&mut reader).await?;
        write_manifest(&mut writer, &local_manifest).await?;
        peer_manifest
    };

    let stats = diff_and_apply(&local_manifest, peer_manifest, provider).await?;
    emit("sync:history:complete", complete_payload(peer, &stats));

    Ok(())
}

/// Builds the manifest restricted to the selected `chapter_sorts` of ONE comic — used
/// by the explicit push of the selected chapter(s) (`acerola/sync-history-entry/1`). Both
/// FFI calls (progress + read chapters) fit in the same `spawn_blocking`, same
/// reasoning as `build_local_manifest`.
async fn build_manifest_for_chapters(
    provider: &Arc<dyn HistorySyncProvider>,
    comic_name: String,
    chapter_sorts: Vec<String>,
) -> Result<HistoryManifest, P2pError> {
    let provider = Arc::clone(provider);
    let (comic_name_2, sorts_2) = (comic_name.clone(), chapter_sorts.clone());
    run_blocking(move || HistoryManifest {
        reading_progress: provider.get_reading_progress_for_chapters(comic_name, chapter_sorts),
        chapters_read: provider.get_chapters_read_for_chapters(comic_name_2, sorts_2),
    })
    .await
}

/// Runs the one-way push of a manifest restricted to the selected chapter(s)
/// (`acerola/sync-history-entry/1`). Two-message contract (standardized with
/// `acerola/sync-comic/1`): first a `HistoryEntryRequest` (comic + scoped chapters,
/// always present even if the manifest that follows is empty), then the
/// `HistoryManifest` itself. `outbound_role`: the initiating side builds and writes both (scope
/// already resolved from `PendingHistoryEntryScope` by the caller); the responding side reads both,
/// validates the comic exists locally BEFORE applying anything, applies with the SAME
/// diff/LWW/idempotent union logic as the full sync (`diff_and_apply`, comparing against the
/// whole local manifest — still correct even though the peer's manifest is a subset), and
/// confirms with a `HistoryEntryAck` carrying the real result — never an empty ack that
/// would hide a missing comic as success. If `comic_known` comes back `false`, the
/// outbound ends with an error (`PEER_COMIC_NOT_FOUND_REASON`), not a silent success.
pub(super) async fn run_entry_exchange(
    outbound_role: bool,
    scope: Option<(String, Vec<String>)>,
    peer: &PeerIdentity,
    emit: &EventEmitter,
    provider: &Arc<dyn HistorySyncProvider>,
    send: Box<dyn AsyncWrite + Send + Unpin>,
    recv: Box<dyn AsyncRead + Send + Unpin>,
) -> Result<(), P2pError> {
    let mut writer: Writer = FramedWrite::new(send, LengthDelimitedCodec::new());
    let mut reader: Recv = FramedRead::new(recv, LengthDelimitedCodec::new());

    emit("sync:history-entry:started", started_payload(peer));

    if outbound_role {
        let (comic_name, chapter_sorts) = scope.ok_or_else(|| {
            P2pError::StreamFailed("no pending history entry scope for this peer".into())
        })?;

        crate::protocol::framing::write_json(
            &mut writer,
            &HistoryEntryRequest {
                comic_name: comic_name.clone(),
                chapter_sorts: chapter_sorts.clone(),
            },
        )
        .await?;

        let manifest =
            build_manifest_for_chapters(provider, comic_name.clone(), chapter_sorts.clone())
                .await?;
        write_manifest(&mut writer, &manifest).await?;

        let ack: HistoryEntryAck =
            crate::protocol::framing::read_json(&mut reader, ENTRY_READ_TIMEOUT).await?;
        if !ack.comic_known {
            return Err(P2pError::StreamFailed(PEER_COMIC_NOT_FOUND_REASON.into()));
        }

        tracing::info!(
            peer = %peer.id,
            comic = %comic_name,
            chapters = chapter_sorts.len(),
            entries_applied = ack.entries_applied,
            markers_applied = ack.markers_applied,
            "[HistoryEntrySync] push applied by peer",
        );
    } else {
        let request: HistoryEntryRequest =
            crate::protocol::framing::read_json(&mut reader, ENTRY_READ_TIMEOUT).await?;
        let peer_manifest = read_manifest(&mut reader).await?;

        let comic_known = {
            let provider = Arc::clone(provider);
            let comic_name = request.comic_name.clone();
            run_blocking(move || provider.comic_exists(comic_name)).await?
        };
        let stats = if comic_known {
            let local_manifest = build_local_manifest(provider).await?;
            diff_and_apply(&local_manifest, peer_manifest, provider).await?
        } else {
            HistorySyncStats::default()
        };

        crate::protocol::framing::write_json(
            &mut writer,
            &HistoryEntryAck {
                comic_known,
                entries_applied: stats.progress_applied,
                markers_applied: stats.chapters_read_applied,
            },
        )
        .await?;

        tracing::info!(
            comic = %request.comic_name,
            chapters = request.chapter_sorts.len(),
            comic_known,
            entries_applied = stats.progress_applied,
            markers_applied = stats.chapters_read_applied,
            "[HistoryEntrySync] request processed",
        );
    }

    emit("sync:history-entry:complete", started_payload(peer));

    Ok(())
}

fn started_payload(peer: &PeerIdentity) -> String {
    serde_json::json!({ "peerId": peer.id }).to_string()
}

fn complete_payload(peer: &PeerIdentity, stats: &HistorySyncStats) -> String {
    serde_json::json!({
        "peerId": peer.id,
        "progressApplied": stats.progress_applied,
        "progressSkipped": stats.progress_skipped,
        "chaptersReadApplied": stats.chapters_read_applied,
        "chaptersReadSkipped": stats.chapters_read_skipped,
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use std::{
        sync::atomic::{AtomicU32, Ordering},
        time::Duration,
    };

    use super::*;
    use crate::callbacks::{FfiChapterReadEntry, FfiReadingProgressEntry};

    struct SlowProvider {
        sleep_ms: u64,
    }

    impl HistorySyncProvider for SlowProvider {
        fn get_reading_progress(&self) -> Vec<FfiReadingProgressEntry> {
            if self.sleep_ms > 0 {
                std::thread::sleep(Duration::from_millis(self.sleep_ms));
            }
            vec![]
        }
        fn get_chapters_read(&self) -> Vec<FfiChapterReadEntry> {
            vec![]
        }
        fn apply_reading_progress(&self, _entry: FfiReadingProgressEntry) -> bool {
            true
        }
        fn apply_chapter_read(&self, _entry: FfiChapterReadEntry) -> bool {
            true
        }
        fn get_reading_progress_for_comic(
            &self,
            _comic_name: String,
        ) -> Option<FfiReadingProgressEntry> {
            None
        }
        fn get_reading_progress_for_chapters(
            &self,
            _comic_name: String,
            _chapter_sorts: Vec<String>,
        ) -> Vec<FfiReadingProgressEntry> {
            vec![]
        }
        fn get_chapters_read_for_chapters(
            &self,
            _comic_name: String,
            _chapter_sorts: Vec<String>,
        ) -> Vec<FfiChapterReadEntry> {
            vec![]
        }
        fn comic_exists(&self, _comic_name: String) -> bool {
            true
        }
    }

    struct InMemoryProvider {
        progress: std::sync::Mutex<HashMap<String, FfiReadingProgressEntry>>,
        chapters_read: std::sync::Mutex<HashMap<(String, String), FfiChapterReadEntry>>,
        known_comics: std::sync::Mutex<HashSet<String>>,
    }

    impl InMemoryProvider {
        fn new() -> Self {
            Self {
                progress: std::sync::Mutex::new(HashMap::new()),
                chapters_read: std::sync::Mutex::new(HashMap::new()),
                known_comics: std::sync::Mutex::new(HashSet::new()),
            }
        }

        /// Marks `comic_name` as existing locally (with no progress/marker) — used
        /// to simulate "the comic exists, but has no saved history yet".
        fn with_known_comic(self, comic_name: &str) -> Self {
            self.known_comics
                .lock()
                .unwrap()
                .insert(comic_name.to_string());
            self
        }

        fn with_entry(entry: FfiReadingProgressEntry) -> Self {
            let provider = Self::new();
            provider
                .known_comics
                .lock()
                .unwrap()
                .insert(entry.comic_name.clone());
            provider
                .progress
                .lock()
                .unwrap()
                .insert(entry.comic_name.clone(), entry);
            provider
        }

        fn with_chapters_read(self, entries: Vec<FfiChapterReadEntry>) -> Self {
            let mut store = self.chapters_read.lock().unwrap();
            let mut known = self.known_comics.lock().unwrap();
            for entry in entries {
                known.insert(entry.comic_name.clone());
                store.insert(
                    (entry.comic_name.clone(), entry.chapter_sort.clone()),
                    entry,
                );
            }
            drop(store);
            drop(known);
            self
        }
    }

    impl HistorySyncProvider for InMemoryProvider {
        fn get_reading_progress(&self) -> Vec<FfiReadingProgressEntry> {
            self.progress.lock().unwrap().values().cloned().collect()
        }
        fn get_chapters_read(&self) -> Vec<FfiChapterReadEntry> {
            self.chapters_read
                .lock()
                .unwrap()
                .values()
                .cloned()
                .collect()
        }
        fn apply_reading_progress(&self, entry: FfiReadingProgressEntry) -> bool {
            self.progress
                .lock()
                .unwrap()
                .insert(entry.comic_name.clone(), entry);
            true
        }
        fn apply_chapter_read(&self, entry: FfiChapterReadEntry) -> bool {
            self.chapters_read.lock().unwrap().insert(
                (entry.comic_name.clone(), entry.chapter_sort.clone()),
                entry,
            );
            true
        }
        fn get_reading_progress_for_comic(
            &self,
            comic_name: String,
        ) -> Option<FfiReadingProgressEntry> {
            self.progress.lock().unwrap().get(&comic_name).cloned()
        }
        fn get_reading_progress_for_chapters(
            &self,
            comic_name: String,
            chapter_sorts: Vec<String>,
        ) -> Vec<FfiReadingProgressEntry> {
            self.progress
                .lock()
                .unwrap()
                .get(&comic_name)
                .filter(|entry| chapter_sorts.contains(&entry.chapter_sort))
                .cloned()
                .into_iter()
                .collect()
        }
        fn get_chapters_read_for_chapters(
            &self,
            comic_name: String,
            chapter_sorts: Vec<String>,
        ) -> Vec<FfiChapterReadEntry> {
            self.chapters_read
                .lock()
                .unwrap()
                .values()
                .filter(|entry| {
                    entry.comic_name == comic_name && chapter_sorts.contains(&entry.chapter_sort)
                })
                .cloned()
                .collect()
        }
        fn comic_exists(&self, comic_name: String) -> bool {
            self.known_comics.lock().unwrap().contains(&comic_name)
        }
    }

    fn test_peer() -> PeerIdentity {
        PeerIdentity {
            id: "peer-a".to_string(),
            device_id: None,
        }
    }

    fn noop_emitter() -> EventEmitter {
        Arc::new(|_, _| {})
    }

    /// The scope (`comic_name`) only exists on the outbound side (comes from `PendingHistoryEntryScope`,
    /// resolved by the caller before invoking `run_entry_exchange`) — the entry applied on the
    /// inbound side must be exactly the scoped comic's, not the whole library.
    #[tokio::test]
    async fn entry_exchange_pushes_only_the_scoped_comic() {
        let outbound_provider: Arc<dyn HistorySyncProvider> =
            Arc::new(InMemoryProvider::with_entry(FfiReadingProgressEntry {
                comic_name: "Berserk".to_string(),
                chapter_sort: "12".to_string(),
                last_page: 7,
                is_completed: false,
                updated_at: 5000,
            }));
        let inbound_provider: Arc<dyn HistorySyncProvider> =
            Arc::new(InMemoryProvider::new().with_known_comic("Berserk"));

        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);
        let peer = test_peer();
        let emit = noop_emitter();

        let outbound_fut = run_entry_exchange(
            true,
            Some(("Berserk".to_string(), vec!["12".to_string()])),
            &peer,
            &emit,
            &outbound_provider,
            Box::new(client_send),
            Box::new(client_recv),
        );
        let inbound_fut = run_entry_exchange(
            false,
            None,
            &peer,
            &emit,
            &inbound_provider,
            Box::new(server_send),
            Box::new(server_recv),
        );

        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        outbound_result.expect("outbound should complete without error");
        inbound_result.expect("inbound should complete without error");

        let applied = inbound_provider.get_reading_progress_for_comic("Berserk".to_string());
        assert_eq!(applied.map(|entry| entry.last_page), Some(7));
    }

    /// The sent manifest only carries the "read" markers of the selected chapters —
    /// other read chapters of the SAME comic must not leak.
    #[tokio::test]
    async fn entry_exchange_only_sends_read_markers_for_the_selected_chapters() {
        let outbound_provider: Arc<dyn HistorySyncProvider> =
            Arc::new(InMemoryProvider::new().with_chapters_read(vec![
                FfiChapterReadEntry {
                    comic_name: "Berserk".to_string(),
                    chapter_sort: "1".to_string(),
                    created_at: 900,
                },
                FfiChapterReadEntry {
                    comic_name: "Berserk".to_string(),
                    chapter_sort: "2".to_string(),
                    created_at: 900,
                },
            ]));
        let inbound_provider: Arc<dyn HistorySyncProvider> =
            Arc::new(InMemoryProvider::new().with_known_comic("Berserk"));

        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);
        let peer = test_peer();
        let emit = noop_emitter();

        let outbound_fut = run_entry_exchange(
            true,
            Some(("Berserk".to_string(), vec!["2".to_string()])),
            &peer,
            &emit,
            &outbound_provider,
            Box::new(client_send),
            Box::new(client_recv),
        );
        let inbound_fut = run_entry_exchange(
            false,
            None,
            &peer,
            &emit,
            &inbound_provider,
            Box::new(server_send),
            Box::new(server_recv),
        );

        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        outbound_result.expect("outbound should complete without error");
        inbound_result.expect("inbound should complete without error");

        let applied = inbound_provider.get_chapters_read();
        assert_eq!(applied.len(), 1);
        assert_eq!(applied[0].chapter_sort, "2");
    }

    /// Core of the new contract: if the peer does not have the comic, the `HistoryEntryAck` comes
    /// back with `comic_known: false` and the outbound ENDS WITH AN ERROR — no longer the false
    /// positive of "sent successfully" when in fact nothing was applied on the other side.
    #[tokio::test]
    async fn entry_exchange_fails_when_peer_does_not_have_the_comic() {
        let outbound_provider: Arc<dyn HistorySyncProvider> =
            Arc::new(InMemoryProvider::with_entry(FfiReadingProgressEntry {
                comic_name: "Berserk".to_string(),
                chapter_sort: "12".to_string(),
                last_page: 7,
                is_completed: false,
                updated_at: 5000,
            }));
        // "Empty" inbound: no known comic.
        let inbound_provider: Arc<dyn HistorySyncProvider> = Arc::new(InMemoryProvider::new());

        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (server_recv, server_send) = tokio::io::split(server_io);
        let peer = test_peer();
        let emit = noop_emitter();

        let outbound_fut = run_entry_exchange(
            true,
            Some(("Berserk".to_string(), vec!["12".to_string()])),
            &peer,
            &emit,
            &outbound_provider,
            Box::new(client_send),
            Box::new(client_recv),
        );
        let inbound_fut = run_entry_exchange(
            false,
            None,
            &peer,
            &emit,
            &inbound_provider,
            Box::new(server_send),
            Box::new(server_recv),
        );

        let (outbound_result, inbound_result) = tokio::join!(outbound_fut, inbound_fut);
        assert!(outbound_result.is_err());
        inbound_result.expect("inbound must still complete the session (the error is the outbound's)");
    }

    /// Without a registered scope (`comic_name`/`chapter_sorts: None` on the outbound side), the
    /// session fails fast instead of trying to guess what to send.
    #[tokio::test]
    async fn entry_exchange_fails_fast_without_a_scoped_comic() {
        let provider: Arc<dyn HistorySyncProvider> = Arc::new(InMemoryProvider::new());
        let (client_io, _server_io) = tokio::io::duplex(1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let peer = test_peer();
        let emit = noop_emitter();

        let result = run_entry_exchange(
            true,
            None,
            &peer,
            &emit,
            &provider,
            Box::new(client_send),
            Box::new(client_recv),
        )
        .await;

        assert!(result.is_err());
    }

    /// Same proof as in `protocol/files/exchange.rs`: a slow FFI call must not block
    /// another concurrent async task on the same runtime.
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn build_local_manifest_does_not_block_runtime() {
        let provider: Arc<dyn HistorySyncProvider> = Arc::new(SlowProvider { sleep_ms: 150 });

        let ticks = Arc::new(AtomicU32::new(0));
        let ticks_clone = Arc::clone(&ticks);
        let ticker = tokio::spawn(async move {
            for _ in 0..12 {
                tokio::time::sleep(Duration::from_millis(10)).await;
                ticks_clone.fetch_add(1, Ordering::SeqCst);
            }
        });

        let manifest = build_local_manifest(&provider).await.unwrap();
        let _ = tokio::join!(ticker);

        assert!(manifest.reading_progress.is_empty());
        assert!(
            ticks.load(Ordering::SeqCst) >= 6,
            "runtime was blocked during get_reading_progress"
        );
    }
}
