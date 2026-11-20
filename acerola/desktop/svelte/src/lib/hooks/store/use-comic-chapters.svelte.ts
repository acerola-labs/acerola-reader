import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { toast } from 'svelte-sonner';
import { LIBRARY_COMMANDS } from '#lib/contracts/library/chapter.commands.ts';
import { LIBRARY_EVENTS } from '#lib/contracts/library/chapter.events.ts';
import type {
	ChapterPayload,
	ChapterFilePayload,
	ChapterPagePayload
} from '#lib/contracts/library/chapter.payloads.ts';
import type { ErrorPayload } from '#lib/contracts/shared/shared.payloads.ts';
import { resolveErrorMessage } from '#lib/contracts/errors/errors.i18n.ts';
import { notificationStore } from '#lib/components/acerola-notification/acerola-notification.svelte';
import { LRUService } from '#lib/services/lru.service.ts';
import { onMount } from 'svelte';
import { m } from '#lib/paraglide/messages.js';
import { extractErrorMessage } from '#lib/utils/error.util.ts';

const { notify } = notificationStore;

// INFO: The chapter list is fetched whole at once — neither RAM (metadata is
// light) nor DOM (rendering is virtualized by scroll position) depend on
// pagination anymore. A comic with thousands of chapters still sits far below
// this ceiling; it only exists to give the query an explicit limit.
const FETCH_ALL_PAGE_SIZE = 1_000_000;

// INFO: Mirrors the backend cache (ChapterCacheService, chapter_cache.rs) —
// same key (comic + volume + sort + search), same capacity. Reopening a
// volume/sort already seen in this session doesn't even need an IPC round-trip:
// it applies right away, without the blank period that caused the "click
// twice" — quite unlike the old LRU here, which cached pagination blocks of a
// single search (removed along with client-side pagination).
const CHAPTER_CACHE_CAPACITY = 32;

type SortBy = 'number_asc' | 'number_desc' | 'modified_asc' | 'modified_desc';

function buildCacheKey(
	comicDirectoryId: string,
	sortBy: SortBy,
	volumeId: string | null,
	searchQuery: string | null
) {
	return `${comicDirectoryId}|${volumeId ?? ''}|${sortBy}|${searchQuery ?? ''}`;
}

export function useComicChapters() {
	const chapterCache = new LRUService<string, ChapterPayload>({ max: CHAPTER_CACHE_CAPACITY });

	let items = $state<ChapterFilePayload[] | undefined>(undefined);
	let loading = $state(false);
	let failed = false;
	let currentKey: string | null = null;

	let metadata = $state<
		(Omit<ChapterPayload, 'archive'> & { archive: Omit<ChapterPagePayload, 'items'> }) | undefined
	>(undefined);

	function applyPayload(payload: ChapterPayload) {
		items = payload.archive.items;
		failed = false;

		metadata = {
			showVolumeHeaders: payload.showVolumeHeaders,
			hasVolumeStructure: payload.hasVolumeStructure,
			effectiveViewMode: payload.effectiveViewMode,
			archive: {
				page: payload.archive.page,
				pageSize: payload.archive.pageSize,
				total: payload.archive.total,
				volumes: payload.archive.volumes,
				volumeSections: payload.archive.volumeSections
			}
		};

		loading = false;
	}

	function clear(keepMetadata = false) {
		items = undefined;
		failed = false;
		if (!keepMetadata) {
			metadata = undefined;
		}
		loading = false;
	}

	// INFO: `clear()` alone isn't enough to force fresh data — the next `fetch()` for
	// that same key (comic + volume + sort + search) still finds the old payload in
	// `chapterCache` and applies it without any round-trip. Used after a real mutation
	// (light/full rescan), where the content behind the same key changed — unlike just
	// switching volume/sort, where reusing the cache is the right behavior.
	function invalidate() {
		chapterCache.clear();
	}

	onMount(() => {
		let unlistenChapters: (() => void) | undefined;
		let unlistenError: (() => void) | undefined;

		const setupListeners = async () => {
			unlistenChapters = await listen<ChapterPayload>(LIBRARY_EVENTS.comicChapters, (event) => {
				const payload = event.payload;

				applyPayload(payload);

				if (currentKey) {
					chapterCache.set(currentKey, payload);
				}
			});

			unlistenError = await listen<ErrorPayload>(LIBRARY_EVENTS.comicChaptersError, (event) => {
				const errorMessage = resolveErrorMessage(event.payload);

				notify.error(m['hooks.comicChapters.error.load'](), {
					description: errorMessage
				});
				toast.error(errorMessage);

				loading = false;
			});
		};

		setupListeners();

		return () => {
			unlistenChapters?.();
			unlistenError?.();
		};
	});

	async function fetch(
		comicDirectoryId: string,
		sortBy: SortBy,
		volumeId: string | null = null,
		searchQuery: string | null = null
	) {
		if (failed) return;
		if (items !== undefined) return;
		if (loading) return;

		const key = buildCacheKey(comicDirectoryId, sortBy, volumeId, searchQuery);
		const cached = chapterCache.get(key);

		if (cached) {
			applyPayload(cached);
			return;
		}

		loading = true;
		currentKey = key;

		try {
			await invoke(LIBRARY_COMMANDS.getComicChapters, {
				comicDirectoryFk: comicDirectoryId,
				volumeId,
				page: 0,
				pageSize: FETCH_ALL_PAGE_SIZE,
				sortBy,
				searchQuery: searchQuery || null
			});
		} catch (error) {
			const errorMessage = extractErrorMessage(error);

			notify.error(m['hooks.comicChapters.error.request'](), {
				description: errorMessage
			});
			toast.error(errorMessage);

			failed = true;
			loading = false;
		}
	}

	const chapters = $derived.by(() => {
		if (!metadata) return undefined;

		// INFO: `items` stays undefined while a new fetch is in flight
		// (clear(true) always clears items, even while keeping metadata) — falling
		// back to [] here instead of returning undefined keeps archive.volumes from
		// vanishing in the meantime. Without it, expanding a volume made the whole
		// volume list unmount and remount on every click (nothing was really
		// broken, it just flickered and swallowed the first click).
		return {
			...metadata,
			archive: {
				...metadata.archive,
				items: items ?? []
			}
		};
	});

	return {
		fetch,
		clear,
		invalidate,
		get chapters() {
			return chapters;
		},
		get loading() {
			return loading;
		}
	};
}
