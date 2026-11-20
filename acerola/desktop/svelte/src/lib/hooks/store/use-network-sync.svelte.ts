import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { error } from '@tauri-apps/plugin-log';
import { SvelteSet } from 'svelte/reactivity';
import { NETWORK_COMMANDS } from '#lib/contracts/network/network.commands.ts';
import { NETWORK_EVENTS } from '#lib/contracts/network/network.events.ts';
import { translateSyncMessage } from '#lib/contracts/errors/sync-error.i18n.ts';
import { m } from '#lib/paraglide/messages.js';

export type TransferLogEntry = {
	id: number;
	/** Empty when the source can't be resolved (e.g. `files:progress`, which carries no peer id). */
	peerId: string;
	kind: 'history' | 'files' | 'comic' | 'historyEntry';
	status: 'started' | 'progress' | 'complete' | 'error';
	message: string;
	timestamp: number;
	/** Only present on `kind: 'comic'` — the comic this session is scoped to (see
	 *  `comic_handler.rs::COMPLETE_EVENT`/`ERROR_EVENT`, payload `{peerId, comicName}` since the
	 *  cover/banner/ComicInfo.xml extension). Used by the comic page to know whether a finished
	 *  session affects the currently open comic, without assuming every individual sync is
	 *  about it. */
	comicName?: string;
	/** Only present on `status: 'complete'` entries of `kind: 'files'`/`'comic'` — how many
	 *  chapters already existed locally with a checksum different from the peer's (a real
	 *  conflict, see `FileSyncService::diff_wanted`). `0`/`undefined` when there was none.
	 *  Fires no toast or separate notification — it only shows on this transfer log line, so
	 *  as not to raise one notification per conflicting chapter. */
	conflicts?: number;
};

type SyncKind = TransferLogEntry['kind'];

/// Explicit direction of a `syncComic` — mirrors `SyncDirection` in the backend
/// (`infra/sync/messages.rs`), serialized as `snake_case` (`"push"`/`"pull"`) in the
/// `sync_comic` Tauri command payload.
export type SyncDirection = 'push' | 'pull';

/// Mirrors `SyncHistoryLogEntry` (`data/models/sync/sync_history_log.rs`, serialized
/// `camelCase`) — one line of the history persisted in SQLite, loaded once when the screen
/// mounts and combined with the live events of the current session.
type PersistedSyncLogEntry = {
	id: number;
	peerId: string;
	kind: string;
	status: string;
	message: string | null;
	createdAt: number;
};

const MAX_LOG_ENTRIES = 200;
// A real network may never return the completion/error event (peer vanished mid
// session) — without this that peer's buttons would stay disabled forever.
// 2-minute inactivity window, renewed on every progress event received from the
// network so large transfers don't hit a false positive midway.
const IN_FLIGHT_TIMEOUT_MS = 120_000;

function syncKey(peerId: string, kind: SyncKind): string {
	return `${peerId}:${kind}`;
}

/// Converts a persisted line into the same shape the live events produce — so
/// `describeEntry()` (on the page) doesn't need to know the difference. Like `push()` below,
/// `message` carries the peer id for `started`/`complete`/`progress` status (used by
/// `peers.peerLabel(...)`) and the real error text only for `error`.
function fromPersisted(row: PersistedSyncLogEntry): TransferLogEntry {
	return {
		// Negative so it never collides with live ids (`nextId` starts at 0 and only grows).
		id: -row.id,
		peerId: row.peerId,
		kind: row.kind as SyncKind,
		status: row.status as TransferLogEntry['status'],
		message: row.status === 'error' ? (row.message ?? '') : row.peerId,
		timestamp: row.createdAt
	};
}

