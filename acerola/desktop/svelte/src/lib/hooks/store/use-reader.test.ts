import { render } from '@testing-library/svelte';
import { tick } from 'svelte';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { READER_COMMANDS } from '#lib/contracts/reader/reader.commands.ts';
import type {
	ReaderChapterPayload,
	ReaderPagePayload,
	ReaderSessionPayload
} from '#lib/contracts/reader/reader.payloads.ts';
import { useReader } from './use-reader.svelte';
import ReaderHarness from '../../../../tests/harness/hooks/reader-store.svelte';
import { invoke } from '@tauri-apps/api/core';

vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn()
}));

const invokeMock = vi.mocked(invoke);

function chapter(id = 'chapter-1'): ReaderChapterPayload {
	return {
		id,
		name: `Chapter ${id}`,
		path: `C:/mangas/${id}.cbz`,
		chapterSort: '1',
		volumeId: null,
		volumeName: null,
		isSpecial: false,
		lastModified: 0
	};
}

function session(chapterPayload = chapter(), pageCount = 5): ReaderSessionPayload {
	return {
		chapter: chapterPayload,
		pageCount,
		currentPage: 0,
		cacheCapacity: 3
	};
}

function page(chapterId: string, index: number): ReaderPagePayload {
	return {
		chapterId,
		index,
		total: 5,
		mimeType: index % 2 === 0 ? 'image/jpeg' : 'image/png',
		bytes: [index, index + 1],
		cacheHit: false
	};
}

async function renderHook() {
	let reader: ReturnType<typeof useReader> | undefined;

	const result = render(ReaderHarness, {
		props: {
			onReady: (hook) => {
				reader = hook;
			}
		}
	});

	await tick();
	await Promise.resolve();

	return { reader: reader!, unmount: result.unmount };
}

async function waitMicrotasks() {
	await Promise.resolve();
	await tick();
	await Promise.resolve();
}

function calls(command: string) {
	return invokeMock.mock.calls.filter(([calledCommand]) => calledCommand === command);
}

