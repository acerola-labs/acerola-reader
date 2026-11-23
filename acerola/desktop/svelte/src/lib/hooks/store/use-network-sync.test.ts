import { render } from '@testing-library/svelte';
import { tick } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { error as tauriError } from '@tauri-apps/plugin-log';
import { NETWORK_COMMANDS } from '#lib/contracts/network/network.commands.ts';
import { NETWORK_EVENTS } from '#lib/contracts/network/network.events.ts';
import HookHarness from '../../../../tests/harness/hooks/rune-wrapper.svelte';
import { useNetworkSync } from './use-network-sync.svelte';

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn()
}));

vi.mock('@tauri-apps/api/event', () => ({
	listen: vi.fn()
}));

vi.mock('@tauri-apps/plugin-log', () => ({
	error: vi.fn()
}));

const invokeMock = vi.mocked(invoke);
const listenMock = vi.mocked(listen);
const errorMock = vi.mocked(tauriError);

async function renderHook() {
	let hook: ReturnType<typeof useNetworkSync> | undefined;

	render(HookHarness, {
		props: {
			create: () => useNetworkSync(),
			onReady: (value) => {
				hook = value as ReturnType<typeof useNetworkSync>;
			}
		}
	});

	await tick();
	await Promise.resolve();

	return hook!;
}

function setupListeners() {
	const callbacks = new Map<string, (event: { payload: unknown }) => void>();
	const unlisteners = new Map<string, ReturnType<typeof vi.fn>>();

	listenMock.mockImplementation((event, callback) => {
		callbacks.set(String(event), callback as (event: { payload: unknown }) => void);
		const unlisten = vi.fn();
		unlisteners.set(String(event), unlisten);
		return Promise.resolve(unlisten);
	});

	return { callbacks, unlisteners };
}