/// Fires sync sessions (history/files) against an already paired peer and keeps a reactive
/// progress log, fed by the `sync:history:*`/`sync:files:*` events that the Rust protocol
/// handlers emit directly (see `infra::sync::protocol` in the backend), plus the persisted
/// history loaded once in `startListening()` (survives restarts — see
/// `get_sync_history_log`/`sync_history_log` in the backend).
export function useNetworkSync() {
	let log = $state<TransferLogEntry[]>([]);
	let nextId = 0;
	const unlisten: UnlistenFn[] = [];

	// `loadPersistedLog` may keep running after `stopListening` already ran (unmount in the
	// middle of the await) — without this flag, the continuation tries to write to the $state
	// of an already discarded instance (`useNetworkSync()` is always a new instance per mount).
	let disposed = false;

	// `peerId:kind` keys with a sync session in progress — used only to disable that
	// peer/protocol's buttons and avoid firing a second concurrent session for the same
	// (peer, ALPN) pair (see the "stream closed before history manifest" error history,
	// caused by two simultaneous sessions competing for the same stream).
	// Per-`kind` granularity, not just per peer: "Sync All" fires history AND files for the
	// same peer, and both must be able to run at once without blocking each other.
	const syncingKeys = new SvelteSet<string>();
	const inFlightTimeouts = new Map<string, ReturnType<typeof setTimeout>>();

	// Resolvers for whoever is waiting for a session's *real* completion (see `syncComic`) —
	// only populated by calls that ask for that wait, so settlePending is a safe no-op for
	// `history`/`files`/`all`, which nobody awaits today.
	const pendingSettlement = new Map<
		string,
		{ resolve: (message: string) => void; reject: (message: string) => void }
	>();

	function settlePending(key: string, ok: boolean, message: string) {
		const pending = pendingSettlement.get(key);
		if (!pending) return;
		pendingSettlement.delete(key);
		if (ok) pending.resolve(message);
		else pending.reject(message);
	}

	function scheduleInFlightTimeout(peerId: string, kind: SyncKind) {
		const key = syncKey(peerId, kind);
		clearTimeout(inFlightTimeouts.get(key));
		inFlightTimeouts.set(
			key,
			setTimeout(() => {
				clearSyncing(peerId, kind);
				const message = m['tauriErrors.sync.timedOut.label']();
				// `clearSyncing` only clears `syncingKeys` (which unlocks the buttons) — without
				// moving the matching log line to a terminal status, it stayed stuck on
				// "started"/"progress" (spinning) forever on the Network screen even with
				// `isSyncing` already `false`, and `inFlightEntryId` never released the key
				// (only `updateInFlightEntry` does that, when processing a terminal status).
				push(peerId, kind, 'error', message);
				// Peer vanished mid session without emitting `complete`/`error` — without this, a
				// `syncComic` awaiting the real completion would hang forever.
				settlePending(key, false, message);
			}, IN_FLIGHT_TIMEOUT_MS)
		);
	}

	/// Resets the inactivity timeout countdown for active sessions of that `kind`.
	/// Called on every progress event received from the network (chapter/file or itemLevel error)
	/// to ensure long transfers with many chapters never time out while real traffic is
	/// flowing.
	function refreshInFlightTimeouts(kind: SyncKind) {
		for (const key of syncingKeys) {
			const [peerId, syncKind] = key.split(':') as [string, SyncKind];
			if (syncKind === kind) {
				scheduleInFlightTimeout(peerId, syncKind);
			}
		}
	}

	function markSyncing(peerId: string, kind: SyncKind) {
		const key = syncKey(peerId, kind);
		syncingKeys.add(key);
		scheduleInFlightTimeout(peerId, kind);
	}

	function clearSyncing(peerId: string, kind: SyncKind) {
		const key = syncKey(peerId, kind);
		syncingKeys.delete(key);
		clearTimeout(inFlightTimeouts.get(key));
		inFlightTimeouts.delete(key);
	}

	function isSyncing(peerId: string, kind: SyncKind): boolean {
		return syncingKeys.has(syncKey(peerId, kind));
	}

	/// Extracts the `peerId` from a `sync:*:error` event payload — these events carry
	/// `{"peerId": ..., "message": ..., "comicName"?: ...}` (see
	/// `history_handler.rs`/`file_handler.rs`/`comic_handler.rs`), unlike `started`, which
	/// carries the bare `peerId` as a string.
	function parseErrorPayload(payload: string): {
		peerId?: string;
		message: string;
		comicName?: string;
		code?: string;
		/** `true` for the 5 PER-ITEM failure points in `receive_files`/`receive_extras` (missing
		 *  blob, fetch failed, checksum mismatch, folder unavailable, persistence failed) — see
		 *  `item_error_payload` in the backend. These events must NEVER touch/finish the running
		 *  session's status line (`started`/`progress`), only show up as their own permanent log
		 *  entry — without this distinction, a single checksum mismatch in the middle of a
		 *  20-chapter sync marked the whole session as "error" on screen, even though it kept
		 *  going (and completed) normally right after. */
		itemLevel?: boolean;
		/** Only present when `itemLevel` is `true` — the chapter or extra `kind`
		 *  (`cover`/`banner`/`comic_info`) that failed, see `item_error_payload`. */
		item?: string;
	} {
		try {
			const parsed = JSON.parse(payload);
			// Stryker disable next-line ConditionalExpression,LogicalOperator: `typeof
			// parsed.message === 'string'` is only true when `parsed` already IS an actual object
			// — no primitive from `JSON.parse` (number, string, boolean) exposes a string
			// `.message` property, so removing the `typeof parsed === 'object'` check never
			// changes the result for these cases. Swapping the `&&` before it for `||` is also
			// equivalent: the only value where it would matter is `parsed = null` (typeof
			// null === 'object', but falsy) — then the mutant evaluates `null.message`, which
			// throws a TypeError, caught by the catch right below, landing on the SAME
			// `return { message: payload }` as the normal fallback. The observable output is
			// identical, only the path changes. Verified empirically: applying each mutant in
			// isolation, no test fails.
			if (parsed && typeof parsed === 'object' && typeof parsed.message === 'string') {
				return {
					peerId: parsed.peerId,
					message: parsed.message,
					comicName: parsed.comicName,
					code: typeof parsed.code === 'string' ? parsed.code : undefined,
					itemLevel: parsed.itemLevel === true,
					item: typeof parsed.item === 'string' ? parsed.item : undefined
				};
			}
		} catch {
			// old/unexpected payload without JSON — treat it as a raw message.
		}
		return { message: payload };
	}

	/// Formats a log entry for an item error (`comicName`/`item` present) — without both
	/// (old/unknown event lacking these fields), falls back to the raw translated
	/// message.
	function formatItemError(
		comicName: string | undefined,
		item: string | undefined,
		message: string
	): string {
		const label = [comicName, item].filter(Boolean).join(' - ');
		return label ? `${label}: ${message}` : message;
	}

	/// Extracts `{peerId, comicName?, conflicts?}` from the `sync:comic:complete`/
	/// `sync:files:complete` payload (see `comic_handler.rs`/`file_handler.rs::COMPLETE_EVENT`) —
	/// `comicName` only comes in the `comic` one. `sync:history:complete`/
	/// `sync:historyEntry:complete` still carry only the bare `peerId` as a string (they never
	/// had a file-conflict concept), so the fallback covers both without breaking.
	function parseCompletePayload(payload: string): {
		peerId: string;
		comicName?: string;
		conflicts?: number;
	} {
		try {
			const parsed = JSON.parse(payload);
			if (parsed && typeof parsed === 'object' && typeof parsed.peerId === 'string') {
				return {
					peerId: parsed.peerId,
					comicName: parsed.comicName,
					conflicts: typeof parsed.conflicts === 'number' ? parsed.conflicts : undefined
				};
			}
		} catch {
			// `sync:history:complete`/`sync:historyEntry:complete`: payload is the raw peerId, not JSON.
		}
		return { peerId: payload };
	}

	/// A sync session has ONE log line, which transitions state (started -> progress ->
	/// complete/error) instead of stacking a new line per event — without this, "started"
	/// stays forever as a separate spinning line next to the "complete" line that already
	/// resolved the same session. Key `peerId:kind` (same granularity as `syncingKeys`).
	/// `files:progress` carries no peer id in its payload (see the `TransferLogEntry` type
	/// comment), so it can't be correlated with a specific peer's `started` line — it keeps
	/// creating its own line; that is a protocol limitation, not a bug in this function.
	const inFlightEntryId = new Map<string, number>();

	function isTerminalStatus(status: TransferLogEntry['status']): boolean {
		return status === 'complete' || status === 'error';
	}

	function isOpenStatus(status: TransferLogEntry['status']): boolean {
		// Stryker disable next-line ConditionalExpression,StringLiteral: this function's result
		// is only read in `appendNewEntry`, always behind `peerId && isOpenStatus(status)`.
		// Every `push(...)` call with `status: 'progress'` in this file (filesProgress,
		// comicProgress) passes `peerId: ''` on purpose — the protocol doesn't correlate
		// `*:progress` events with a peer (see the `inFlightEntryId` comment above). So whenever
		// `status === 'progress'` gets here, `peerId` is already falsy and short-circuits
		// `appendNewEntry` before isOpenStatus matters — forcing the 'progress' branch to
		// always-false or changing the string never changes anything observable. Verified
		// empirically.
		return status === 'started' || status === 'progress';
	}

	function updateInFlightEntry(
		index: number,
		key: string,
		status: TransferLogEntry['status'],
		message: string,
		comicName?: string,
		conflicts?: number
	) {
		const updated: TransferLogEntry = {
			...log[index],
			status,
			message,
			timestamp: Date.now(),
			comicName: comicName ?? log[index].comicName,
			conflicts: conflicts ?? log[index].conflicts
		};
		if (isTerminalStatus(status)) {
			inFlightEntryId.delete(key);
			// Moving the completed/errored entry to the top of the log ensures `sync.log[0]`
			// (used by Home, History and Detail) sees the final outcome even if progress
			// events were added before, and keeps the transfer list ordered with the most
			// recent completion at the top.
			log = [updated, ...log.filter((_, i) => i !== index)];
		} else {
			log[index] = updated;
		}
	}

	function appendNewEntry(
		peerId: string,
		kind: SyncKind,
		key: string,
		status: TransferLogEntry['status'],
		message: string,
		comicName?: string,
		conflicts?: number
	) {
		const entry: TransferLogEntry = {
			id: nextId++,
			peerId,
			kind,
			status,
			message,
			timestamp: Date.now(),
			comicName,
			conflicts
		};
		// `files:progress`/`comic:progress` carry no peer id (see the `inFlightEntryId`
		// comment above) — each event becomes a NEW line here, never cleared via
		// `updateInFlightEntry`. In a sync with many chapters, this filled the
		// `MAX_LOG_ENTRIES` cap with progress alone and pushed the "started" line (the only
		// one with the real `peerId`, tracked in `inFlightEntryId`) out of the array BEFORE the
		// `complete`/`error` event arrived — the line kept spinning ("syncing..." spinner)
		// forever on the Network screen, even with the session already finished in the
		// backend. Protecting lines with a pending transition from the cut avoids this leak;
		// the array can exceed `MAX_LOG_ENTRIES` by at most the number of concurrent sessions
		// still open, which is always small.
		const pendingIds = new Set(inFlightEntryId.values());
		log = [entry, ...log].filter(
			(row, index) => index < MAX_LOG_ENTRIES || pendingIds.has(row.id)
		);
		if (peerId && isOpenStatus(status)) inFlightEntryId.set(key, entry.id);
	}

	function push(
		peerId: string,
		kind: SyncKind,
		status: TransferLogEntry['status'],
		message: string,
		comicName?: string,
		conflicts?: number
	) {
		const key = syncKey(peerId, kind);
		const existingId = peerId ? inFlightEntryId.get(key) : undefined;
		const existingIndex =
			// Stryker disable next-line ConditionalExpression: every `log` entry gets
			// `id: nextId++` (always a number) in `appendNewEntry` — no entry ever has
			// `id === undefined`. So when `existingId` is genuinely `undefined`, forcing the
			// condition to `true` still runs `findIndex((entry) => entry.id === undefined)`, which
			// always returns `-1` — identical to the `false` branch's literal `-1`. Verified
			// empirically.
			existingId !== undefined ? log.findIndex((entry) => entry.id === existingId) : -1;

		if (existingIndex !== -1) {
			updateInFlightEntry(existingIndex, key, status, message, comicName, conflicts);
		} else {
			appendNewEntry(peerId, kind, key, status, message, comicName, conflicts);
		}
	}

	/// Refetches the persisted sessions and replaces only the part of the log that came from
	/// there (`id < 0`, see `fromPersisted`) — used both on mount and by the Network screen's
	/// "Refresh" button. Keeps any LIVE entry still at `id >= 0` (a `started`/`progress`
	/// session from this app run): the repository only stores terminal states
	/// (`complete`/`error`, see the `SyncHistoryLogEntry` doc in the backend), so a refresh in
	/// the middle of a sync must not make the "syncing..." line disappear from the screen.
	/// Propagates the `invoke` error for the caller to decide how to show it (toast) — unlike
	/// the initial load in `startListening`, which fails silently.
	async function refreshLog() {
		const rows = await invoke<PersistedSyncLogEntry[]>(NETWORK_COMMANDS.getSyncHistoryLog);
		if (disposed) return;
		const liveEntries = log.filter((entry) => entry.id >= 0);
		log = [...liveEntries, ...rows.map(fromPersisted)];
	}

	/// Loads the persisted sessions as soon as the screen opens — before any live event
	/// arrives. Fails silently: without persisted history, the screen still works with just
	/// the current session's events.
	async function loadPersistedLog() {
		try {
			await refreshLog();
		} catch (err) {
			if (disposed) return;
			error(`failed to load persisted sync history log: ${err}`);
		}
	}

	/// Permanently erases the transfer history — both the one persisted in SQLite
	/// (`clear_sync_history_log`, irreversible) and the list shown on screen. Propagates the
	/// `invoke` error to the caller (same pattern as `syncHistory`/`syncFiles`).
	async function clearLog() {
		await invoke(NETWORK_COMMANDS.clearSyncHistoryLog);
		if (disposed) return;
		log = [];
	}

	async function startListening() {
		await loadPersistedLog();

		unlisten.push(
			await listen<string>(NETWORK_EVENTS.historyStarted, (event) =>
				push(event.payload, 'history', 'started', event.payload)
			),
			await listen<string>(NETWORK_EVENTS.historyComplete, (event) => {
				clearSyncing(event.payload, 'history');
				push(event.payload, 'history', 'complete', event.payload);
			}),
			await listen<string>(NETWORK_EVENTS.historyError, (event) => {
				const { peerId, message, code } = parseErrorPayload(event.payload);
				if (peerId) clearSyncing(peerId, 'history');
				push(peerId ?? '', 'history', 'error', translateSyncMessage(code, message));
			}),
			await listen<string>(NETWORK_EVENTS.filesStarted, (event) =>
				push(event.payload, 'files', 'started', event.payload)
			),
			await listen<string>(NETWORK_EVENTS.filesProgress, (event) => {
				refreshInFlightTimeouts('files');
				push('', 'files', 'progress', event.payload);
			}),
			await listen<string>(NETWORK_EVENTS.filesComplete, (event) => {
				const { peerId, conflicts } = parseCompletePayload(event.payload);
				clearSyncing(peerId, 'files');
				push(peerId, 'files', 'complete', peerId, undefined, conflicts);
			}),
			await listen<string>(NETWORK_EVENTS.filesError, (event) => {
				const parsed = parseErrorPayload(event.payload);
				if (parsed.itemLevel) {
					refreshInFlightTimeouts('files');
					// `peerId: ''` on purpose — same trick `progress` already uses to never
					// correlate with `inFlightEntryId` and clobber the running session's line
					// (see the `itemLevel` doc in `parseErrorPayload`).
					push(
						'',
						'files',
						'error',
						formatItemError(
							parsed.comicName,
							parsed.item,
							translateSyncMessage(parsed.code, parsed.message)
						),
						parsed.comicName
					);
					return;
				}
				const { peerId, message, code } = parsed;
				if (peerId) clearSyncing(peerId, 'files');
				push(peerId ?? '', 'files', 'error', translateSyncMessage(code, message));
			}),
			await listen<string>(NETWORK_EVENTS.comicStarted, (event) =>
				push(event.payload, 'comic', 'started', event.payload)
			),
			await listen<string>(NETWORK_EVENTS.comicProgress, (event) => {
				refreshInFlightTimeouts('comic');
				push('', 'comic', 'progress', event.payload);
			}),
			await listen<string>(NETWORK_EVENTS.comicComplete, (event) => {
				const { peerId, comicName, conflicts } = parseCompletePayload(event.payload);
				clearSyncing(peerId, 'comic');
				push(peerId, 'comic', 'complete', peerId, comicName, conflicts);
				settlePending(syncKey(peerId, 'comic'), true, peerId);
			}),
			await listen<string>(NETWORK_EVENTS.historyEntryStarted, (event) =>
				push(event.payload, 'historyEntry', 'started', event.payload)
			),
			await listen<string>(NETWORK_EVENTS.historyEntryComplete, (event) => {
				clearSyncing(event.payload, 'historyEntry');
				push(event.payload, 'historyEntry', 'complete', event.payload);
				settlePending(syncKey(event.payload, 'historyEntry'), true, event.payload);
			}),
			await listen<string>(NETWORK_EVENTS.historyEntryError, (event) => {
				const { peerId, message, code } = parseErrorPayload(event.payload);
				const translated = translateSyncMessage(code, message);
				if (peerId) clearSyncing(peerId, 'historyEntry');
				push(peerId ?? '', 'historyEntry', 'error', translated);
				if (peerId) settlePending(syncKey(peerId, 'historyEntry'), false, translated);
			}),
			await listen<string>(NETWORK_EVENTS.comicError, (event) => {
				const parsed = parseErrorPayload(event.payload);
				if (parsed.itemLevel) {
					refreshInFlightTimeouts('comic');
					// Same reasoning as `filesError` above — a chapter/extra that failed inside
					// a `sync-comic` must not be mistaken for the session's final error, nor
					// resolve the `syncComic()` promise (which only cares about the real
					// outcome of the whole session).
					push(
						'',
						'comic',
						'error',
						formatItemError(
							parsed.comicName,
							parsed.item,
							translateSyncMessage(parsed.code, parsed.message)
						),
						parsed.comicName
					);
					return;
				}
				const { peerId, message, comicName, code } = parsed;
				const translated = translateSyncMessage(code, message);
				if (peerId) clearSyncing(peerId, 'comic');
				push(peerId ?? '', 'comic', 'error', translated, comicName);
				if (peerId) settlePending(syncKey(peerId, 'comic'), false, translated);
			})
		);
	}

	function stopListening() {
		disposed = true;
		unlisten.forEach((fn) => fn());
		unlisten.length = 0;
		inFlightTimeouts.forEach((timeout) => clearTimeout(timeout));
		inFlightTimeouts.clear();
		inFlightEntryId.clear();
		// Without this, an `await syncComic(...)` whose component unmounted mid session would
		// hang forever — the listeners that would resolve that promise were just unregistered
		// above.
		pendingSettlement.forEach(({ reject }) => reject('sync cancelled: listener stopped'));
		pendingSettlement.clear();
	}

	/// Calls the Tauri sync command, marking/clearing the `peerId:kind` key(s) around the
	/// invocation itself — the "real" clear (session completed/errored) happens in the event
	/// listeners above; this only covers the command failing immediately (peer not resolved,
	/// channel closed etc.), before any network event is emitted.
	async function withSyncGuard(
		peerId: string,
		kinds: SyncKind[],
		command: string,
		args: Record<string, unknown>
	) {
		if (kinds.some((kind) => isSyncing(peerId, kind))) return;
		kinds.forEach((kind) => markSyncing(peerId, kind));
		try {
			await invoke(command, args);
		} catch (error) {
			kinds.forEach((kind) => clearSyncing(peerId, kind));
			throw error;
		}
	}

	async function syncHistory(peerId: string, addrs: number[]) {
		await withSyncGuard(peerId, ['history'], NETWORK_COMMANDS.syncHistory, { peerId, addrs });
	}

	async function syncFiles(peerId: string, addrs: number[]) {
		await withSyncGuard(peerId, ['files'], NETWORK_COMMANDS.syncFiles, { peerId, addrs });
	}

	async function syncAll(peerId: string, addrs: number[]) {
		await withSyncGuard(peerId, ['history', 'files'], NETWORK_COMMANDS.syncAll, {
			peerId,
			addrs
		});
	}

	/// Individual sync of ONE comic (push or pull, see `comic_handler.rs` in the backend) — the
	/// `comicName` is the same name (`comic_directory.name`) used as the natural key throughout
	/// the rest of the file sync protocol.
	///
	/// `chapterIds` (same `ChapterId` string as the selection UI, see `syncHistoryEntry`) scopes
	/// the session to a subset of chapters — empty (default) syncs the whole comic, the
	/// behavior before this change, used by "Sync with device". The multi-chapter selection's
	/// "Send" button uses the list to send only the chosen files, without sending the rest of
	/// the comic.
	///
	/// Unlike `syncHistory`/`syncFiles`/`syncAll`, the returned promise only resolves (or
	/// rejects) when the session really ends (`sync:comic:complete`/`error`, or the
	/// `markSyncing` timeout) — the Tauri command's `invoke` resolves as soon as the connection
	/// is merely queued, long before the handshake/transfer happens, so using that promise
	/// directly to decide success/error (as the comic screen used to) fires the success toast
	/// too early and never shows the error one if the session fails later.
	async function syncComic(
		peerId: string,
		addrs: number[],
		comicName: string,
		direction: SyncDirection,
		chapterIds: string[] = []
	): Promise<string> {
		if (isSyncing(peerId, 'comic')) {
			// Same translated text as the `code: "busy"` the backend sends — this guard is
			// local (it doesn't even open a connection), but it is the same error from the
			// user's point of view, so it uses the same message instead of raw English text.
			throw new Error(m['tauriErrors.sync.sessionBusy.label']());
		}

		const key = syncKey(peerId, 'comic');
		const settlement = new Promise<string>((resolve, reject) => {
			pendingSettlement.set(key, { resolve, reject });
		});

		try {
			await withSyncGuard(peerId, ['comic'], NETWORK_COMMANDS.syncComic, {
				peerId,
				addrs,
				comicName,
				direction,
				chapterIds
			});
		} catch (err) {
			pendingSettlement.delete(key);
			throw err;
		}

		return settlement;
	}

	/// Pushes the selected chapter(s) (progress + "read" marker) of ONE comic to a peer —
	/// lighter than `syncComic`, which exchanges the whole comic. Used both for sending a
	/// single chapter and several selected at once. `chapterIds` are the same `ChapterId`
	/// (string) used by the rest of the selection UI (see `useChapterSelection` /
	/// `markChaptersReadBatch`) — the backend parses and resolves them to `chapter_sort` (the
	/// key comparable across devices) before building the manifest. Same real-completion wait
	/// pattern as `syncComic`: the `invoke` only queues the connection, the promise
	/// resolves/rejects with `sync:history-entry:complete`/`error`.
	async function syncHistoryEntry(
		peerId: string,
		addrs: number[],
		comicName: string,
		chapterIds: string[]
	): Promise<string> {
		if (isSyncing(peerId, 'historyEntry')) {
			throw new Error(m['tauriErrors.sync.sessionBusy.label']());
		}

		const key = syncKey(peerId, 'historyEntry');
		const settlement = new Promise<string>((resolve, reject) => {
			pendingSettlement.set(key, { resolve, reject });
		});

		try {
			await withSyncGuard(peerId, ['historyEntry'], NETWORK_COMMANDS.syncHistoryEntry, {
				peerId,
				addrs,
				comicName,
				chapterIds
			});
		} catch (err) {
			pendingSettlement.delete(key);
			throw err;
		}

		return settlement;
	}

	/// Timestamp of the last successfully completed session for this peer (any `kind`), or
	/// `undefined` if it never synced — used to show "Last sync: ..." per peer.
	function lastSyncedAt(peerId: string): number | undefined {
		return log.find((entry) => entry.peerId === peerId && entry.status === 'complete')
			?.timestamp;
	}

	/// Most recent live session still in progress (`status: 'started'`), from ANY peer —
	/// used by the global indicator in the header (`+layout.svelte`), which needs to know "is
	/// any sync running now" without depending on any specific screen being mounted. A line
	/// only leaves `'started'` when the `complete`/`error` event arrives and transitions it in
	/// place (`updateInFlightEntry`) — it never gets stuck on `'progress'` because progress
	/// events have no peer id to correlate with it (see the `inFlightEntryId` doc), so checking
	/// only `'started'` already covers the whole session, start to end.
	function activeSession(): { peerId: string; kind: SyncKind } | undefined {
		const entry = log.find((row) => row.status === 'started');
		return entry ? { peerId: entry.peerId, kind: entry.kind } : undefined;
	}

	/// Text of the most recent progress (`"{comicName} - {chapter}"`, see `send_files`/
	/// `receive_files` in the backend) of ANY session — since these events carry no peer id,
	/// there's no telling for sure which session they belong to if more than one is running at
	/// once (`syncAll`, for example); same protocol limitation already documented in
	/// `inFlightEntryId`, not a regression introduced here. Complements `activeSession()`,
	/// does not replace it: `activeSession()` is already `undefined` on its own when nothing runs.
	function activeProgressMessage(): string | undefined {
		return log.find((row) => row.status === 'progress')?.message;
	}

	return {
		startListening,
		stopListening,
		syncHistory,
		syncFiles,
		syncAll,
		syncComic,
		syncHistoryEntry,
		isSyncing,
		lastSyncedAt,
		activeSession,
		activeProgressMessage,
		refreshLog,
		clearLog,
		get log() {
			return log;
		}
	};
}
