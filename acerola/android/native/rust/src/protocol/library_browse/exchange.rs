use std::{sync::Arc, time::Duration};

use acerola_p2p::api::error::P2pError;
use futures::SinkExt;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use super::model::{ComicSummaryEntry, LibrarySummary};
use crate::{callbacks::FileSyncProvider, protocol::ffi_blocking::run_blocking};

const RESPONSE_READ_TIMEOUT: Duration = Duration::from_secs(15);

use crate::protocol::framing::{Recv, Writer};

/// Uses `get_library_summary()` (Room/SQLite-only query, no SAF) instead of
/// `get_file_manifest()` — the latter does a `DocumentFile.exists()` per chapter (one
/// binder transaction each), which is unnecessary cost just to list titles and already caused
/// `browse-library` to exceed `RESPONSE_READ_TIMEOUT` on a large library. See the comment in
/// `callbacks::FileSyncProvider::get_library_summary`.
pub(super) async fn build_summary(
    provider: &Arc<dyn FileSyncProvider>,
) -> Result<LibrarySummary, P2pError> {
    let provider = Arc::clone(provider);
    let entries = run_blocking(move || provider.get_library_summary()).await?;

    let mut comics: Vec<ComicSummaryEntry> = entries
        .into_iter()
        .map(|entry| ComicSummaryEntry {
            comic_name: entry.comic_name,
            chapter_count: entry.chapter_count,
            cover_version: entry.cover_version,
        })
        .collect();
    comics.sort_by(|a, b| a.comic_name.cmp(&b.comic_name));

    Ok(LibrarySummary { comics })
}

/// Inbound role: the marker `run_outbound` writes only exists to satisfy quinn's rule on the
/// dialing side (`open_bi()` must write before the remote `accept_bi()` can
/// see the stream) — this side does NOT wait for nor read that marker before replying. Reading it
/// here would couple the reply to how long the P2P path (NAT traversal/multipath) takes to be
/// ready to carry real data, which can be much longer than the connection's initial
/// handshake — blocking the whole session in the meantime (same problem already fixed on Desktop,
/// see `library_browse_handler.rs` there).
pub(super) async fn run_inbound(
    provider: &Arc<dyn FileSyncProvider>,
    send: Box<dyn AsyncWrite + Send + Unpin>,
) -> Result<(), P2pError> {
    let summary = build_summary(provider).await?;

    let mut writer: Writer = FramedWrite::new(send, LengthDelimitedCodec::new());
    crate::protocol::framing::write_json(&mut writer, &summary).await
}

