import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { SvelteMap, SvelteSet } from 'svelte/reactivity';
import { NETWORK_COMMANDS } from '#lib/contracts/network/network.commands.ts';
import { NETWORK_EVENTS } from '#lib/contracts/network/network.events.ts';
import { translateSyncMessage } from '#lib/contracts/errors/sync-error.i18n.ts';
import { resolveArtworkPath } from '#lib/utils/artwork.util.ts';
import type {
	ComicSummary,
	CoverQueryResultPayload,
	LibraryQueryResultPayload
} from '#lib/contracts/network/network.payloads.ts';

const coverKey = (peerId: string, comicName: string) => `${peerId}:${comicName}`;

/// Mirrors the error payload of the other `sync:*:error` events (see
/// `use-network-sync.svelte.ts::parseErrorPayload`) — `library:query:error` carries
/// `{ peerId, message, code? }` in the same shape.
function parseErrorPayload(payload: string): { peerId?: string; message: string; code?: string } {
	try {
		const parsed = JSON.parse(payload);
		// Stryker disable next-line ConditionalExpression,LogicalOperator: `typeof parsed.message
		// === 'string'` is only true when `parsed` already IS an actual object — no primitive
		// from `JSON.parse` (number, string, boolean) exposes a string `.message` property, so
		// removing the `typeof parsed === 'object'` check never changes the result for these
		// cases. Swapping the `&&` before it for `||` (LogicalOperator) is also equivalent: the
		// only value where it would matter is `parsed = null` (typeof null === 'object', but
		// falsy) — then the mutant evaluates `null.message`, which throws a TypeError, caught by
		// the catch right below, landing on the SAME `return { message: payload }` as the normal
		// fallback. The observable output is identical in both cases, only the path (exception
		// vs. short-circuit) changes. Verified empirically: applying each mutant in isolation,
		// no test fails.
		if (parsed && typeof parsed === 'object' && typeof parsed.message === 'string') {
			return {
				peerId: parsed.peerId,
				message: parsed.message,
				code: typeof parsed.code === 'string' ? parsed.code : undefined
			};
		}
	} catch {
		// unexpected payload without JSON — treat it as a raw message.
	}
	// Stryker disable next-line ObjectLiteral: this fallback never includes `peerId` — the
	// returned object only has the `message` key. The only consumer (`libraryQueryError`) drops
	// the whole event via `if (!peerId) return;` before reading `message` on this path, so
	// swapping `{ message: payload }` for `{}` changes no observable behavior.
	return { message: payload };
}

/// Queries and caches (per `peerId`) the comic list of remote libraries — used to "pull" an
/// individual comic that only exists on another device (see `sync_comic` in
/// `use-network-sync.svelte.ts`). The query happens once per click (it doesn't refire on
/// every keystroke in the search field): the title filter is reactive on the frontend over
/// the already cached result, same pattern as the Home's "Search comics by title".
export function useRemoteLibrary() {
	const libraries = new SvelteMap<string, ComicSummary[]>();
	const loading = new SvelteSet<string>();
	const errors = new SvelteMap<string, string>();
	const coverPaths = new SvelteMap<string, string>();
	const unlisten: UnlistenFn[] = [];

	// `addrs` of the last `queryRemoteLibrary` per peer — reused to fire `queryRemoteCover` in
	// parallel as soon as the list arrives, without the caller having to keep it again.
	// Already confirmed version (cached or "unchanged") per `"$peerId:$comicName"` — never
	// downloads the same version twice in the same app session.
	const addrsByPeer = new Map<string, number[]>();
	const knownCoverVersions = new Map<string, number>();

	async function startListening() {
		unlisten.push(
			await listen<string>(NETWORK_EVENTS.libraryQueryResult, (event) => {
				const payload = JSON.parse(event.payload) as LibraryQueryResultPayload;
				loading.delete(payload.peerId);
				errors.delete(payload.peerId);
				libraries.set(payload.peerId, payload.comics);
				void fetchCoversFor(payload.peerId, payload.comics);
			}),
			await listen<string>(NETWORK_EVENTS.libraryQueryError, (event) => {
				const { peerId, message, code } = parseErrorPayload(event.payload);
				if (!peerId) return;
				loading.delete(peerId);
				errors.set(peerId, translateSyncMessage(code, message));
			}),
			await listen<string>(NETWORK_EVENTS.coverQueryResult, (event) => {
				const payload = JSON.parse(event.payload) as CoverQueryResultPayload;
				const key = coverKey(payload.peerId, payload.comicName);
				if (payload.coverVersion !== null) knownCoverVersions.set(key, payload.coverVersion);
				if (payload.status === 'changed' && payload.path) {
					const resolved = resolveArtworkPath(payload.path);
					if (resolved) coverPaths.set(key, resolved);
				}
			}),
			await listen<string>(NETWORK_EVENTS.coverQueryError, () => {
				// best-effort — the list stays without a thumb for this item, without blocking the rest.
			})
		);
	}

	function stopListening() {
		unlisten.forEach((fn) => fn());
		unlisten.length = 0;
	}

	async function queryRemoteLibrary(peerId: string, addrs: number[]) {
		addrsByPeer.set(peerId, addrs);
		loading.add(peerId);
		errors.delete(peerId);
		try {
			await invoke(NETWORK_COMMANDS.queryRemoteLibrary, { peerId, addrs });
		} catch (err) {
			loading.delete(peerId);
			errors.set(peerId, String(err));
			throw err;
		}
	}

	/// Fires `acerola/browse-cover/1` in parallel for each comic in the list — streams are
	/// cheap on a connection already pooled by `(peer, alpn)`, no need to serialize.
	async function fetchCoversFor(peerId: string, comics: ComicSummary[]) {
		const addrs = addrsByPeer.get(peerId);
		if (!addrs) return;

		await Promise.all(
			comics.map((comic) => {
				const key = coverKey(peerId, comic.comicName);
				if (knownCoverVersions.get(key) === comic.coverVersion) return Promise.resolve();
				return invoke(NETWORK_COMMANDS.queryRemoteCover, {
					peerId,
					addrs,
					comicName: comic.comicName,
					knownVersion: knownCoverVersions.get(key) ?? null
				}).catch(() => {
					// best-effort, same spirit as coverQueryError above.
				});
			})
		);
	}

	function comicsFor(peerId: string): ComicSummary[] {
		return libraries.get(peerId) ?? [];
	}

	function isLoading(peerId: string): boolean {
		return loading.has(peerId);
	}

	function errorFor(peerId: string): string | undefined {
		return errors.get(peerId);
	}

	function coverPathFor(peerId: string, comicName: string): string | undefined {
		return coverPaths.get(coverKey(peerId, comicName));
	}

	return {
		startListening,
		stopListening,
		queryRemoteLibrary,
		comicsFor,
		isLoading,
		errorFor,
		coverPathFor
	};
}