describe('useReader', () => {
	beforeEach(() => {
		vi.clearAllMocks();

		Object.defineProperty(URL, 'createObjectURL', {
			value: vi.fn((blob: Blob) => `blob:${blob.type}:${blob.size}`),
			configurable: true
		});

		Object.defineProperty(URL, 'revokeObjectURL', {
			value: vi.fn(),
			configurable: true
		});

		invokeMock.mockImplementation(async (command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				return session(payload.chapter);
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };
				return page('chapter-1', payload.index);
			}

			return undefined;
		});
	});

	it('starts with no open session and not loading', async () => {
		const { reader } = await renderHook();

		expect(reader.session).toBeUndefined();
		expect(reader.pageCount).toBe(0);
		expect(reader.currentPage).toBe(0);
		expect(reader.loading).toBe(false);
		expect(reader.error).toBeNull();
	});

	it('opens chapter, clamps initial page and loads current page', async () => {
		const { reader } = await renderHook();

		const chapterPayload = chapter();
		const readerSession = await reader.open(chapterPayload, 10);
		await waitMicrotasks();

		expect(readerSession.chapter).toEqual(chapterPayload);
		expect(reader.session?.chapter.id).toBe('chapter-1');
		expect(reader.currentPage).toBe(4);
		expect(reader.current?.index).toBe(4);
		expect(reader.current?.mimeType).toBe('image/jpeg');
		// Real Blob: type and size come from `payload.mimeType`/`payload.bytes`, not from an
		// empty literal — the createObjectURL mock exposes this as `blob:<type>:<size>`.
		expect(reader.current?.url).toBe('blob:image/jpeg:2');
		expect(invokeMock).toHaveBeenCalledWith(READER_COMMANDS.openChapter, {
			chapter: chapterPayload
		});
		expect(invokeMock).toHaveBeenCalledWith(READER_COMMANDS.loadPage, {
			index: 4,
			setCurrent: true
		});
		expect(invokeMock).toHaveBeenCalledWith(READER_COMMANDS.prefetchWindow, {
			center: 4,
			radius: 2
		});
	});

	it('goToPage before any chapter is open does nothing (no session to clamp against)', async () => {
		const { reader } = await renderHook();

		const page = await reader.goToPage(5);

		expect(page).toBeUndefined();
		expect(invokeMock).not.toHaveBeenCalled();
	});

	it('rejects a negative page index and an index at/after pageCount, accepts the last valid index', async () => {
		const { reader } = await renderHook();

		await reader.open(chapter(), 0);
		await waitMicrotasks();
		invokeMock.mockClear();

		expect(await reader.loadPage(-1)).toBeUndefined();
		expect(await reader.loadPage(5)).toBeUndefined();
		expect(invokeMock).not.toHaveBeenCalled();

		// pageCount is 5 (0..4) — the last valid index is the boundary, not pageCount.
		const lastPage = await reader.loadPage(4);
		expect(lastPage?.index).toBe(4);
	});

	it('nextPage/previousPage move relative to currentPage and clamp at the session bounds', async () => {
		const { reader } = await renderHook();

		await reader.open(chapter(), 2);
		await waitMicrotasks();

		await reader.nextPage();
		await waitMicrotasks();
		expect(reader.currentPage).toBe(3);

		await reader.previousPage();
		await waitMicrotasks();
		expect(reader.currentPage).toBe(2);

		await reader.previousPage();
		await reader.previousPage();
		await reader.previousPage();
		await waitMicrotasks();
		// Clamped at 0, never negative.
		expect(reader.currentPage).toBe(0);
	});

	it('reuses cached page without requesting bytes from backend again', async () => {
		const { reader } = await renderHook();

		await reader.open(chapter(), 0);
		await waitMicrotasks();
		invokeMock.mockClear();

		const cachedPage = await reader.loadPage(0);

		expect(cachedPage?.index).toBe(0);
		expect(calls(READER_COMMANDS.loadPage)).toHaveLength(0);
		expect(invokeMock).toHaveBeenCalledWith(READER_COMMANDS.setCurrentPage, { index: 0 });
	});

	it('ignores invalid page without calling backend', async () => {
		const { reader } = await renderHook();

		await reader.open(chapter(), 0);
		await waitMicrotasks();
		invokeMock.mockClear();

		const invalidPage = await reader.loadPage(99);

		expect(invalidPage).toBeUndefined();
		expect(invokeMock).not.toHaveBeenCalled();
	});

	it('records error when opening fails', async () => {
		const { reader } = await renderHook();
		const error = new Error('file not found');

		invokeMock.mockRejectedValueOnce(error);

		await expect(reader.open(chapter())).rejects.toThrow(error);

		expect(reader.error).toBe('file not found');
		expect(reader.loading).toBe(false);
	});

	it('records error when page loading fails', async () => {
		const { reader } = await renderHook();
		const error = new Error('decode failed');

		await reader.open(chapter(), 4);
		await waitMicrotasks();
		invokeMock.mockClear();
		invokeMock.mockRejectedValueOnce(error);

		await expect(reader.loadPage(1)).rejects.toThrow(error);

		expect(reader.error).toBe('decode failed');
		expect(reader.loading).toBe(false);
		expect(invokeMock).toHaveBeenCalledWith(READER_COMMANDS.loadPage, {
			index: 1,
			setCurrent: true
		});
	});

	it('ignores stale page from previous session', async () => {
		const { reader } = await renderHook();
		let activeChapterId = 'chapter-1';
		let resolveStalePage: (payload: ReaderPagePayload) => void = () => {};

		invokeMock.mockImplementation((command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				activeChapterId = payload.chapter.id;
				return Promise.resolve(session(payload.chapter));
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number; setCurrent: boolean };

				if (activeChapterId === 'chapter-1' && payload.index === 1 && payload.setCurrent) {
					return new Promise<ReaderPagePayload>((resolve) => {
						resolveStalePage = resolve;
					});
				}

				return Promise.resolve(page(activeChapterId, payload.index));
			}

			return Promise.resolve(undefined);
		});

		await reader.open(chapter('chapter-1'), 4);
		await waitMicrotasks();

		const staleLoad = reader.loadPage(1);
		await Promise.resolve();

		await reader.open(chapter('chapter-2'), 4);
		await waitMicrotasks();

		resolveStalePage(page('chapter-1', 1));

		await expect(staleLoad).resolves.toBeUndefined();
		expect(reader.session?.chapter.id).toBe('chapter-2');
		expect(reader.currentPage).toBe(4);
		expect(reader.pageAt(1)).toBeUndefined();
	});

	it('prefetches a window around the center, skipping the center itself and already-cached pages', async () => {
		const { reader } = await renderHook();

		// pageCount 10, opens on page 5: expected window = [3,4,6,7] (radius 2, without 5,
		// which already became "current" and is already cached).
		invokeMock.mockImplementation(async (command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				return session(chapter(), 10);
			}
			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };
				return page('chapter-1', payload.index);
			}
			return undefined;
		});

		await reader.open(chapter(), 5);
		await waitMicrotasks();
		await waitMicrotasks();

		const loadedIndices = calls(READER_COMMANDS.loadPage).map(
			([, args]) => (args as { index: number }).index
		);

		expect(loadedIndices).toContain(5); // current
		for (const neighbor of [3, 4, 6, 7]) {
			expect(loadedIndices).toContain(neighbor);
		}
		expect(loadedIndices).not.toContain(2);
		expect(loadedIndices).not.toContain(8);
		expect(invokeMock).toHaveBeenCalledWith(READER_COMMANDS.prefetchWindow, {
			center: 5,
			radius: 2
		});
	});

	it('clamps the prefetch window near the start of the chapter (no negative indices)', async () => {
		const { reader } = await renderHook();

		await reader.open(chapter(), 0);
		await waitMicrotasks();
		await waitMicrotasks();

		const loadedIndices = calls(READER_COMMANDS.loadPage).map(
			([, args]) => (args as { index: number }).index
		);

		// pageCount 5, center 0, radius 2 -> the real window is [0,1,2], never a negative index.
		expect(Math.min(...loadedIndices)).toBe(0);
		expect(loadedIndices).toContain(2);
	});

	it('records an error from setCurrentPage without throwing, still returning the cached page', async () => {
		const { reader } = await renderHook();

		await reader.open(chapter(), 0);
		await waitMicrotasks();
		invokeMock.mockClear();

		const failure = new Error('backend unreachable');
		invokeMock.mockImplementation(async (command) => {
			if (command === READER_COMMANDS.setCurrentPage) throw failure;
			return undefined;
		});

		const cachedPage = await reader.loadPage(0);

		expect(cachedPage?.index).toBe(0);
		expect(reader.error).toBe('backend unreachable');
	});

	it('closes chapter, clears cache and revokes object urls', async () => {
		const { reader } = await renderHook();

		await reader.open(chapter(), 0);
		await waitMicrotasks();

		expect(reader.cacheKeys.length).toBeGreaterThan(0);

		await reader.close();

		expect(invokeMock).toHaveBeenCalledWith(READER_COMMANDS.closeChapter);
		expect(reader.session).toBeUndefined();
		expect(reader.currentPage).toBe(0);
		expect(reader.cacheKeys).toEqual([]);
		expect(reader.loading).toBe(false);
		expect(URL.revokeObjectURL).toHaveBeenCalled();
	});

	it('pageAt returns the cached page for a loaded index', async () => {
		const { reader } = await renderHook();

		await reader.open(chapter(), 0);
		await waitMicrotasks();
		await waitMicrotasks();

		expect(reader.pageAt(0)?.index).toBe(0);
	});

	it('cleans up resources when component unmounts', async () => {
		const { reader, unmount } = await renderHook();

		await reader.open(chapter(), 0);
		await waitMicrotasks();
		invokeMock.mockClear();

		unmount();
		await waitMicrotasks();

		expect(invokeMock).toHaveBeenCalledWith(READER_COMMANDS.closeChapter);
		expect(URL.revokeObjectURL).toHaveBeenCalled();
	});

	it('clears in-flight pending requests on reset, so a later request for the same index is not served the abandoned pending promise', async () => {
		const { reader } = await renderHook();

		let oldRequestIssued = false;

		invokeMock.mockImplementation((command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				return Promise.resolve(session(payload.chapter));
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };

				// Index 4 is outside startPage 0's prefetch radius (the window is
				// [0,1,2]), so it is never touched by the automatic prefetch below.
				if (payload.index === 4 && !oldRequestIssued) {
					oldRequestIssued = true;
					// Never resolved by the test — simulates a request abandoned by a reset.
					return new Promise<ReaderPagePayload>(() => {});
				}

				return Promise.resolve(page('chapter-1', payload.index));
			}

			return Promise.resolve(undefined);
		});

		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();

		// Start an in-flight request for index 4 that will never resolve.
		void reader.loadPage(4, false).catch(() => undefined);
		await Promise.resolve();

		// Reopening resets the pending request map — a later request for the SAME index
		// must not be served the abandoned, forever-pending promise.
		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();
		invokeMock.mockClear();

		void reader.loadPage(4, false);
		await waitMicrotasks();

		expect(calls(READER_COMMANDS.loadPage)).toHaveLength(1);
	});

	it('sets loading synchronously the moment open() is called, before the backend responds', async () => {
		const { reader } = await renderHook();
		let resolveOpenChapter: (value: ReaderSessionPayload) => void = () => {};

		invokeMock.mockImplementation((command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				return new Promise<ReaderSessionPayload>((resolve) => {
					resolveOpenChapter = resolve;
				});
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };
				return Promise.resolve(page('chapter-1', payload.index));
			}

			return Promise.resolve(undefined);
		});

		const openPromise = reader.open(chapter('chapter-1'), 0);

		// Still no await/tick — this must already be true synchronously.
		expect(reader.loading).toBe(true);

		resolveOpenChapter(session(chapter()));
		await openPromise;
		await waitMicrotasks();

		expect(reader.loading).toBe(false);
	});

	it('resets in-flight request bookkeeping before awaiting the new chapter, so a stale response cannot slip through while opening', async () => {
		const { reader } = await renderHook();

		let resolveStalePage: (payload: ReaderPagePayload) => void = () => {};
		let resolveOpenChapter2: (value: ReaderSessionPayload) => void = () => {};

		invokeMock.mockImplementation((command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };

				if (payload.chapter.id === 'chapter-1') {
					return Promise.resolve(session(payload.chapter));
				}

				return new Promise<ReaderSessionPayload>((resolve) => {
					resolveOpenChapter2 = resolve;
				});
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };

				if (payload.index === 2) {
					return new Promise<ReaderPagePayload>((resolve) => {
						resolveStalePage = resolve;
					});
				}

				return Promise.resolve(page('chapter-1', payload.index));
			}

			return Promise.resolve(undefined);
		});

		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();

		// A request unrelated to openChapter starts and stays pending.
		const stalePromise = reader.loadPage(2, true);
		await Promise.resolve();

		// Start opening chapter-2 — openChapter itself is still pending, so everything up
		// to (and including) the reset must already have run synchronously by the time
		// we get back here.
		const openPromise = reader.open(chapter('chapter-2'), 0);
		await Promise.resolve();

		// While openChapter is still being awaited, the OLD pending request resolves.
		resolveStalePage(page('chapter-1', 2));
		await Promise.resolve();
		await Promise.resolve();

		// It must already have been invalidated by the reset that runs BEFORE awaiting
		// openChapter — not just by the later reset that comes after it.
		await expect(stalePromise).resolves.toBeUndefined();
		expect(reader.currentPage).toBe(0);
		expect(reader.pageAt(2)).toBeUndefined();

		resolveOpenChapter2(session(chapter('chapter-2')));
		await openPromise;
		await waitMicrotasks();

		expect(reader.session?.chapter.id).toBe('chapter-2');
	});

	it('applies the session cache capacity from the backend, evicting beyond it (not the default pre-session size)', async () => {
		const { reader } = await renderHook();

		invokeMock.mockImplementation(async (command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				return session(payload.chapter, 10);
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };
				return page('chapter-1', payload.index);
			}

			return undefined;
		});

		await reader.open(chapter(), 5);
		await waitMicrotasks();
		await waitMicrotasks();

		// session.cacheCapacity is 3 (see the `session()` helper); the prefetch loads 5
		// distinct pages (3,4,5,6,7) around center 5 — the cache must have evicted down to
		// the capacity coming from the backend, not the pre-session default of 20.
		expect(reader.cacheKeys.length).toBe(3);
	});

	it('does not sync current page or touch backend setCurrentPage when loading an already-cached page with setCurrent=false', async () => {
		const { reader } = await renderHook();

		await reader.open(chapter(), 0);
		await waitMicrotasks();
		invokeMock.mockClear();

		const before = reader.currentPage;
		const cachedPage = await reader.loadPage(0, false);

		expect(cachedPage?.index).toBe(0);
		expect(reader.currentPage).toBe(before);
		expect(calls(READER_COMMANDS.setCurrentPage)).toHaveLength(0);
	});

	it('reuses the same in-flight promise for concurrent loadPage calls on the same index', async () => {
		const { reader } = await renderHook();
		let resolvePage3: (payload: ReaderPagePayload) => void = () => {};

		invokeMock.mockImplementation((command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				return Promise.resolve(session(payload.chapter, 10));
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };

				if (payload.index === 3) {
					return new Promise<ReaderPagePayload>((resolve) => {
						resolvePage3 = resolve;
					});
				}

				return Promise.resolve(page('chapter-1', payload.index));
			}

			return Promise.resolve(undefined);
		});

		await reader.open(chapter(), 0);
		await waitMicrotasks();
		invokeMock.mockClear();

		const first = reader.loadPage(3, false);
		const second = reader.loadPage(3, false);
		await Promise.resolve();

		expect(calls(READER_COMMANDS.loadPage)).toHaveLength(1);

		resolvePage3(page('chapter-1', 3));

		const [firstResult, secondResult] = await Promise.all([first, second]);

		expect(firstResult).toBe(secondResult);
		expect(firstResult?.index).toBe(3);
	});

	it('discards an in-flight response purely because the request version changed, even when the chapter id still matches', async () => {
		const { reader } = await renderHook();
		let hangingResolve: ((payload: ReaderPagePayload) => void) | undefined;

		invokeMock.mockImplementation((command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				return Promise.resolve(session(payload.chapter, 10));
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };

				if (payload.index === 3 && !hangingResolve) {
					return new Promise<ReaderPagePayload>((resolve) => {
						hangingResolve = resolve;
					});
				}

				return Promise.resolve(page('chapter-1', payload.index));
			}

			return Promise.resolve(undefined);
		});

		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();

		const staleLoad = reader.loadPage(3, false);
		await Promise.resolve();

		// Reopen the SAME chapter id — bumps the request version, but the chapterId (both
		// the one captured at issue time and the current one) stays identical to the
		// pending request's.
		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();
		invokeMock.mockClear();

		// Resolve the request issued BEFORE the reopen with a payload whose chapterId matches
		// perfectly — only the request version counter marks it as stale.
		hangingResolve?.(page('chapter-1', 3));

		await expect(staleLoad).resolves.toBeUndefined();
		expect(reader.pageAt(3)).toBeUndefined();
	});

	it('discards a response whose chapterId does not match the active chapter, even when the request version has not changed', async () => {
		const { reader } = await renderHook();

		invokeMock.mockImplementation(async (command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				return session(payload.chapter, 10);
			}

			if (command === READER_COMMANDS.loadPage) {
				// The backend answers with a payload tagged for a DIFFERENT chapter than the
				// one actually open — simulates a misrouted/crossed backend response.
				const payload = args as { index: number };
				return page('some-other-chapter', payload.index);
			}

			return undefined;
		});

		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();
		invokeMock.mockClear();

		const result = await reader.loadPage(3, false);

		expect(result).toBeUndefined();
		expect(reader.pageAt(3)).toBeUndefined();
	});

	it('does not crash and discards the response when the session is closed while a page request is still in flight', async () => {
		const { reader } = await renderHook();
		let resolvePendingPage: (payload: ReaderPagePayload) => void = () => {};

		invokeMock.mockImplementation((command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				return Promise.resolve(session(payload.chapter, 10));
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };

				if (payload.index === 3) {
					return new Promise<ReaderPagePayload>((resolve) => {
						resolvePendingPage = resolve;
					});
				}

				return Promise.resolve(page('chapter-1', payload.index));
			}

			return Promise.resolve(undefined);
		});

		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();

		const pending = reader.loadPage(3, false);
		await Promise.resolve();

		await reader.close();

		expect(() => resolvePendingPage(page('chapter-1', 3))).not.toThrow();

		await expect(pending).resolves.toBeUndefined();
		expect(reader.session).toBeUndefined();
		expect(reader.pageAt(3)).toBeUndefined();
	});

	it('does not record an error from a stale (superseded) rejected request', async () => {
		const { reader } = await renderHook();
		let rejectPendingPage: (error: Error) => void = () => {};

		invokeMock.mockImplementation((command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				return Promise.resolve(session(payload.chapter, 10));
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };

				if (payload.index === 3) {
					return new Promise<ReaderPagePayload>((_resolve, reject) => {
						rejectPendingPage = reject;
					});
				}

				return Promise.resolve(page('chapter-1', payload.index));
			}

			return Promise.resolve(undefined);
		});

		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();

		const staleLoad = reader.loadPage(3, false).catch(() => undefined);
		await Promise.resolve();

		// Replacement request/version bump (reopening invalidates the pending request).
		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();

		rejectPendingPage(new Error('stale backend failure'));
		await staleLoad;
		await waitMicrotasks();

		expect(reader.error).toBeNull();
	});

	it('does not change currentPage when loading a fresh (non-cached) page with setCurrent=false, but does when setCurrent=true', async () => {
		const { reader } = await renderHook();

		invokeMock.mockImplementation(async (command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				return session(payload.chapter, 10);
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };
				return page('chapter-1', payload.index);
			}

			return undefined;
		});

		await reader.open(chapter(), 0);
		await waitMicrotasks();
		await waitMicrotasks();

		const before = reader.currentPage;
		const loaded = await reader.loadPage(6, false);

		expect(loaded?.index).toBe(6);
		expect(reader.currentPage).toBe(before);

		const loadedCurrent = await reader.loadPage(7, true);

		expect(loadedCurrent?.index).toBe(7);
		expect(reader.currentPage).toBe(7);
	});

	it("does not let a superseded request's cleanup evict a newer in-flight request for the same page index", async () => {
		const { reader } = await renderHook();

		let resolveOldIndex5: (payload: ReaderPagePayload) => void = () => {};
		let resolveNewIndex5: (payload: ReaderPagePayload) => void = () => {};
		let oldRequestIssued = false;

		invokeMock.mockImplementation((command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				return Promise.resolve(session(payload.chapter, 10));
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };

				if (payload.index === 5) {
					if (!oldRequestIssued) {
						oldRequestIssued = true;
						return new Promise<ReaderPagePayload>((resolve) => {
							resolveOldIndex5 = resolve;
						});
					}

					return new Promise<ReaderPagePayload>((resolve) => {
						resolveNewIndex5 = resolve;
					});
				}

				return Promise.resolve(page('chapter-1', payload.index));
			}

			return Promise.resolve(undefined);
		});

		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();

		const oldRequest = reader.loadPage(5, false).catch(() => undefined);
		await Promise.resolve();

		// Reopen the same chapter: bumps the request version and clears the pending map, but
		// the OLD backend call for index 5 keeps running underneath.
		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();

		const newRequest = reader.loadPage(5, false);
		await Promise.resolve();

		invokeMock.mockClear();

		// The stale request (from before the reopen) finally resolves now.
		resolveOldIndex5(page('chapter-1', 5));
		await oldRequest;
		await waitMicrotasks();

		// A third call for the same index, while the NEW request is still pending, must
		// reuse it instead of firing a redundant backend call — proving the stale request's
		// cleanup did not wrongly evict the newer pending entry.
		void reader.loadPage(5, false);
		await waitMicrotasks();

		expect(calls(READER_COMMANDS.loadPage)).toHaveLength(0);

		resolveNewIndex5(page('chapter-1', 5));
		await newRequest;
	});

	it('only clears loading when the completing request is itself tracking the current page (setCurrent=true)', async () => {
		const { reader } = await renderHook();

		let resolveSide: (payload: ReaderPagePayload) => void = () => {};
		let resolveMain: (payload: ReaderPagePayload) => void = () => {};

		invokeMock.mockImplementation((command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				return Promise.resolve(session(payload.chapter, 10));
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };

				if (payload.index === 4) {
					return new Promise<ReaderPagePayload>((resolve) => {
						resolveSide = resolve;
					});
				}

				if (payload.index === 6) {
					return new Promise<ReaderPagePayload>((resolve) => {
						resolveMain = resolve;
					});
				}

				return Promise.resolve(page('chapter-1', payload.index));
			}

			return Promise.resolve(undefined);
		});

		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();

		// Request with setCurrent=false: can never force `loading` true/false on its own.
		const sideRequest = reader.loadPage(4, false);
		await Promise.resolve();
		expect(reader.loading).toBe(false);

		// Request with setCurrent=true: turns loading on.
		const mainRequest = reader.loadPage(6, true);
		await Promise.resolve();
		expect(reader.loading).toBe(true);

		// Completing the setCurrent=false request must NOT clear loading while the
		// setCurrent=true request is still in flight.
		resolveSide(page('chapter-1', 4));
		await sideRequest;
		await waitMicrotasks();
		expect(reader.loading).toBe(true);

		// Completing the setCurrent=true request clears loading.
		resolveMain(page('chapter-1', 6));
		await mainRequest;
		await waitMicrotasks();
		expect(reader.loading).toBe(false);
	});

	it('does not clear loading from a stale (superseded) setCurrent request while a newer open() is still in flight', async () => {
		const { reader } = await renderHook();

		let resolveStalePage: (payload: ReaderPagePayload) => void = () => {};
		let resolveOpenChapter2: (value: ReaderSessionPayload) => void = () => {};

		invokeMock.mockImplementation((command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };

				if (payload.chapter.id === 'chapter-1') {
					return Promise.resolve(session(payload.chapter, 10));
				}

				return new Promise<ReaderSessionPayload>((resolve) => {
					resolveOpenChapter2 = resolve;
				});
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };

				if (payload.index === 4) {
					return new Promise<ReaderPagePayload>((resolve) => {
						resolveStalePage = resolve;
					});
				}

				return Promise.resolve(page('chapter-1', payload.index));
			}

			return Promise.resolve(undefined);
		});

		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();

		const stale = reader.loadPage(4, true).catch(() => undefined);
		await Promise.resolve();

		// Start opening a different chapter — its own setCurrent load stalls mid-await once
		// openChapter resolves, but for now openChapter itself hasn't resolved yet.
		const openPromise = reader.open(chapter('chapter-2'), 0).catch(() => undefined);
		await Promise.resolve();

		// While the open is still in flight, the stale index-4 request from BEFORE resolves.
		// It must not clear `loading` — open() already forced it to true, and a real load is
		// still pending.
		resolveStalePage(page('chapter-1', 4));
		await stale;
		await waitMicrotasks();

		expect(reader.loading).toBe(true);

		resolveOpenChapter2(session(chapter('chapter-2'), 10));
		await openPromise;
		await waitMicrotasks();

		expect(reader.loading).toBe(false);
	});

	it('prefetchWindow does nothing when no chapter is open', async () => {
		const { reader } = await renderHook();

		await expect(reader.prefetchWindow(0)).resolves.toBeUndefined();
		expect(invokeMock).not.toHaveBeenCalled();
	});

	it('records an error when the backend prefetchWindow call rejects', async () => {
		const { reader } = await renderHook();

		// pageCount 3 + radius 2 makes the whole window [0,1,2] — exactly the session's
		// cacheCapacity of 3, so nothing is evicted after the prefetch.
		invokeMock.mockImplementation(async (command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				return session(payload.chapter, 3);
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };
				return page('chapter-1', payload.index);
			}

			return undefined;
		});

		await reader.open(chapter('chapter-1'), 1);
		await waitMicrotasks();
		await waitMicrotasks();

		// By now, every page of the chapter is already cached via the initial prefetch, so
		// a second prefetchWindow call issues no more loadPage calls — isolating the
		// backend prefetchWindow rejection as the only source of `error`.
		invokeMock.mockClear();
		const failure = new Error('prefetch failed');
		invokeMock.mockImplementation(async (command, args) => {
			if (command === READER_COMMANDS.prefetchWindow) {
				throw failure;
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };
				return page('chapter-1', payload.index);
			}

			return undefined;
		});

		await reader.prefetchWindow(1);
		await waitMicrotasks();

		expect(reader.error).toBe('prefetch failed');
		expect(calls(READER_COMMANDS.loadPage)).toHaveLength(0);
	});

	it('prefetch never loads the center index itself, even if it is not yet cached', async () => {
		const { reader } = await renderHook();

		invokeMock.mockImplementation(async (command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				return session(chapter(), 10);
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };
				return page('chapter-1', payload.index);
			}

			return undefined;
		});

		await reader.open(chapter(), 0);
		await waitMicrotasks();
		await waitMicrotasks();
		invokeMock.mockClear();

		// Call prefetchWindow directly with a center never loaded via goToPage, ensuring
		// it is absent from the cache.
		await reader.prefetchWindow(8);
		await waitMicrotasks();

		const loadedIndices = calls(READER_COMMANDS.loadPage).map(
			([, args]) => (args as { index: number }).index
		);

		// The window is [6,7,8,9] (radius 2, capped at pageCount-1=9) minus the center.
		expect(loadedIndices).not.toContain(8);
		expect(loadedIndices).toEqual(expect.arrayContaining([6, 7, 9]));
	});

	it('prefetch loads the current-page index exactly once and skips a repeat prefetch of already-cached indices', async () => {
		const { reader } = await renderHook();

		invokeMock.mockImplementation(async (command, args) => {
			if (command === READER_COMMANDS.openChapter) {
				const payload = args as { chapter: ReaderChapterPayload };
				// The [3,4,5,6,7] window has 5 pages — raise cacheCapacity above that so
				// nothing is evicted, keeping this test focused on the repeat-skipping logic,
				// not on the cache capacity/eviction behavior.
				return { ...session(payload.chapter, 10), cacheCapacity: 10 };
			}

			if (command === READER_COMMANDS.loadPage) {
				const payload = args as { index: number };
				return page('chapter-1', payload.index);
			}

			return undefined;
		});

		await reader.open(chapter('chapter-1'), 5);
		await waitMicrotasks();
		await waitMicrotasks();

		const centerLoads = calls(READER_COMMANDS.loadPage).filter(
			([, args]) => (args as { index: number }).index === 5
		);
		expect(centerLoads).toHaveLength(1);

		invokeMock.mockClear();
		await reader.prefetchWindow(5);
		await waitMicrotasks();

		expect(calls(READER_COMMANDS.loadPage)).toHaveLength(0);
	});

	it("does not trigger prefetch when goToPage's own load is discarded (falsy result)", async () => {
		const { reader } = await renderHook();

		await reader.open(chapter('chapter-1'), 0);
		await waitMicrotasks();
		invokeMock.mockClear();

		invokeMock.mockImplementation(async (command, args) => {
			if (command === READER_COMMANDS.loadPage) {
				// The backend answers for a chapter other than the open one —
				// loadPage discards it and resolves to undefined.
				const payload = args as { index: number };
				return page('some-other-chapter', payload.index);
			}

			return undefined;
		});

		const result = await reader.goToPage(3);

		expect(result).toBeUndefined();
		expect(calls(READER_COMMANDS.prefetchWindow)).toHaveLength(0);
	});
});
