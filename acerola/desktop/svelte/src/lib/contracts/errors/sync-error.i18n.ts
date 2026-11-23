import { m } from '#lib/paraglide/messages.js';

// Contract for the optional `code` that the P2P sync protocols (sync-files, sync-comic,
// sync-history, browse-library, browse-cover) attach to the `*:error` payload when the cause
// has a stable translation (see `classify_sync_error`/`SyncErrorCode` in `transfer.rs`) — the
// raw `message` stays English/technical (log/wire, sometimes low-level QUIC text like
// "stream reset by peer"), so without this these errors would show untranslated in the UI
// even with the whole app in pt-BR. An unrecognized code (`classify_sync_error` returned
// `None`) falls into `translateSyncMessage`'s natural fallback — shows the raw `message`, same
// pattern as `errors.i18n.ts` for `ComicError` errors. Shared by every hook that listens to
// these protocols' `*:error` events (`use-network-sync.svelte.ts`,
// `use-remote-library.svelte.ts`) — same `code` everywhere, a single place to translate.
const SYNC_ERROR_MESSAGES: Record<string, () => string> = {
	busy: m['tauriErrors.sync.sessionBusy.label'],
	timeout: m['tauriErrors.sync.timedOut.label'],
	connection_lost: m['tauriErrors.sync.connectionLost.label'],
	peer_not_found: m['tauriErrors.sync.peerNotFound.label'],
	auth_denied: m['tauriErrors.sync.authDenied.label'],
	shutdown: m['tauriErrors.sync.shutdown.label'],
	incompatible_version: m['tauriErrors.sync.incompatibleVersion.label'],
	startup_failed: m['tauriErrors.sync.startupFailed.label'],
	blob_not_found: m['tauriErrors.sync.blobNotFound.label'],
	missing_blob_hash: m['tauriErrors.sync.missingBlobHash.label'],
	blob_fetch_failed: m['tauriErrors.sync.blobFetchFailed.label'],
	checksum_mismatch: m['tauriErrors.sync.checksumMismatch.label'],
	comic_directory_unavailable: m['tauriErrors.sync.comicDirectoryUnavailable.label'],
	persist_failed: m['tauriErrors.sync.persistFailed.label'],
	no_pending_request: m['tauriErrors.sync.noPendingRequest.label'],
	partial_sync: m['tauriErrors.sync.partialSync.label'],
	comic_not_found: m['tauriErrors.sync.comicNotFound.label']
};

export function translateSyncMessage(code: string | undefined, message: string): string {
	return (code && SYNC_ERROR_MESSAGES[code]?.()) ?? message;
}