/// Outbound role: writes a minimal marker (`{}`) before waiting for the reply — without this
/// write, the side that called `open_bi()` never guarantees the inbound will see the stream via
/// `accept_bi()` (rule of quinn, the QUIC lib under iroh), which caused random
/// timeouts/delays instead of a predictable exchange.
pub(super) async fn run_outbound(
    send: Box<dyn AsyncWrite + Send + Unpin>,
    recv: Box<dyn AsyncRead + Send + Unpin>,
) -> Result<LibrarySummary, P2pError> {
    let mut writer: Writer = FramedWrite::new(send, LengthDelimitedCodec::new());
    writer.send(b"{}".to_vec().into()).await.map_err(|err| {
        P2pError::StreamFailed(format!("failed to write library browse request: {err}"))
    })?;

    let mut reader: Recv = FramedRead::new(recv, LengthDelimitedCodec::new());
    crate::protocol::framing::read_json(&mut reader, RESPONSE_READ_TIMEOUT).await
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::callbacks::{FfiComicSummaryEntry, FfiFileManifestEntry};

    struct StubProvider {
        summary: Vec<FfiComicSummaryEntry>,
    }

    impl FileSyncProvider for StubProvider {
        fn get_file_manifest(&self) -> Vec<FfiFileManifestEntry> {
            vec![]
        }
        fn get_library_summary(&self) -> Vec<FfiComicSummaryEntry> {
            self.summary.clone()
        }
        fn open_chapter_for_read(&self, _comic_name: String, _chapter: String) -> i64 {
            -1
        }
        fn read_chapter_chunk(&self, _handle: i64, _chunk_size: u32) -> Vec<u8> {
            vec![]
        }
        fn close_read_handle(&self, _handle: i64) {}
        fn begin_chapter_write(
            &self,
            _comic_name: String,
            _chapter: String,
            _file_name: String,
            _expected_checksum: String,
            _size_bytes: u64,
        ) -> i64 {
            -1
        }
        fn write_chapter_chunk(&self, _handle: i64, _bytes: Vec<u8>) -> bool {
            false
        }
        fn finalize_chapter_write(&self, _handle: i64) -> bool {
            false
        }
        fn abort_chapter_write(&self, _handle: i64) {}
        fn get_extras_manifest(&self) -> Vec<crate::callbacks::FfiExtraManifestEntry> {
            vec![]
        }
        fn open_extra_for_read(&self, _comic_name: String, _kind: String) -> i64 {
            -1
        }
        fn begin_extra_write(
            &self,
            _comic_name: String,
            _kind: String,
            _file_name: String,
            _expected_checksum: String,
            _size_bytes: u64,
        ) -> i64 {
            -1
        }
        fn finalize_extra_write(&self, _handle: i64) -> bool {
            false
        }
        fn abort_extra_write(&self, _handle: i64) {}
    }

    fn summary_entry(comic_name: &str, chapter_count: u32) -> FfiComicSummaryEntry {
        FfiComicSummaryEntry {
            comic_name: comic_name.to_string(),
            chapter_count,
            cover_version: 0,
        }
    }

    /// `build_summary` only maps/sorts what `get_library_summary()` (grouping already done in
    /// SQL, see `ChapterArchiveDao.getLibrarySummary`) returned — it no longer groups anything in Rust.
    #[tokio::test]
    async fn build_summary_sorts_by_comic_name() {
        let provider: Arc<dyn FileSyncProvider> = Arc::new(StubProvider {
            summary: vec![summary_entry("Comic B", 1), summary_entry("Comic A", 3)],
        });

        let summary = build_summary(&provider).await.unwrap();

        assert_eq!(summary.comics.len(), 2);
        assert_eq!(summary.comics[0].comic_name, "Comic A");
        assert_eq!(summary.comics[0].chapter_count, 3);
        assert_eq!(summary.comics[1].comic_name, "Comic B");
        assert_eq!(summary.comics[1].chapter_count, 1);
    }

    /// Real round-trip over a `tokio::io::duplex`: proves `run_inbound`/`run_outbound`
    /// speak the same wire format (without depending on any handler/ALPN, only the pair of
    /// pure I/O functions) — the outbound writes the request marker, but the inbound never reads
    /// anything, it just replies directly (same guarantee the equivalent Desktop test covers).
    #[tokio::test]
    async fn run_outbound_writes_request_but_inbound_never_reads_it_and_still_responds() {
        let provider: Arc<dyn FileSyncProvider> = Arc::new(StubProvider {
            summary: vec![summary_entry("Solo Leveling", 2)],
        });

        let (client_io, server_io) = tokio::io::duplex(64 * 1024);
        let (client_recv, client_send) = tokio::io::split(client_io);
        let (_server_recv, server_send) = tokio::io::split(server_io);

        let inbound_fut = run_inbound(
            &provider,
            Box::new(server_send) as Box<dyn AsyncWrite + Send + Unpin>,
        );
        let outbound_fut = run_outbound(
            Box::new(client_send) as Box<dyn AsyncWrite + Send + Unpin>,
            Box::new(client_recv) as Box<dyn AsyncRead + Send + Unpin>,
        );

        let (inbound_result, outbound_result) = tokio::join!(inbound_fut, outbound_fut);
        inbound_result.expect("inbound should complete without error");
        let summary = outbound_result.expect("outbound should receive the summary without error");

        assert_eq!(summary.comics.len(), 1);
        assert_eq!(summary.comics[0].comic_name, "Solo Leveling");
        assert_eq!(summary.comics[0].chapter_count, 2);
    }
}
