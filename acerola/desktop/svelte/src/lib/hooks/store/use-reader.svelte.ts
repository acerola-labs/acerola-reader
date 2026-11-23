import { invoke } from '@tauri-apps/api/core';
import { onDestroy } from 'svelte';
import { READER_COMMANDS } from '#lib/contracts/reader/reader.commands.ts';
import type {
	ReaderCachedPagePayload,
	ReaderChapterPayload,
	ReaderPagePayload,
	ReaderSessionPayload
} from '#lib/contracts/reader/reader.payloads.ts';
import { LRUService } from '#lib/services/lru.service.ts';

// INFO: Mirrors the backend's DEFAULT_CACHE_CAPACITY (core/services/reader/mod.rs) —
// a sliding window, not the whole chapter. It is only the value used before
// session.cacheCapacity arrives from the backend; resetCache(session.cacheCapacity)
// overrides it as soon as the session opens.
const DEFAULT_READER_CACHE_SIZE = 20;
export const PREFETCH_RADIUS = 2;

export function useReader() {
	let pageCache = createPageCache(DEFAULT_READER_CACHE_SIZE);
	let pendingPages = new Map<number, Promise<ReaderCachedPagePayload | undefined>>();
	let requestVersion = 0;

	let session = $state<ReaderSessionPayload | undefined>(undefined);
	let currentPage = $state(0);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let cacheVersion = $state(0);

	onDestroy(() => {
		pageCache.clear();
		void invoke(READER_COMMANDS.closeChapter).catch(() => undefined);
	});

	function createPageCache(max: number) {
		return new LRUService<number, ReaderCachedPagePayload>({
			max,
			dispose: (page) => URL.revokeObjectURL(page.url)
		});
	}

	function resetCache(max = DEFAULT_READER_CACHE_SIZE) {
		// Stryker disable next-line UpdateOperator: requestVersion is only compared for
		// (in)equality against a previously captured value (`version !== requestVersion` /
		// `version === requestVersion`), never ordered or displayed. This is the only
		// increment point, so the value sequence it produces is strictly monotonic either
		// way (0,1,2,... or 0,-1,-2,...) — always injective, never colliding with a previous
		// value. Swapping ++ for -- labels each "changed" vs "unchanged" result identically;
		// no equality check in this file tells the two apart. Verified empirically.
		requestVersion++;
		pageCache.clear();
		pageCache = createPageCache(max);
		pendingPages.clear();
		// Stryker disable next-line UpdateOperator: cacheVersion is only read bare
		// (`cacheVersion;`) inside $derived.by blocks to force Svelte to treat them as
		// dependent on it — its numeric value is never compared or displayed. ++ and --
		// change the value by exactly 1 from what it was, so both always trigger Svelte's
		// dirty-check and recompute the same way; the direction of the change is
		// unobservable. Verified empirically.
		cacheVersion++;
	}

	function setError(value: unknown) {
		error = value instanceof Error ? value.message : String(value);
	}

	function clampPage(index: number) {
		if (!session) return 0;
		return Math.max(0, Math.min(index, session.pageCount - 1));
	}

	function isValidPage(index: number) {
		return Boolean(session && index >= 0 && index < session.pageCount);
	}

	function toCachedPage(payload: ReaderPagePayload): ReaderCachedPagePayload {
		const bytes = new Uint8Array(payload.bytes);
		const blob = new Blob([bytes], { type: payload.mimeType });

		return {
			index: payload.index,
			total: payload.total,
			mimeType: payload.mimeType,
			url: URL.createObjectURL(blob),
			cacheHit: payload.cacheHit
		};
	}

	function windowIndices(center: number) {
		// Stryker disable next-line ConditionalExpression: prefetchWindow() (the only caller
		// of this function) already returns early through its own `if (!session)` guard,
		// and does so synchronously without any `await` before calling windowIndices()
		// — so `session` can never change between the two checks. This guard is
		// unreachably false given the single current call site. Verified empirically.
		if (!session) return [];

		const start = Math.max(0, center - PREFETCH_RADIUS);
		// Stryker disable next-line ArithmeticOperator: this clamp's only role is to keep
		// the window from going past the last valid page. Whenever it is the active
		// constraint (i.e. when this mutation would change `end`'s value), every index it
		// would admit is >= pageCount and therefore invalid; loadPage's own isValidPage()
		// guard silently drops those before any observable side effect (no backend call,
		// no cache write). Verified empirically: applying this mutation and running the
		// whole suite leaves every test green.
		const end = Math.min(session.pageCount - 1, center + PREFETCH_RADIUS);

		return Array.from({ length: end - start + 1 }, (_, index) => start + index);
	}

	async function open(chapter: ReaderChapterPayload, startPage = 0) {
		loading = true;
		error = null;
		resetCache();

		try {
			session = await invoke<ReaderSessionPayload>(READER_COMMANDS.openChapter, { chapter });
			resetCache(session.cacheCapacity);

			currentPage = clampPage(startPage);
			await goToPage(currentPage);

			return session;
		} catch (caught) {
			setError(caught);
			throw caught;
		} finally {
			loading = false;
		}
	}

	async function loadPage(index: number, setCurrent = true) {
		if (!isValidPage(index)) return undefined;

		const cached = setCurrent ? pageCache.get(index) : pageCache.peek(index);
		if (cached) {
			if (setCurrent) {
				await syncCurrentPage(index);
			}

			// Stryker disable next-line UpdateOperator: cacheVersion is only read bare
			// (`cacheVersion;`) inside $derived.by blocks to force Svelte to treat them as
			// dependent on it — its numeric value is never compared or displayed. ++ and --
			// change the value by exactly 1 from what it was, so both always trigger
			// Svelte's dirty-check and recompute the same way; the direction of the change
			// is unobservable. Verified empirically.
			cacheVersion++;
			return cached;
		}

		const pending = pendingPages.get(index);
		if (pending) return pending;

		const version = requestVersion;
		// Stryker disable next-line OptionalChaining: isValidPage(index) at the top of this
		// function already required `session` to be truthy (it short-circuits `session && ...`
		// to false otherwise), and nothing between that check and this line awaits — so
		// `session` cannot have become undefined yet. Verified empirically.
		const chapterId = session?.chapter.id;

		const request = (async () => {
			if (setCurrent) loading = true;

			try {
				const payload = await invoke<ReaderPagePayload>(READER_COMMANDS.loadPage, {
					index,
					setCurrent
				});

				// The two disjuncts below (`payload.chapterId !== chapterId` and `payload.chapterId
				// !== session?.chapter.id`) only matter for the result when `version ===
				// requestVersion` (otherwise the first disjunct above already discards). And
				// `session` only changes via `open()` or `close()`, both of which always increment
				// `requestVersion` synchronously in the same tick as the session swap, with no
				// `await` in between — so whenever `version === requestVersion`, `session` hasn't
				// changed since `chapterId` was captured, which guarantees
				// `chapterId === session?.chapter.id`. That is, the two disjuncts below always
				// evaluate to the SAME boolean in the only regime where they make a difference —
				// mutating one alone never changes the final result, because the other (intact)
				// one always "covers" the same check. Verified empirically.
				if (
					version !== requestVersion ||
					// Stryker disable next-line ConditionalExpression
					payload.chapterId !== chapterId ||
					// Stryker disable next-line ConditionalExpression,OptionalChaining
					payload.chapterId !== session?.chapter.id
				) {
					return undefined;
				}

				const page = toCachedPage(payload);

				pageCache.set(index, page);

				if (setCurrent) {
					currentPage = index;
				}

				error = null;
				// Stryker disable next-line UpdateOperator: same reasoning as the other
				// cacheVersion++ points — only read bare as a Svelte dependency trigger,
				// never compared, so ++ vs -- is unobservable. Verified empirically.
				cacheVersion++;

				return page;
			} catch (caught) {
				if (version === requestVersion) {
					setError(caught);
				}

				throw caught;
			} finally {
				if (version === requestVersion) {
					pendingPages.delete(index);
				}
				if (setCurrent && version === requestVersion) loading = false;
			}
		})();

		pendingPages.set(index, request);
		return request;
	}

	async function syncCurrentPage(index: number) {
		// Stryker disable next-line ConditionalExpression: this function's only caller
		// (loadPage's cache branch) gets `cached` via `pageCache.get(index)` right before
		// calling syncCurrentPage(index) with that SAME index, and isValidPage(index) is
		// implicitly true for any index that was actually cached — nothing awaits in
		// between, so this guard never sees an invalid index. Verified empirically.
		if (!isValidPage(index)) return;

		currentPage = index;
		// Stryker disable next-line CallExpression: the caller above already called
		// `pageCache.get(index)` (not `.peek()`) for this exact index right before invoking
		// syncCurrentPage, which already promoted the entry to most-recently-used in the
		// LRU cache. This second `.get()` call touches the same already-MRU entry again — a
		// no-op for ordering purposes, since nothing else reads or writes the cache in the
		// meantime. Verified empirically.
		pageCache.get(index);
		// Stryker disable next-line UpdateOperator: same reasoning as the other
		// cacheVersion++ points — only read bare as a Svelte dependency trigger, never
		// compared, so ++ vs -- is unobservable. Verified empirically.
		cacheVersion++;

		try {
			await invoke(READER_COMMANDS.setCurrentPage, { index });
		} catch (caught) {
			setError(caught);
		}
	}

	async function prefetchWindow(center = currentPage) {
		if (!session) return;

		void invoke(READER_COMMANDS.prefetchWindow, {
			center,
			radius: PREFETCH_RADIUS
		}).catch((caught) => setError(caught));

		for (const index of windowIndices(center)) {
			if (index !== center && !pageCache.has(index)) {
				void loadPage(index, false).catch(() => undefined);
			}
		}
	}

	async function goToPage(index: number) {
		const nextPage = clampPage(index);
		const page = await loadPage(nextPage, true);

		if (page) {
			void prefetchWindow(nextPage);
		}

		return page;
	}

	function nextPage() {
		return goToPage(currentPage + 1);
	}

	function previousPage() {
		return goToPage(currentPage - 1);
	}

	async function close() {
		try {
			await invoke(READER_COMMANDS.closeChapter);
		} finally {
			session = undefined;
			currentPage = 0;
			loading = false;
			error = null;
			resetCache();
		}
	}

	const pages = $derived.by(() => {
		cacheVersion;

		return pageCache.keys
			.slice()
			.sort((left, right) => left - right)
			.map((key) => pageCache.peek(key))
			.filter((page): page is ReaderCachedPagePayload => Boolean(page));
	});

	const current = $derived.by(() => {
		cacheVersion;
		return pageCache.peek(currentPage);
	});

	const cacheKeys = $derived.by(() => {
		cacheVersion;
		return pageCache.keys;
	});

	return {
		open,
		loadPage,
		goToPage,
		nextPage,
		previousPage,
		prefetchWindow,
		close,
		pageAt(index: number) {
			cacheVersion;
			return pageCache.peek(index);
		},
		get session() {
			return session;
		},
		get pageCount() {
			return session?.pageCount ?? 0;
		},
		get currentPage() {
			return currentPage;
		},
		get current() {
			return current;
		},
		get pages() {
			return pages;
		},
		get cacheKeys() {
			return cacheKeys;
		},
		get loading() {
			return loading;
		},
		get error() {
			return error;
		}
	};
}
