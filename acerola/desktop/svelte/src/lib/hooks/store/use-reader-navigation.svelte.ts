import { browser } from '$app/environment';
import { replaceState } from '$app/navigation';
import { page } from '$app/state';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { ReaderChapterPayload } from '#lib/contracts/reader/reader.payloads.ts';
import { LIBRARY_EVENTS } from '#lib/contracts/library/chapter.events.ts';
import { LIBRARY_COMMANDS } from '#lib/contracts/library/chapter.commands.ts';
import { SESSION_KEYS } from '#lib/constants/session-keys.ts';
import type { ChapterPayload } from '#lib/contracts/library/chapter.payloads.ts';
import { onMount } from 'svelte';

export type ReaderNavigationState = {
	chapter?: ReaderChapterPayload;
	startPage?: number;
	chapterIndex?: number;
	totalChapters?: number;
	chapterScope?: string;
	comicDirectoryId?: string;
	sortBy?: 'number_asc' | 'number_desc' | 'modified_asc' | 'modified_desc';
};

export function useReaderNavigation() {
	let initializing = $state(false);

	// Keeps track of the pending navigation so the listener knows what to do
	let pendingAction = $state<'load' | 'navigate' | null>(null);
	let pendingTargetIndex = $state<number | null>(null);

	const resolveInitialState = (): ReaderNavigationState => {
		const pageState = (page.state ?? {}) as ReaderNavigationState;

		if (!browser) return pageState;

		if (Object.keys(pageState).length > 0 && pageState.chapter) {
			sessionStorage.setItem(SESSION_KEYS.readerState, JSON.stringify(pageState));
			return pageState;
		}

		const saved = sessionStorage.getItem(SESSION_KEYS.readerState);
		if (saved) return JSON.parse(saved);

		return pageState;
	};

	let state = $state<ReaderNavigationState>(resolveInitialState());

	$effect(() => {
		const pageState = (page.state ?? {}) as ReaderNavigationState;

		if (Object.keys(pageState).length > 0 && pageState.chapter) {
			state = pageState;
			sessionStorage.setItem(SESSION_KEYS.readerState, JSON.stringify(pageState));
		}
	});

	const chapter = $derived(state.chapter);

	const chapterIndex = $derived.by(() => {
		const value = state.chapterIndex;
		// Stryker disable next-line ConditionalExpression: forcing `typeof value === 'number'` to
		// `true` is equivalent — `Number.isFinite` never returns `true` for a non-number argument
		// (it doesn't coerce), so for any non-number `value` the right operand already evaluates
		// to `false` on its own. Both branches behave identically for any possible input.
		return typeof value === 'number' && Number.isFinite(value) ? Math.max(0, value) : undefined;
	});

	const totalChapters = $derived.by(() => {
		const value = state.totalChapters;
		// Stryker disable next-line ConditionalExpression: same equivalence as the
		// `chapterIndex` clamp above — `Number.isFinite` never returns `true` for a non-number
		// argument, so forcing `typeof value === 'number'` to `true` doesn't change the
		// result for any input.
		return typeof value === 'number' && Number.isFinite(value) ? Math.max(0, value) : undefined;
	});

	const chaptersRemaining = $derived.by(() => {
		if (chapterIndex === undefined || totalChapters === undefined) return undefined;
		return Math.max(0, totalChapters - chapterIndex - 1);
	});

	// Stryker disable next-line ConditionalExpression: forcing `chaptersRemaining !== undefined` to
	// `true` is equivalent — when `chaptersRemaining` really is `undefined`,
	// `chaptersRemaining > 0` becomes `undefined > 0`, which is already `false`, so both branches
	// agree for any input.
	const hasNextChapter = $derived(chaptersRemaining !== undefined && chaptersRemaining > 0);
	// Stryker disable next-line ConditionalExpression: same equivalence as above — `chapterIndex > 0`
	// is already `false` whenever `chapterIndex` is `undefined`, so forcing the left operand to
	// `true` doesn't change the result for any input.
	const hasPreviousChapter = $derived(chapterIndex !== undefined && chapterIndex > 0);

	onMount(() => {
		let unlistenChapters: (() => void) | undefined;
		let unlistenError: (() => void) | undefined;

		const setupListeners = async () => {
			unlistenChapters = await listen<ChapterPayload>(LIBRARY_EVENTS.comicChapters, (event) => {
				const payload = event.payload;

				if (pendingAction === 'load') {
					const items = payload.archive.items;
					const idx = items.findIndex((item) => item.id === state.chapter?.id);

					if (idx !== -1) {
						state.chapterIndex = idx;
						state.totalChapters = payload.archive.total ?? items.length;
						sessionStorage.setItem(SESSION_KEYS.readerState, JSON.stringify(state));
					}

					pendingAction = null;
					return;
				}

				const isPendingNavigate = pendingAction === 'navigate';

				// `pendingTargetIndex` stays inline in the condition (instead of extracted into a
				// named variable like `isPendingNavigate`) on purpose: TypeScript only narrows
				// `pendingTargetIndex` from `number | null` to `number` inside this block through
				// the inline `!== null` here — going through a separate boolean breaks that
				// narrowing and `chapterIndex: pendingTargetIndex` further down (which expects
				// `number | undefined`) stops compiling.
				//
				// Stryker disable next-line ConditionalExpression: `pendingTargetIndex` is only
				// set to a non-null number in the same statement that sets `pendingAction =
				// 'navigate'` (see `navigateToRelativeChapter` below), and every completion path
				// resets both together. So whenever `isPendingNavigate` is true,
				// `pendingTargetIndex !== null` is guaranteed true as well — there is no reachable
				// state where `isPendingNavigate` is true and this check's real value is false,
				// so forcing it to `true` never changes the result.
				if (isPendingNavigate && pendingTargetIndex !== null) {
					const nextChapterData = payload.archive.items[0];

					if (!nextChapterData) {
						initializing = false;
						pendingAction = null;
						pendingTargetIndex = null;
						return;
					}

					const readerChapter: ReaderChapterPayload = {
						id: nextChapterData.id,
						name: nextChapterData.name,
						path: nextChapterData.path,
						chapterSort: nextChapterData.chapterSort ?? '',
						volumeId: nextChapterData.volumeId ?? null,
						volumeName: nextChapterData.volumeName ?? null,
						isSpecial: nextChapterData.isSpecial ?? false,
						lastModified: nextChapterData.lastModified ?? 0
					};

					const newState: ReaderNavigationState = {
						chapter: readerChapter,
						comicDirectoryId: state.comicDirectoryId,
						chapterIndex: pendingTargetIndex,
						totalChapters: state.totalChapters,
						chapterScope: state.chapterScope,
						sortBy: state.sortBy
					};

					state = newState;
					replaceState(page.url, newState);

					initializing = false;
					pendingAction = null;
					pendingTargetIndex = null;
				}
			});

			unlistenError = await listen<any>(LIBRARY_EVENTS.comicChaptersError, () => {
				initializing = false;
				pendingAction = null;
				pendingTargetIndex = null;
			});
		};

		void setupListeners();

		return () => {
			unlistenChapters?.();
			unlistenError?.();
		};
	});

	async function loadContext() {
		if (!chapter || !state.comicDirectoryId) return;

		pendingAction = 'load';

		const sortBy = state.sortBy ?? 'number_asc';

		try {
			await invoke(LIBRARY_COMMANDS.getComicChapters, {
				comicDirectoryFk: state.comicDirectoryId,
				volumeId: null,
				page: 0,
				pageSize: 99999,
				sortBy: sortBy,
				searchQuery: null
			});
		} catch {
			pendingAction = null;
		}
	}

	async function goToNextChapter() {
		// Stryker disable next-line ConditionalExpression: `hasNextChapter` is `$derived` from
		// `chaptersRemaining`, which is only defined when `chapterIndex` is too — so
		// `hasNextChapter === true` already guarantees `chapterIndex !== undefined`. There is no
		// reachable state where `hasNextChapter` is true and `chapterIndex` is undefined, so
		// forcing this check to `false` never changes the result.
		const chapterIndexIsMissing = chapterIndex === undefined;

		if (!hasNextChapter || chapterIndexIsMissing || !state.comicDirectoryId) return;
		await navigateToRelativeChapter(chapterIndex + 1);
	}

	async function goToPreviousChapter() {
		// Stryker disable next-line ConditionalExpression: `hasPreviousChapter` is `$derived` from
		// `chapterIndex !== undefined && chapterIndex > 0`, so `hasPreviousChapter === true` already
		// guarantees `chapterIndex !== undefined`. There is no reachable state where
		// `hasPreviousChapter` is true and `chapterIndex` is undefined, so forcing this check to
		// `false` never changes the result.
		const chapterIndexIsMissing = chapterIndex === undefined;

		if (!hasPreviousChapter || chapterIndexIsMissing || !state.comicDirectoryId) return;
		await navigateToRelativeChapter(chapterIndex - 1);
	}

	async function navigateToRelativeChapter(targetIndex: number) {
		initializing = true;
		pendingAction = 'navigate';
		pendingTargetIndex = targetIndex;

		const sortBy = state.sortBy ?? 'number_asc';

		try {
			await invoke(LIBRARY_COMMANDS.getComicChapters, {
				comicDirectoryFk: state.comicDirectoryId,
				volumeId: null,
				page: targetIndex,
				pageSize: 1,
				sortBy: sortBy,
				searchQuery: null
			});
		} catch {
			initializing = false;
			pendingAction = null;
			pendingTargetIndex = null;
		}
	}

	return {
		get state() {
			return state;
		},
		get initializing() {
			return initializing;
		},
		set initializing(value) {
			initializing = value;
		},
		get chapter() {
			return chapter;
		},
		get chapterIndex() {
			return chapterIndex;
		},
		get totalChapters() {
			return totalChapters;
		},
		get chaptersRemaining() {
			return chaptersRemaining;
		},
		get hasNextChapter() {
			return hasNextChapter;
		},
		get hasPreviousChapter() {
			return hasPreviousChapter;
		},
		loadContext,
		goToNextChapter,
		goToPreviousChapter
	};
}