describe('useNetworkSync', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		invokeMock.mockResolvedValue(undefined);
	});

	afterEach(() => {
		vi.useRealTimers();
	});

	it('loads the persisted sync history log on startListening', async () => {
		setupListeners();
		invokeMock.mockImplementation((command) => {
			if (command === NETWORK_COMMANDS.getSyncHistoryLog) {
				return Promise.resolve([
					{
						id: 3,
						peerId: 'peer-1',
						kind: 'history',
						status: 'complete',
						message: null,
						createdAt: 1000
					},
					{
						id: 4,
						peerId: 'peer-2',
						kind: 'history',
						status: 'error',
						message: 'boom',
						createdAt: 2000
					}
				]);
			}
			return Promise.resolve(undefined);
		});

		const hook = await renderHook();
		await hook.startListening();

		expect(hook.log).toHaveLength(2);
		expect(hook.log[0]).toMatchObject({
			id: -3,
			peerId: 'peer-1',
			kind: 'history',
			status: 'complete',
			// Non-error: `message` carries the peer id (used by `peers.peerLabel`), not the
			// error text (which doesn't even exist here, it is `null`).
			message: 'peer-1',
			timestamp: 1000
		});
		expect(hook.log[1]).toMatchObject({
			id: -4,
			peerId: 'peer-2',
			kind: 'history',
			status: 'error',
			// Error: `message` carries the real error text, NOT the peerId.
			message: 'boom',
			timestamp: 2000
		});
	});

	it('refreshLog replaces persisted entries but preserves an in-flight live entry', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		// Live session still in progress (id >= 0) — must not vanish on a refresh.
		callbacks.get(NETWORK_EVENTS.historyStarted)?.({ payload: 'peer-live' });
		const liveId = hook.log[0].id;

		invokeMock.mockImplementation((command) => {
			if (command === NETWORK_COMMANDS.getSyncHistoryLog) {
				return Promise.resolve([
					{
						id: 9,
						peerId: 'peer-persisted',
						kind: 'files',
						status: 'complete',
						message: null,
						createdAt: 5000
					}
				]);
			}
			return Promise.resolve(undefined);
		});

		await hook.refreshLog();

		expect(hook.log).toHaveLength(2);
		expect(hook.log.find((entry) => entry.id === liveId)).toMatchObject({
			peerId: 'peer-live',
			status: 'started'
		});
		expect(hook.log.find((entry) => entry.id === -9)).toMatchObject({
			peerId: 'peer-persisted',
			status: 'complete'
		});
	});

	it('refreshLog propagates a backend failure to the caller', async () => {
		setupListeners();
		invokeMock.mockResolvedValue([]);
		const hook = await renderHook();
		await hook.startListening();

		invokeMock.mockRejectedValueOnce(new Error('db locked'));

		await expect(hook.refreshLog()).rejects.toThrow('db locked');
	});

	it('clearLog calls the backend command and empties the log', async () => {
		setupListeners();
		invokeMock.mockImplementation((command) => {
			if (command === NETWORK_COMMANDS.getSyncHistoryLog) {
				return Promise.resolve([
					{
						id: 1,
						peerId: 'peer-1',
						kind: 'history',
						status: 'complete',
						message: null,
						createdAt: 1000
					}
				]);
			}
			return Promise.resolve(undefined);
		});

		const hook = await renderHook();
		await hook.startListening();
		expect(hook.log).toHaveLength(1);

		await hook.clearLog();

		expect(invokeMock).toHaveBeenCalledWith(NETWORK_COMMANDS.clearSyncHistoryLog);
		expect(hook.log).toEqual([]);
	});

	it('clearLog propagates a backend failure without clearing the log', async () => {
		setupListeners();
		invokeMock.mockResolvedValue([]);
		const hook = await renderHook();
		await hook.startListening();

		invokeMock.mockRejectedValueOnce(new Error('disk full'));

		await expect(hook.clearLog()).rejects.toThrow('disk full');
	});

	it('registers listeners for every sync event and appends live entries', async () => {
		const { callbacks, unlisteners } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		expect(unlisteners.size).toBe(14);

		callbacks.get(NETWORK_EVENTS.historyStarted)?.({ payload: 'peer-1' });
		expect(hook.log[0]).toMatchObject({ peerId: 'peer-1', kind: 'history', status: 'started' });

		callbacks.get(NETWORK_EVENTS.historyComplete)?.({ payload: 'peer-1' });
		expect(hook.log[0]).toMatchObject({
			peerId: 'peer-1',
			kind: 'history',
			status: 'complete'
		});
		expect(hook.log).toHaveLength(1);
	});

	it('parses JSON error payloads and clears the syncing flag for that peer', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		// `isSyncing` reflects a sync THIS client started (via `withSyncGuard`), not the
		// `historyStarted` event received from the backend — that event only feeds `log`.
		await hook.syncHistory('peer-1', [1, 2, 3]);
		expect(hook.isSyncing('peer-1', 'history')).toBe(true);

		callbacks.get(NETWORK_EVENTS.historyError)?.({
			payload: JSON.stringify({ peerId: 'peer-1', message: 'stream closed' })
		});

		expect(hook.isSyncing('peer-1', 'history')).toBe(false);
		expect(hook.log[0]).toMatchObject({
			kind: 'history',
			status: 'error',
			message: 'stream closed'
		});
	});

	it('treats a non-JSON error payload as a raw message', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.filesError)?.({ payload: 'connection refused' });

		expect(hook.log[0]).toMatchObject({
			kind: 'files',
			status: 'error',
			message: 'connection refused'
		});
	});

	it('files:progress entries always create a new row (no peerId to correlate)', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.filesProgress)?.({ payload: '10%' });
		callbacks.get(NETWORK_EVENTS.filesProgress)?.({ payload: '20%' });

		expect(hook.log).toHaveLength(2);
		// Prepended (most recent first) with an ever-growing id: the 2nd entry (top) has a
		// greater id than the 1st (older, now below).
		expect(hook.log[1].id).toBeLessThan(hook.log[0].id);
	});

	it('started -> progress -> complete updates the same row in place, then a new started creates a fresh row', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.historyStarted)?.({ payload: 'peer-1' });
		expect(hook.log).toHaveLength(1);
		const startedId = hook.log[0].id;

		callbacks.get(NETWORK_EVENTS.historyStarted)?.({ payload: 'peer-1' });
		expect(hook.log).toHaveLength(1);
		expect(hook.log[0].id).toBe(startedId);
		expect(hook.log[0].status).toBe('started');

		callbacks.get(NETWORK_EVENTS.historyComplete)?.({ payload: 'peer-1' });
		expect(hook.log).toHaveLength(1);
		expect(hook.log[0].id).toBe(startedId);
		expect(hook.log[0].status).toBe('complete');

		// `complete` is terminal: the next `started` for the same peer/kind must not reuse the
		// already resolved line, it has to open a new one.
		callbacks.get(NETWORK_EVENTS.historyStarted)?.({ payload: 'peer-1' });
		expect(hook.log).toHaveLength(2);
		expect(hook.log[0].id).not.toBe(startedId);
		expect(hook.log[0].status).toBe('started');
	});

	it('an error is a terminal status too: a later started for the same peer/kind opens a fresh row', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.historyStarted)?.({ payload: 'peer-16' });
		const startedId = hook.log[0].id;

		callbacks.get(NETWORK_EVENTS.historyError)?.({
			payload: JSON.stringify({ peerId: 'peer-16', message: 'boom' })
		});
		expect(hook.log).toHaveLength(1);
		expect(hook.log[0].id).toBe(startedId);
		expect(hook.log[0].status).toBe('error');

		// `error` closes the session (like `complete`) — the next `started` must not reuse
		// the line already resolved with an error.
		callbacks.get(NETWORK_EVENTS.historyStarted)?.({ payload: 'peer-16' });
		expect(hook.log).toHaveLength(2);
		expect(hook.log[0].id).not.toBe(startedId);
		expect(hook.log[0].status).toBe('started');
	});

	it('a complete/error entry created without a prior in-flight row is not incorrectly reused by a later started event', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		// `historyComplete` arrives without any prior `historyStarted` — creates a new line, but
		// since `complete` is not an "open" status, that line must NOT stay correlatable
		// (otherwise the next `started` for the same peer/kind updates the already resolved
		// line instead of opening a new one).
		callbacks.get(NETWORK_EVENTS.historyComplete)?.({ payload: 'peer-17' });
		expect(hook.log).toHaveLength(1);
		const completeId = hook.log[0].id;
		expect(hook.log[0].status).toBe('complete');

		callbacks.get(NETWORK_EVENTS.historyStarted)?.({ payload: 'peer-17' });
		expect(hook.log).toHaveLength(2);
		expect(hook.log[0].id).not.toBe(completeId);
		expect(hook.log[0].status).toBe('started');
		expect(hook.log[1].id).toBe(completeId);
		expect(hook.log[1].status).toBe('complete');
	});

	it('updates the correct in-flight entry when sessions are interleaved (not just the most recent log row)', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.historyStarted)?.({ payload: 'peer-A' });
		const peerAId = hook.log[0].id;

		callbacks.get(NETWORK_EVENTS.filesStarted)?.({ payload: 'peer-B' });
		const peerBId = hook.log[0].id;
		expect(hook.log).toHaveLength(2);

		// peer-A is no longer at the top of the log (peer-B was prepended on top) — only the
		// entry with the right `id` may be updated, not the array's most recent one.
		callbacks.get(NETWORK_EVENTS.historyComplete)?.({ payload: 'peer-A' });

		expect(hook.log).toHaveLength(2);
		const entryA = hook.log.find((entry) => entry.id === peerAId);
		const entryB = hook.log.find((entry) => entry.id === peerBId);
		expect(entryA).toMatchObject({ peerId: 'peer-A', kind: 'history', status: 'complete' });
		expect(entryB).toMatchObject({ peerId: 'peer-B', kind: 'files', status: 'started' });
	});

	it('isSyncing keys peer and kind together, independently of other peers/kinds', async () => {
		setupListeners();
		invokeMock.mockResolvedValue(undefined);
		const hook = await renderHook();

		let resolveSync: () => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveSync = () => resolve(undefined);
				})
		);
		const pending = hook.syncHistory('peer-a', []);

		expect(hook.isSyncing('peer-a', 'history')).toBe(true);
		expect(hook.isSyncing('peer-a', 'files')).toBe(false);
		expect(hook.isSyncing('peer-b', 'history')).toBe(false);

		resolveSync();
		await pending;
	});

	it('caps the log at MAX_LOG_ENTRIES, dropping the oldest resolved entries', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		// Each session ends (started -> complete) before the next starts, so none stays
		// "pending" — the cap must apply to them normally, as before.
		for (let i = 0; i < 201; i++) {
			callbacks.get(NETWORK_EVENTS.comicStarted)?.({ payload: `peer-${i}` });
			callbacks.get(NETWORK_EVENTS.comicComplete)?.({ payload: `peer-${i}` });
		}

		expect(hook.log).toHaveLength(200);
		// The most recent (peer-200) stays, the oldest (peer-0) was dropped.
		expect(hook.log[0]).toMatchObject({ peerId: 'peer-200' });
		expect(hook.log.some((entry) => entry.peerId === 'peer-0')).toBe(false);
	});

	/// Reproduces the real bug: a large sync produces many `sync:comic:progress` events
	/// (no peer id — see the `appendNewEntry` comment), each becoming a new log line. Before
	/// the fix, this filled the `MAX_LOG_ENTRIES` cap and pushed the "started" line (the only
	/// one tracked per peer, in `inFlightEntryId`) out of the array before the `complete` event
	/// arrived — the line got stuck spinning "syncing..." forever on the Network screen, even
	/// with the session already finished in the backend.
	it('keeps an in-flight entry past MAX_LOG_ENTRIES until it resolves, instead of losing it silently', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.comicStarted)?.({ payload: 'peer-stuck' });
		for (let i = 0; i < 200; i++) {
			callbacks.get(NETWORK_EVENTS.comicProgress)?.({ payload: `chapter-${i}` });
		}

		expect(
			hook.log.find((entry) => entry.peerId === 'peer-stuck' && entry.status === 'started')
		).toBeDefined();

		callbacks.get(NETWORK_EVENTS.comicComplete)?.({ payload: 'peer-stuck' });

		expect(hook.log.find((entry) => entry.peerId === 'peer-stuck')?.status).toBe('complete');
		expect(hook.log.length).toBeLessThanOrEqual(201);
	});

	it('logs when loading the persisted log fails, without throwing', async () => {
		setupListeners();
		invokeMock.mockRejectedValue(new Error('db locked'));

		const hook = await renderHook();
		await expect(hook.startListening()).resolves.toBeUndefined();

		expect(errorMock).toHaveBeenCalledWith(
			expect.stringContaining('failed to load persisted sync history log: Error: db locked')
		);
		expect(hook.log).toEqual([]);
	});

	it('discards a successful persisted-log load if stopListening ran before it resolved', async () => {
		setupListeners();
		let resolveLoad: (rows: unknown[]) => void = () => {};
		invokeMock.mockImplementation((command) => {
			if (command === NETWORK_COMMANDS.getSyncHistoryLog) {
				return new Promise((resolve) => {
					resolveLoad = resolve;
				});
			}
			return Promise.resolve(undefined);
		});

		const hook = await renderHook();
		const startPromise = hook.startListening();
		hook.stopListening();

		resolveLoad([
			{
				id: 1,
				peerId: 'peer-1',
				kind: 'history',
				status: 'complete',
				message: null,
				createdAt: 1
			}
		]);
		await startPromise;

		// `disposed` was already true when the promise resolved — the result must be
		// discarded, not written to `log`.
		expect(hook.log).toEqual([]);
	});

	it('discards a persisted-log load failure if stopListening ran before it rejected (does not log)', async () => {
		setupListeners();
		let rejectLoad: (err: unknown) => void = () => {};
		invokeMock.mockImplementation((command) => {
			if (command === NETWORK_COMMANDS.getSyncHistoryLog) {
				return new Promise((_resolve, reject) => {
					rejectLoad = reject;
				});
			}
			return Promise.resolve(undefined);
		});

		const hook = await renderHook();
		const startPromise = hook.startListening();
		hook.stopListening();

		rejectLoad(new Error('db locked'));
		await startPromise;

		// `disposed` was already true when the promise rejected — must not log the error.
		expect(errorMock).not.toHaveBeenCalled();
	});

	it('treats a JSON error payload that is not an object as a raw message', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.historyError)?.({ payload: '42' });

		expect(hook.log[0]).toMatchObject({ status: 'error', message: '42', peerId: '' });
	});

	it('treats a JSON error payload without a string message field as a raw message', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		const payload = JSON.stringify({ peerId: 'peer-1' });
		callbacks.get(NETWORK_EVENTS.filesError)?.({ payload });

		expect(hook.log[0]).toMatchObject({ status: 'error', message: payload, peerId: '' });
	});

	it('treats a JSON error payload with a non-string message field as a raw message', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		// `message` present but non-string (number): `typeof parsed.message === 'string'`
		// must really be false, not just "a message key exists".
		const payload = JSON.stringify({ peerId: 'peer-1', message: 123 });
		callbacks.get(NETWORK_EVENTS.filesError)?.({ payload });

		expect(hook.log[0]).toMatchObject({ status: 'error', message: payload, peerId: '' });
		expect(typeof hook.log[0].message).toBe('string');
	});

	it('treats a JSON error payload that parses to a truthy non-object primitive as a raw message', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		// `JSON.parse('42')` is a truthy value (number), but not an object — the condition
		// must really check `typeof parsed === 'object'`, not just `parsed`'s truthiness,
		// otherwise this leaks a `{ peerId: undefined, message: undefined }`.
		callbacks.get(NETWORK_EVENTS.historyError)?.({ payload: '42' });

		expect(hook.log[0]).toMatchObject({
			kind: 'history',
			status: 'error',
			message: '42',
			peerId: ''
		});
	});

	it('does not clear a syncing flag for an error payload missing peerId', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		let resolveSync: () => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveSync = () => resolve(undefined);
				})
		);
		const pending = hook.syncHistory('peer-7', []);
		expect(hook.isSyncing('peer-7', 'history')).toBe(true);

		callbacks.get(NETWORK_EVENTS.historyError)?.({
			payload: JSON.stringify({ message: 'no peer id here' })
		});

		expect(hook.isSyncing('peer-7', 'history')).toBe(true);

		resolveSync();
		await pending;
	});

	it('does not clear a syncing flag when the error payload resolves to an empty (falsy) peerId', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		let resolveSync: () => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveSync = () => resolve(undefined);
				})
		);
		// Empty peerId: `syncKey('', 'history')` is a key distinct from any real peer, but it
		// proves that the `if (peerId)` — not an `if (true)` — is what decides whether
		// `clearSyncing` runs.
		const pending = hook.syncHistory('', []);
		expect(hook.isSyncing('', 'history')).toBe(true);

		callbacks.get(NETWORK_EVENTS.historyError)?.({
			payload: JSON.stringify({ peerId: '', message: 'boom' })
		});

		expect(hook.isSyncing('', 'history')).toBe(true);

		resolveSync();
		await pending;
	});

	it('historyComplete clears the syncing flag for the "history" kind specifically, not "files"', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		let resolveSync: () => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveSync = () => resolve(undefined);
				})
		);
		const pending = hook.syncAll('peer-9', []);
		expect(hook.isSyncing('peer-9', 'history')).toBe(true);
		expect(hook.isSyncing('peer-9', 'files')).toBe(true);

		callbacks.get(NETWORK_EVENTS.historyComplete)?.({ payload: 'peer-9' });

		expect(hook.isSyncing('peer-9', 'history')).toBe(false);
		expect(hook.isSyncing('peer-9', 'files')).toBe(true);

		resolveSync();
		await pending;
	});

	it('comicComplete clears the syncing flag for the "comic" kind specifically', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		let resolveSync: () => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveSync = () => resolve(undefined);
				})
		);
		const pending = hook.syncComic('peer-12', [], 'Some Comic', 'push');
		expect(hook.isSyncing('peer-12', 'comic')).toBe(true);

		callbacks.get(NETWORK_EVENTS.comicComplete)?.({ payload: 'peer-12' });

		expect(hook.isSyncing('peer-12', 'comic')).toBe(false);

		resolveSync();
		await pending;
	});

	it('filesError clears the syncing flag for the "files" kind when the payload resolves to a peerId', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		let resolveSync: () => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveSync = () => resolve(undefined);
				})
		);
		const pending = hook.syncFiles('peer-10', []);
		expect(hook.isSyncing('peer-10', 'files')).toBe(true);

		callbacks.get(NETWORK_EVENTS.filesError)?.({
			payload: JSON.stringify({ peerId: 'peer-10', message: 'boom' })
		});

		expect(hook.isSyncing('peer-10', 'files')).toBe(false);

		resolveSync();
		await pending;
	});

	it('does not clear a "files" syncing flag when a filesError payload resolves to an empty peerId', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		let resolveSync: () => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveSync = () => resolve(undefined);
				})
		);
		const pending = hook.syncFiles('', []);
		expect(hook.isSyncing('', 'files')).toBe(true);

		callbacks.get(NETWORK_EVENTS.filesError)?.({
			payload: JSON.stringify({ peerId: '', message: 'boom' })
		});

		expect(hook.isSyncing('', 'files')).toBe(true);

		resolveSync();
		await pending;
	});

	it('each live event pushes a log entry with the exact kind and status for that event', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.filesStarted)?.({ payload: 'peer-f1' });
		expect(hook.log[0]).toMatchObject({ kind: 'files', status: 'started', peerId: 'peer-f1' });

		callbacks.get(NETWORK_EVENTS.filesProgress)?.({ payload: '50%' });
		expect(hook.log[0]).toMatchObject({
			kind: 'files',
			status: 'progress',
			peerId: '',
			message: '50%'
		});

		callbacks.get(NETWORK_EVENTS.comicStarted)?.({ payload: 'peer-c1' });
		expect(hook.log[0]).toMatchObject({ kind: 'comic', status: 'started', peerId: 'peer-c1' });

		callbacks.get(NETWORK_EVENTS.comicProgress)?.({ payload: '75%' });
		expect(hook.log[0]).toMatchObject({
			kind: 'comic',
			status: 'progress',
			peerId: '',
			message: '75%'
		});

		// A peer other than peer-c1 on purpose, so it doesn't correlate with the "started"
		// entry above and keeps coming in as a new line at the top (index 0).
		callbacks.get(NETWORK_EVENTS.comicComplete)?.({ payload: 'peer-c2' });
		expect(hook.log[0]).toMatchObject({ kind: 'comic', status: 'complete', peerId: 'peer-c2' });
	});

	it('clearSyncing cancels the pending in-flight timeout and removes its entry from the map', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		let resolveSync: () => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveSync = () => resolve(undefined);
				})
		);
		const pending = hook.syncHistory('peer-15', []);
		expect(hook.isSyncing('peer-15', 'history')).toBe(true);

		const clearTimeoutSpy = vi.spyOn(globalThis, 'clearTimeout');
		const deleteSpy = vi.spyOn(Map.prototype, 'delete');
		clearTimeoutSpy.mockClear();
		deleteSpy.mockClear();

		// No `historyStarted` was fired for peer-15, so `inFlightEntryId` has no entry for
		// this key. The two `.delete('peer-15:history')` calls expected here come from
		// `syncingKeys.delete(key)` (SvelteSet, which delegates to its internal sources Map)
		// and `inFlightTimeouts.delete(key)`, both inside `clearSyncing`.
		callbacks.get(NETWORK_EVENTS.historyComplete)?.({ payload: 'peer-15' });

		expect(clearTimeoutSpy).toHaveBeenCalledTimes(1);
		const deleteCallsForKey = deleteSpy.mock.calls.filter(
			(call) => call[0] === 'peer-15:history'
		);
		expect(deleteCallsForKey).toHaveLength(2);

		clearTimeoutSpy.mockRestore();
		deleteSpy.mockRestore();

		resolveSync();
		await pending;
	});

	it('syncHistory calls the backend and guards against a concurrent duplicate call', async () => {
		setupListeners();
		invokeMock.mockResolvedValue(undefined);
		const hook = await renderHook();

		let resolveSync: () => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveSync = () => resolve(undefined);
				})
		);

		const first = hook.syncHistory('peer-1', [1, 2, 3]);
		expect(hook.isSyncing('peer-1', 'history')).toBe(true);

		await hook.syncHistory('peer-1', [1, 2, 3]);

		expect(invokeMock).toHaveBeenCalledWith(NETWORK_COMMANDS.syncHistory, {
			peerId: 'peer-1',
			addrs: [1, 2, 3]
		});
		expect(invokeMock).toHaveBeenCalledTimes(1);

		resolveSync();
		await first;
	});

	it('syncFiles marks "files" specifically (not another kind), calls the backend with the right args, and clears + rethrows on failure', async () => {
		setupListeners();
		const failure = new Error('peer unreachable');
		let rejectSync: () => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((_resolve, reject) => {
					rejectSync = () => reject(failure);
				})
		);
		const hook = await renderHook();

		const pending = hook.syncFiles('peer-2', []);
		// Marks the 'files' key — not 'history'/'comic', which syncFiles doesn't touch.
		expect(hook.isSyncing('peer-2', 'files')).toBe(true);
		expect(hook.isSyncing('peer-2', 'history')).toBe(false);
		expect(hook.isSyncing('peer-2', 'comic')).toBe(false);
		expect(invokeMock).toHaveBeenCalledWith(NETWORK_COMMANDS.syncFiles, {
			peerId: 'peer-2',
			addrs: []
		});

		rejectSync();
		await expect(pending).rejects.toThrow(failure);
		expect(hook.isSyncing('peer-2', 'files')).toBe(false);
	});

	it('withSyncGuard bails out if ANY requested kind is already syncing, not only when ALL are', async () => {
		setupListeners();
		let resolveHistorySync: () => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveHistorySync = () => resolve(undefined);
				})
		);
		const hook = await renderHook();

		const historyPending = hook.syncHistory('peer-13', []);
		expect(hook.isSyncing('peer-13', 'history')).toBe(true);
		expect(hook.isSyncing('peer-13', 'files')).toBe(false);

		// 'history' is already syncing — syncAll(['history','files']) must abort entirely
		// (neither mark 'files' nor call the backend again), not only when ALL requested
		// kinds are already syncing.
		await hook.syncAll('peer-13', []);

		expect(hook.isSyncing('peer-13', 'files')).toBe(false);
		expect(invokeMock).not.toHaveBeenCalledWith(NETWORK_COMMANDS.syncAll, expect.anything());

		resolveHistorySync();
		await historyPending;
	});

	it('syncAll marks both history and files kinds as syncing and calls the backend with the right args', async () => {
		setupListeners();
		let resolveSync: () => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveSync = () => resolve(undefined);
				})
		);
		const hook = await renderHook();

		const pending = hook.syncAll('peer-3', []);

		expect(hook.isSyncing('peer-3', 'history')).toBe(true);
		expect(hook.isSyncing('peer-3', 'files')).toBe(true);
		expect(invokeMock).toHaveBeenCalledWith(NETWORK_COMMANDS.syncAll, {
			peerId: 'peer-3',
			addrs: []
		});

		resolveSync();
		await pending;
	});

	it('syncComic calls the backend with the comic name and guards the "comic" kind specifically', async () => {
		const { callbacks } = setupListeners();
		let resolveSync: () => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveSync = () => resolve(undefined);
				})
		);
		const hook = await renderHook();

		const pending = hook.syncComic('peer-4', [9], 'One Piece', 'push');

		// Guards the 'comic' kind specifically — not 'history'/'files', which syncComic doesn't
		// touch.
		expect(hook.isSyncing('peer-4', 'comic')).toBe(true);
		expect(hook.isSyncing('peer-4', 'history')).toBe(false);
		expect(hook.isSyncing('peer-4', 'files')).toBe(false);

		resolveSync();
		// Resolving the `invoke` only queues the connection — until the P2P session has
		// really "finished" (event below), the `syncComic` promise doesn't resolve, so nothing
		// beyond the invoke can be checked up to there.
		await Promise.resolve();

		expect(invokeMock).toHaveBeenCalledWith(NETWORK_COMMANDS.syncComic, {
			peerId: 'peer-4',
			addrs: [9],
			comicName: 'One Piece',
			direction: 'push',
			chapterIds: []
		});
		expect(hook.isSyncing('peer-4', 'comic')).toBe(true);

		await hook.startListening();
		callbacks.get(NETWORK_EVENTS.comicComplete)?.({ payload: 'peer-4' });

		await expect(pending).resolves.toBe('peer-4');
		expect(hook.isSyncing('peer-4', 'comic')).toBe(false);
	});

	it('syncComic forwards chapterIds to the backend when scoped to specific chapters', async () => {
		invokeMock.mockResolvedValueOnce(undefined);
		const hook = await renderHook();

		void hook.syncComic('peer-4', [9], 'One Piece', 'push', ['1', '2']);
		await Promise.resolve();

		expect(invokeMock).toHaveBeenCalledWith(NETWORK_COMMANDS.syncComic, {
			peerId: 'peer-4',
			addrs: [9],
			comicName: 'One Piece',
			direction: 'push',
			chapterIds: ['1', '2']
		});
	});

	it('syncComic rejects with the real error once sync:comic:error arrives, not just on an immediate invoke failure', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValueOnce(undefined);
		const hook = await renderHook();
		await hook.startListening();

		const pending = hook.syncComic('peer-4', [9], 'One Piece', 'push');
		expect(hook.isSyncing('peer-4', 'comic')).toBe(true);

		callbacks.get(NETWORK_EVENTS.comicError)?.({
			payload: JSON.stringify({ peerId: 'peer-4', message: 'peer disconnected' })
		});

		await expect(pending).rejects.toBe('peer disconnected');
		expect(hook.isSyncing('peer-4', 'comic')).toBe(false);
	});

	it('syncHistoryEntry calls the backend with the comic name and chapter ids and guards the "historyEntry" kind specifically', async () => {
		const { callbacks } = setupListeners();
		let resolveSync: () => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveSync = () => resolve(undefined);
				})
		);
		const hook = await renderHook();

		const pending = hook.syncHistoryEntry('peer-5', [9], 'One Piece', ['1', '2']);

		expect(hook.isSyncing('peer-5', 'historyEntry')).toBe(true);
		expect(hook.isSyncing('peer-5', 'history')).toBe(false);
		expect(hook.isSyncing('peer-5', 'comic')).toBe(false);

		resolveSync();
		await Promise.resolve();

		expect(invokeMock).toHaveBeenCalledWith(NETWORK_COMMANDS.syncHistoryEntry, {
			peerId: 'peer-5',
			addrs: [9],
			comicName: 'One Piece',
			chapterIds: ['1', '2']
		});
		expect(hook.isSyncing('peer-5', 'historyEntry')).toBe(true);

		await hook.startListening();
		callbacks.get(NETWORK_EVENTS.historyEntryComplete)?.({ payload: 'peer-5' });

		await expect(pending).resolves.toBe('peer-5');
		expect(hook.isSyncing('peer-5', 'historyEntry')).toBe(false);
	});

	it('syncHistoryEntry rejects with the real error once sync:history-entry:error arrives', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValueOnce(undefined);
		const hook = await renderHook();
		await hook.startListening();

		const pending = hook.syncHistoryEntry('peer-5', [9], 'One Piece', ['1']);
		expect(hook.isSyncing('peer-5', 'historyEntry')).toBe(true);

		callbacks.get(NETWORK_EVENTS.historyEntryError)?.({
			payload: JSON.stringify({ peerId: 'peer-5', message: 'peer disconnected' })
		});

		await expect(pending).rejects.toBe('peer disconnected');
		expect(hook.isSyncing('peer-5', 'historyEntry')).toBe(false);
	});

	it('lastSyncedAt returns the timestamp of the most recent complete entry for a peer', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);
		const hook = await renderHook();
		await hook.startListening();

		expect(hook.lastSyncedAt('peer-5')).toBeUndefined();

		callbacks.get(NETWORK_EVENTS.comicStarted)?.({ payload: 'peer-5' });
		callbacks.get(NETWORK_EVENTS.comicComplete)?.({ payload: 'peer-5' });

		expect(hook.lastSyncedAt('peer-5')).toBeDefined();
	});

	it('lastSyncedAt ignores entries from other peers and non-complete entries of the same peer', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);
		const hook = await renderHook();
		await hook.startListening();

		// peer-6 synced successfully.
		callbacks.get(NETWORK_EVENTS.historyStarted)?.({ payload: 'peer-6' });
		callbacks.get(NETWORK_EVENTS.historyComplete)?.({ payload: 'peer-6' });

		// peer-7 has an open session (not complete) and no "complete" at all.
		callbacks.get(NETWORK_EVENTS.filesStarted)?.({ payload: 'peer-7' });

		expect(hook.lastSyncedAt('peer-6')).toBeDefined();
		// peer-7 only has a "started" entry — must not "leak" another peer's timestamp nor
		// count a non-complete status as a finished sync.
		expect(hook.lastSyncedAt('peer-7')).toBeUndefined();
	});

	it('stopListening unregisters every listener', async () => {
		const { unlisteners } = setupListeners();
		invokeMock.mockResolvedValue([]);
		const hook = await renderHook();
		await hook.startListening();

		hook.stopListening();

		for (const unlisten of unlisteners.values()) {
			expect(unlisten).toHaveBeenCalledOnce();
		}
	});

	it('auto-clears an in-flight syncing flag after the timeout elapses', async () => {
		vi.useFakeTimers();
		setupListeners();
		let resolveSync: () => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveSync = () => resolve(undefined);
				})
		);
		const hook = await renderHook();

		const pending = hook.syncHistory('peer-6', []);
		expect(hook.isSyncing('peer-6', 'history')).toBe(true);

		vi.advanceTimersByTime(120_000);
		expect(hook.isSyncing('peer-6', 'history')).toBe(false);

		resolveSync();
		await pending;
	});

	it('stopListening cancels pending in-flight timeouts so they cannot fire later', async () => {
		vi.useFakeTimers();
		setupListeners();
		let resolveSync: () => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveSync = () => resolve(undefined);
				})
		);
		const hook = await renderHook();

		const pending = hook.syncHistory('peer-14', []);
		expect(hook.isSyncing('peer-14', 'history')).toBe(true);

		hook.stopListening();
		vi.advanceTimersByTime(60_000);

		// If the 60s timeout had not been cancelled in stopListening, it would fire
		// `clearSyncing` here and wipe the flag even without any completion/error event.
		expect(hook.isSyncing('peer-14', 'history')).toBe(true);

		resolveSync();
		await pending;
	});

	/// Regression (05/09/2026): before this fix, `filesError`/`comicError` correlated ANY error
	/// (including the 5 PER-ITEM failure points in `receive_files`/`receive_extras` — missing
	/// blob, fetch failed, checksum mismatch, folder unavailable, persistence failed) with that
	/// peer/kind's running line (`started`/`progress`) and marked it terminal — a 20-chapter
	/// sync with 1 checksum mismatch midway showed "error" on screen even though it kept going
	/// (and completed) normally afterwards. `itemLevel: true` in the payload (see
	/// `item_error_payload` in the backend) is the explicit signal to never do that.
	it('an itemLevel filesError never touches the in-flight session row — it becomes its own permanent entry', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.filesStarted)?.({ payload: 'peer-item' });
		const startedId = hook.log[0].id;
		expect(hook.log[0].status).toBe('started');

		callbacks.get(NETWORK_EVENTS.filesError)?.({
			payload: JSON.stringify({
				peerId: 'peer-item',
				message: 'checksum mismatch',
				code: 'checksum_mismatch',
				comicName: 'One Piece',
				item: 'Ch. 3',
				itemLevel: true
			})
		});

		// The "started" line stays exactly as it was — the item error did not touch it.
		const startedEntry = hook.log.find((entry) => entry.id === startedId);
		expect(startedEntry).toMatchObject({ status: 'started', peerId: 'peer-item' });

		// The item error became its OWN line, without a peerId (never correlatable) but with the
		// comicName/formatted message so the user can tell which chapter failed.
		const itemEntry = hook.log.find((entry) => entry.id !== startedId);
		expect(itemEntry).toMatchObject({
			kind: 'files',
			status: 'error',
			peerId: '',
			comicName: 'One Piece'
		});
		expect(itemEntry?.message).toContain('One Piece');
		expect(itemEntry?.message).toContain('Ch. 3');

		expect(hook.log).toHaveLength(2);
	});

	it('an itemLevel filesError does not clear the syncing flag for that peer', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		let resolveSync: () => void = () => {};
		invokeMock.mockImplementationOnce(
			() =>
				new Promise((resolve) => {
					resolveSync = () => resolve(undefined);
				})
		);
		const pending = hook.syncFiles('peer-item-2', []);
		expect(hook.isSyncing('peer-item-2', 'files')).toBe(true);

		callbacks.get(NETWORK_EVENTS.filesError)?.({
			payload: JSON.stringify({
				peerId: 'peer-item-2',
				message: 'missing blob hash',
				code: 'missing_blob_hash',
				comicName: 'One Piece',
				item: 'Ch. 5',
				itemLevel: true
			})
		});

		// Unlike a SESSION error (see the `filesError clears the syncing flag...` test above),
		// an item error must not unlock the sync button — the session goes on.
		expect(hook.isSyncing('peer-item-2', 'files')).toBe(true);

		resolveSync();
		await pending;
	});

	it('an itemLevel comicError never resolves/rejects the syncComic() promise, only the terminal session error does', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValueOnce(undefined);
		const hook = await renderHook();
		await hook.startListening();

		const pending = hook.syncComic('peer-item-3', [9], 'One Piece', 'push');
		expect(hook.isSyncing('peer-item-3', 'comic')).toBe(true);

		callbacks.get(NETWORK_EVENTS.comicError)?.({
			payload: JSON.stringify({
				peerId: 'peer-item-3',
				message: 'checksum mismatch',
				code: 'checksum_mismatch',
				comicName: 'One Piece',
				item: 'Ch. 2',
				itemLevel: true
			})
		});

		// The session keeps running — the item error neither resolved the promise nor freed the button.
		expect(hook.isSyncing('peer-item-3', 'comic')).toBe(true);

		callbacks.get(NETWORK_EVENTS.comicError)?.({
			payload: JSON.stringify({ peerId: 'peer-item-3', message: 'peer disconnected' })
		});

		await expect(pending).rejects.toBe('peer disconnected');
		expect(hook.isSyncing('peer-item-3', 'comic')).toBe(false);
	});

	it('activeSession is undefined when nothing is syncing', async () => {
		setupListeners();
		invokeMock.mockResolvedValue([]);
		const hook = await renderHook();
		await hook.startListening();

		expect(hook.activeSession()).toBeUndefined();
	});

	it('activeSession reports the peer/kind of a session while it is started, and clears once it completes', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.filesStarted)?.({ payload: 'peer-active' });
		expect(hook.activeSession()).toEqual({ peerId: 'peer-active', kind: 'files' });

		callbacks.get(NETWORK_EVENTS.filesComplete)?.({ payload: 'peer-active' });
		expect(hook.activeSession()).toBeUndefined();
	});

	it('activeSession stays reported across progress ticks (which never touch its started status)', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.comicStarted)?.({ payload: 'peer-active-2' });
		callbacks.get(NETWORK_EVENTS.comicProgress)?.({ payload: 'One Piece - Ch. 5' });

		expect(hook.activeSession()).toEqual({ peerId: 'peer-active-2', kind: 'comic' });
	});

	it('activeProgressMessage returns the most recent progress tick, and clears once resolved', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue([]);

		const hook = await renderHook();
		await hook.startListening();

		expect(hook.activeProgressMessage()).toBeUndefined();

		callbacks.get(NETWORK_EVENTS.filesStarted)?.({ payload: 'peer-progress' });
		callbacks.get(NETWORK_EVENTS.filesProgress)?.({ payload: 'One Piece - Ch. 1' });
		expect(hook.activeProgressMessage()).toBe('One Piece - Ch. 1');

		callbacks.get(NETWORK_EVENTS.filesProgress)?.({ payload: 'One Piece - Ch. 2' });
		expect(hook.activeProgressMessage()).toBe('One Piece - Ch. 2');
	});

	it('comicProgress refreshes the in-flight timeout so long transfers do not time out', async () => {
		vi.useFakeTimers();
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue(undefined);
		const hook = await renderHook();
		await hook.startListening();

		let promiseRejected = false;
		hook.syncComic('peer-long', [1234], 'Big Manga', 'pull').catch(() => {
			promiseRejected = true;
		});
		await tick();

		expect(hook.isSyncing('peer-long', 'comic')).toBe(true);

		// Advance 80s (past the old 60s, but within the new 120s)
		vi.advanceTimersByTime(80_000);
		expect(hook.isSyncing('peer-long', 'comic')).toBe(true);
		expect(promiseRejected).toBe(false);

		// A progress event arrives and resets the inactivity timer
		callbacks.get(NETWORK_EVENTS.comicProgress)?.({ payload: 'Big Manga - Ch. 1' });
		await tick();

		// Advance another 80s (160s total, but only 80s since the last progress)
		vi.advanceTimersByTime(80_000);
		expect(hook.isSyncing('peer-long', 'comic')).toBe(true);
		expect(promiseRejected).toBe(false);

		// A successful completion event arrives
		callbacks.get(NETWORK_EVENTS.comicComplete)?.({
			payload: JSON.stringify({ peerId: 'peer-long', comicName: 'Big Manga' })
		});
		await tick();

		expect(hook.isSyncing('peer-long', 'comic')).toBe(false);
		expect(promiseRejected).toBe(false);
		expect(hook.log[0]).toMatchObject({
			peerId: 'peer-long',
			kind: 'comic',
			status: 'complete',
			comicName: 'Big Manga'
		});
	});

	it('comicComplete payload with conflicts populates entry.conflicts', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue(undefined);
		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.comicComplete)?.({
			payload: JSON.stringify({ peerId: 'peer-conflict', comicName: 'Manga', conflicts: 2 })
		});
		await tick();

		expect(hook.log[0]).toMatchObject({
			peerId: 'peer-conflict',
			kind: 'comic',
			status: 'complete',
			comicName: 'Manga',
			conflicts: 2
		});
	});

	it('filesComplete payload with conflicts populates entry.conflicts', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue(undefined);
		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.filesComplete)?.({
			payload: JSON.stringify({ peerId: 'peer-files-conflict', conflicts: 3 })
		});
		await tick();

		expect(hook.log[0]).toMatchObject({
			peerId: 'peer-files-conflict',
			kind: 'files',
			status: 'complete',
			conflicts: 3
		});
	});

	it('filesComplete payload without conflicts (legacy raw peerId string) still works', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue(undefined);
		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.filesComplete)?.({ payload: 'peer-legacy' });
		await tick();

		expect(hook.log[0]).toMatchObject({
			peerId: 'peer-legacy',
			kind: 'files',
			status: 'complete',
			conflicts: undefined
		});
	});

	it('completing a session moves the terminal entry to index 0 of log', async () => {
		const { callbacks } = setupListeners();
		invokeMock.mockResolvedValue(undefined);
		const hook = await renderHook();
		await hook.startListening();

		callbacks.get(NETWORK_EVENTS.comicStarted)?.({ payload: 'peer-pos' });
		callbacks.get(NETWORK_EVENTS.comicProgress)?.({ payload: 'Ch. 1' });
		callbacks.get(NETWORK_EVENTS.comicProgress)?.({ payload: 'Ch. 2' });

		expect(hook.log[0].status).toBe('progress');
		expect(hook.log[0].message).toBe('Ch. 2');

		callbacks.get(NETWORK_EVENTS.comicComplete)?.({
			payload: JSON.stringify({ peerId: 'peer-pos', comicName: 'Manga' })
		});

		expect(hook.log[0].status).toBe('complete');
		expect(hook.log[0].peerId).toBe('peer-pos');
	});

	it('stopListening clears the in-flight timeout and entry-id maps', async () => {
		setupListeners();
		invokeMock.mockResolvedValue([]);
		const hook = await renderHook();
		await hook.startListening();

		const clearSpy = vi.spyOn(Map.prototype, 'clear');
		clearSpy.mockClear();

		hook.stopListening();

		// `inFlightTimeouts.clear()` + `inFlightEntryId.clear()` + `pendingSettlement.clear()`
		// — exactly three `Map.prototype.clear` calls in this synchronous window.
		expect(clearSpy).toHaveBeenCalledTimes(3);

		clearSpy.mockRestore();
	});
});
