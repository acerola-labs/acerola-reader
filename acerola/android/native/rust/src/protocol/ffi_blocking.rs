use acerola_p2p::api::error::P2pError;

/// Runs a synchronous FFI call (implemented in Kotlin — file I/O via SAF, Room
/// query via `runBlocking`) on a thread of Tokio's *blocking* pool, without blocking the async
/// runtime's worker threads. No call to `Arc<dyn FileSyncProvider>`/
/// `Arc<dyn HistorySyncProvider>` should run directly inside an `async fn` — the Kotlin behind
/// those traits does real blocking I/O (Room, SAF/DocumentFile, file hashing),
/// and blocking a reactor worker thread with that can saturate the whole runtime under
/// concurrency. `JoinError` (panic or cancellation of the blocking task) becomes
/// `P2pError::StreamFailed` — there is no more specific variant in `ConnectionError` for
/// it, and semantically this failure invalidates the current stream session the same way
/// a network I/O error does.
pub(crate) async fn run_blocking<F, T>(f: F) -> Result<T, P2pError>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|err| P2pError::StreamFailed(format!("blocking FFI call failed: {err}")))
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            atomic::{AtomicU32, Ordering},
            Arc,
        },
        time::Duration,
    };

    use super::*;

    /// With only 1 worker thread, if the blocking call ran directly on the async runtime (without
    /// spawn_blocking), the "tick" task would not even get a chance to progress while the
    /// 300ms call is in progress — that is the exact signature of the reported bug (runtime frozen
    /// during sync-files).
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn run_blocking_does_not_starve_concurrent_async_work() {
        let ticks = Arc::new(AtomicU32::new(0));
        let ticks_clone = Arc::clone(&ticks);

        let ticker = tokio::spawn(async move {
            for _ in 0..30 {
                tokio::time::sleep(Duration::from_millis(10)).await;
                ticks_clone.fetch_add(1, Ordering::SeqCst);
            }
        });

        let blocking = run_blocking(|| std::thread::sleep(Duration::from_millis(300)));

        let (blocking_result, _) = tokio::join!(blocking, ticker);
        assert!(blocking_result.is_ok());

        // If the blocking call had blocked the only worker thread, `ticks` would have stayed
        // near 0 until the 300ms `sleep` released the worker.
        assert!(
            ticks.load(Ordering::SeqCst) >= 15,
            "async ticker was blocked by the blocking call (starvation)"
        );
    }

    #[tokio::test]
    async fn run_blocking_propagates_join_error_as_stream_failed() {
        let result: Result<(), _> = run_blocking(|| panic!("simulated FFI panic")).await;
        assert!(matches!(result, Err(P2pError::StreamFailed(_))));
    }
}
