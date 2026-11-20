<script lang="ts">
	import { goto, invalidateAll } from '$app/navigation';
	import AcerolaButton from '#lib/components/acerola-button/acerola-button.svelte';
	import AcerolaButtonIcon from '#lib/components/acerola-button/acerola-button-icon.svelte';
	import AcerolaToggleGroup from '#lib/components/acerola-toggle-group/acerola-toggle-group.svelte';
	import AcerolaPopover from '#lib/components/acerola-popover/acerola-popover.svelte';

	import { ToggleGroupItem } from '#lib/components/ui/toggle-group/index.ts';

	import { useVolumeViewMode } from '#lib/hooks/preferences/use-volume-view-mode.svelte.ts';
	import { useComicChapters } from '#lib/hooks/store/use-comic-chapters.svelte.ts';
	import { useBookmarks } from '#lib/hooks/store/use-bookmarks.svelte.ts';
	import { useChapterSelection } from '#lib/hooks/store/use-chapter-selection.svelte.ts';
	import { useHistory } from '#lib/hooks/store/use-history.svelte.ts';
	import { usePeerConnection } from '#lib/hooks/store/use-peer-connection.svelte.ts';
	import type { useNetworkSync, SyncDirection } from '#lib/hooks/store/use-network-sync.svelte.ts';
	import AcerolaPeerPicker from '#lib/components/acerola-peer-picker/acerola-peer-picker.svelte';
	import type { PairedPeerPayload } from '#lib/contracts/network/network.payloads.ts';
	import { CONTEXT_KEYS } from '#lib/constants/context-keys.ts';

	import { useComicContext } from '#lib/state/comic-context.svelte.ts';
	import { useMetadataSync } from '#lib/hooks/store/use-metadata-sync.svelte.ts';

	import { resolveArtworkPath, resolveBanner, resolveCover } from '#lib/utils/artwork.util.ts';
	import { bumpArtworkVersion } from '#lib/state/artwork-version.svelte.ts';
	import type { ReaderChapterPayload } from '#lib/contracts/reader/reader.payloads.ts';
	import { invoke } from '@tauri-apps/api/core';
	import { HOME_COMMANDS } from '#lib/contracts/home/home.commands.ts';
	import { HISTORY_COMMANDS } from '#lib/contracts/history/history.commands.ts';
	import { LIBRARY_COMMANDS } from '#lib/contracts/library/chapter.commands.ts';

	import ArrowLeft from '@lucide/svelte/icons/arrow-left';
	import ArrowUpDown from '@lucide/svelte/icons/arrow-up-down';
	import Check from '@lucide/svelte/icons/check';
	import Share2 from '@lucide/svelte/icons/share-2';

	import { getContext, onMount, untrack } from 'svelte';
	import { fade } from 'svelte/transition';
	import { m } from '#lib/paraglide/messages.js';
	import { extractErrorMessage } from '#lib/utils/error.util.ts';
	import { toastAsync } from '#lib/utils/toast-async.util.ts';
	import { slidingIndicator } from '#lib/utils/sliding-indicator.util.ts';

	import ComicChapterList, { type Chapter } from './components/acerola-comic-chapter-list.svelte';
	import ComicHeroBanner from './components/acerola-comic-hero-banner.svelte';
	import ComicMetadataPanel from './components/acerola-comic-metadata-panel.svelte';
	import ComicPreferences from './components/acerola-comic-preferences.svelte';
	import ComicVolumeList, {
		type VolumeChapter
	} from './components/acerola-comic-volume-list.svelte';

	let { data } = $props();

	const activeComic = useComicContext();
	const chapterStore = useComicChapters();

	// INFO: Chunk size only for render virtualization — it no longer relates to how
	// many chapters are fetched from the backend (that now comes all at once, see
	// use-comic-chapters.svelte.ts).
	const RENDER_CHUNK_SIZE = 25;

	const volumeViewPreference = useVolumeViewMode();
	const bookmarkStore = useBookmarks();
	const metadataSync = useMetadataSync();
	const chapterSelection = useChapterSelection();
	const historyActions = useHistory();
	const peers = usePeerConnection();
	// Shared with `+layout.svelte` (never unmounts) via context — doesn't create its own
	// instance. See `CONTEXT_KEYS.networkSync` / `routes/network/+page.svelte` for why: a
	// page-only instance, unmounted when navigating to another screen, rejected in-flight sync
	// promises (`p2pSync.syncComic`) with "sync cancelled: listener stopped" even though the
	// real sync kept going in the backend.
	const p2pSync = getContext<ReturnType<typeof useNetworkSync>>(CONTEXT_KEYS.networkSync);

	let expandedVolumeId = $state<string | null>(null);
	let currentBookmarkId = $state<number | null>(null);

	let activeTab = $state('content');
	let sortBy = $state<'number_asc' | 'number_desc' | 'modified_asc' | 'modified_desc'>(
		'number_asc'
	);
	let searchQuery = $state('');
	let showSortMenu = $state(false);
	let sendToPeerPickerOpen = $state(false);

	// Bumped by the P2P sync effect below — read (not written) by the chapter fetch effect,
	// only to force a re-fetch when a sync relevant to this comic finishes.
	let syncRefreshTrigger = $state(0);

	// Loading state of each file/cover action — without it the refresh icon had NO visual
	// feedback that the action (which may take a while, e.g. rescanning a large comic) was
	// running.
	let rescanning = $state(false);
	let regeneratingCover = $state(false);
	let regeneratingVolumeCovers = $state(false);

	const onBack = () => window.history.back();

	function toReaderChapter(chapter: Chapter | VolumeChapter): ReaderChapterPayload | null {
		if (!chapter.path) return null;

		return {
			id: chapter.id,
			name: chapter.name ?? chapter.title,
			path: chapter.path,
			chapterSort: chapter.chapterSort ?? '',
			volumeId: chapter.volumeId ?? null,
			volumeName: chapter.volumeName ?? null,
			isSpecial: chapter.isSpecial ?? false,
			lastModified: chapter.lastModified ?? 0
		};
	}

	function getReaderProgress(chapter: Chapter | VolumeChapter) {
		const currentManga = manga;
		if (!currentManga) return {};

		const volume = chapter.volumeId
			? currentManga.volumes.find((item) => item.id === chapter.volumeId)
			: null;

		return {
			chapterIndex: chapter.chapterIndex,
			totalChapters: volume?.totalChapters ?? currentManga.chaptersCount,
			chapterScope: volume?.title ?? currentManga.title,
			sortBy: sortBy
		};
	}

	function openReader(chapter: Chapter | VolumeChapter) {
		const readerChapter = toReaderChapter(chapter);
		if (!readerChapter) return;

		goto('/reader', {
			state: {
				chapter: readerChapter,
				comicDirectoryId:
					activeComic.item?.relations.directoryId ?? data.comic?.relations.directoryId,
				...getReaderProgress(chapter)
			}
		});
	}

	let readingHistory = $state<any | null>(null);
	let readChapters = $state<string[]>([]);
	let isHistoryLoading = $state(false);
	let isSyncing = $state(false);

	async function handleMarkRead(chapter: Chapter | VolumeChapter) {
		const id = comicId;
		if (!id) return;

		await historyActions.markChapterRead(id.toString(), chapter.id);
		if (!readChapters.includes(chapter.id)) {
			readChapters = [...readChapters, chapter.id];
		}
	}

	async function handleMarkUnread(chapter: Chapter | VolumeChapter) {
		const id = comicId;
		if (!id) return;

		await historyActions.unmarkChapterRead(id.toString(), chapter.id);
		readChapters = readChapters.filter((readId) => readId !== chapter.id);
	}

	function handleEnterSelection(chapter: Chapter | VolumeChapter) {
		chapterSelection.selectSingle(chapter.id);
	}

	function handleToggleSelect(chapter: Chapter | VolumeChapter) {
		chapterSelection.toggleSelection(chapter.id);
	}

	async function handleSelectAllChapters() {
		const id = comicId;
		if (!id) return;

		const ids = await invoke<string[]>(LIBRARY_COMMANDS.getComicChapterIds, {
			comicDirectoryFk: id.toString()
		});
		chapterSelection.selectAll(ids);
	}

	async function handleBatchMarkRead() {
		const id = comicId;
		if (!id) return;

		const ids = chapterSelection.selectedIdsArray;
		await historyActions.markChaptersReadBatch(id.toString(), ids);
		const newlyRead = ids.filter((chapterId) => !readChapters.includes(chapterId));
		readChapters = [...readChapters, ...newlyRead];
		chapterSelection.exitSelectionMode();
	}

	async function handleBatchMarkUnread() {
		const id = comicId;
		if (!id) return;

		const ids = chapterSelection.selectedIdsArray;
		await historyActions.unmarkChaptersReadBatch(id.toString(), ids);
		readChapters = readChapters.filter((readId) => !ids.includes(readId));
		chapterSelection.exitSelectionMode();
	}

	/// Sends the FILE(S) of the currently selected chapter(s) to a peer picked in
	/// `AcerolaPeerPicker` — same selection used by `handleBatchMarkRead`/
	/// `handleBatchMarkUnread`, so it works both for a single chapter (selection of 1) and for
	/// several at once. Uses `syncComic` (not `syncHistoryEntry`) scoped to those chapters:
	/// unlike the history push (progress/"read" only), this sends the actual `.cbz`/`.cbr` and
	/// creates the comic on the target if it doesn't exist there yet. Direction is always
	/// `push`: whoever clicks "Send" is sending to the peer, never pulling from it.
	async function handleSendChaptersToPeer(peer: PairedPeerPayload) {
		sendToPeerPickerOpen = false;
		if (!manga?.title) return;

		const chapterIds = chapterSelection.selectedIdsArray;
		chapterSelection.exitSelectionMode();

		try {
			await toastAsync(
				() => p2pSync.syncComic(peer.peerId, peer.addrs, manga!.title, 'push', chapterIds),
				{
					loading: m['pages.comic.selection.sendToPeer.toast.start'](),
					success: m['pages.comic.selection.sendToPeer.toast.success'](),
					error: (err) =>
						m['pages.comic.selection.sendToPeer.toast.error']({ msg: extractErrorMessage(err) })
				}
			);
		} catch {
			// Error already shown by the toastAsync above.
		}
	}

	async function handleSyncMangadex() {
		const id = activeComic.item?.relations.directoryId ?? data.comic?.relations.directoryId;
		if (!id || !manga?.title) return;
		try {
			await toastAsync(() => metadataSync.syncMangadex(manga!.title, id.toString()), {
				loading: m['pages.comic.toast.sync.startMangadex'](),
				success: m['pages.comic.toast.sync.success'](),
				error: (err) => m['pages.comic.toast.mangadexError']({ msg: extractErrorMessage(err) })
			});
			bumpArtworkVersion();
			await invalidateAll();
		} catch {
			// Error already shown by the toastAsync above.
		}
	}

	async function handleSyncAnilist() {
		const id = activeComic.item?.relations.directoryId ?? data.comic?.relations.directoryId;
		if (!id || !manga?.title) return;
		try {
			await toastAsync(() => metadataSync.syncAnilist(manga!.title, id.toString()), {
				loading: m['pages.comic.toast.sync.startAnilist'](),
				success: m['pages.comic.toast.sync.success'](),
				error: (err) => m['pages.comic.toast.anilistError']({ msg: extractErrorMessage(err) })
			});
			bumpArtworkVersion();
			await invalidateAll();
		} catch {
			// Error already shown by the toastAsync above.
		}
	}

	async function handleSyncComicInfo() {
		const id = activeComic.item?.relations.directoryId ?? data.comic?.relations.directoryId;
		if (!id) return;
		try {
			await toastAsync(() => metadataSync.syncComicInfo(id.toString()), {
				loading: m['pages.comic.toast.sync.startComicInfo'](),
				success: m['pages.comic.toast.sync.success'](),
				error: (err) => m['pages.comic.toast.comicInfo.error']({ msg: extractErrorMessage(err) })
			});
			await invalidateAll();
		} catch {
			// Error already shown by the toastAsync above.
		}
	}

	async function handleRescanComic() {
		const id = activeComic.item?.relations.directoryId ?? data.comic?.relations.directoryId;
		if (!id) return;

		rescanning = true;
		try {
			await toastAsync(() => invoke(HOME_COMMANDS.rescanComic, { id: id.toString() }), {
				loading: m['pages.comic.toast.sync.startRescan'](),
				success: m['pages.comic.toast.sync.rescanSuccess'](),
				error: (err) => m['pages.comic.toast.rescanError']({ msg: extractErrorMessage(err) })
			});
			// `invalidateAll()` only reloads SvelteKit's `load()` — the chapter list lives in
			// `chapterStore` (separate Svelte state, with its own LRU cache), which
			// `invalidateAll()` doesn't know exists. Without this, the list kept the old
			// chapters until the user left and came back to the comic.
			chapterStore.invalidate();
			syncRefreshTrigger++;
			bumpArtworkVersion();
			await invalidateAll();
		} catch {
			// Error already shown by the toastAsync above.
		} finally {
			rescanning = false;
		}
	}

	async function handleClearMetadata() {
		const id = activeComic.item?.relations.directoryId ?? data.comic?.relations.directoryId;
		if (!id) return;
		try {
			await toastAsync(() => metadataSync.clearMetadata(id.toString()), {
				loading: m['pages.comic.toast.sync.startClearMetadata'](),
				success: m['pages.comic.toast.sync.clearMetadataSuccess'](),
				error: (err) =>
					m['pages.comic.toast.clearMetadataError']({ msg: extractErrorMessage(err) })
			});
			bumpArtworkVersion();
			await invalidateAll();
		} catch {
			// Error already shown by the toastAsync above.
		}
	}

	async function handleRegenerateCover() {
		const id = activeComic.item?.relations.directoryId ?? data.comic?.relations.directoryId;
		if (!id) return;
		regeneratingCover = true;
		try {
			await toastAsync(() => invoke(HOME_COMMANDS.regenerateComicCover, { id: id.toString() }), {
				loading: m['pages.comic.toast.sync.startRegenerateCover'](),
				success: m['pages.comic.toast.sync.regenerateCoverSuccess'](),
				error: (err) =>
					m['pages.comic.toast.regenerateCoverError']({ msg: extractErrorMessage(err) })
			});
			bumpArtworkVersion();
			await invalidateAll();
		} catch {
			// Error already shown by the toastAsync above.
		} finally {
			regeneratingCover = false;
		}
	}

	async function handleRegenerateVolumeCovers() {
		const id = activeComic.item?.relations.directoryId ?? data.comic?.relations.directoryId;
		if (!id) return;
		regeneratingVolumeCovers = true;
		try {
			await toastAsync(() => invoke(HOME_COMMANDS.regenerateVolumeCovers, { id: id.toString() }), {
				loading: m['pages.comic.toast.sync.startRegenerateVolumeCovers'](),
				success: m['pages.comic.toast.sync.regenerateVolumeCoversSuccess'](),
				error: (err) =>
					m['pages.comic.toast.regenerateVolumeCoversError']({ msg: extractErrorMessage(err) })
			});
			bumpArtworkVersion();
			await invalidateAll();
		} catch {
			// Error already shown by the toastAsync above.
		} finally {
			regeneratingVolumeCovers = false;
		}
	}

	async function handleExternalSyncChange(value: boolean) {
		const id = activeComic.item?.relations.directoryId ?? data.comic?.relations.directoryId;
		if (!id) return;
		try {
			await toastAsync(
				() => invoke(HOME_COMMANDS.toggleComicExternalSync, { id: id.toString(), enabled: value }),
				{
					loading: m['pages.comic.toast.sync.startToggle'](),
					success: value
						? m['pages.comic.toast.sync.enabled']()
						: m['pages.comic.toast.sync.disabled'](),
					error: (err) =>
						m['pages.comic.toast.sync.toggleError']({ msg: extractErrorMessage(err) })
				}
			);
			if (manga) {
				manga.metadata.externalSync = value;
			}
			await invalidateAll();
		} catch {
			// Error already shown by the toastAsync above.
		}
	}

	const pairedPeersForUi = $derived(
		peers.pairedPeers.map((peer) => ({
			peerId: peer.peerId,
			label: peers.peerLabel(peer.peerId),
			addrs: peer.addrs
		}))
	);
	const syncingPeerIds = $derived(
		peers.pairedPeers
			.filter((peer) => p2pSync.isSyncing(peer.peerId, 'comic'))
			.map((peer) => peer.peerId)
	);

	async function handleSyncToDevice(peerId: string, addrs: number[], direction: SyncDirection) {
		if (!manga?.title) return;
		await toastAsync(() => p2pSync.syncComic(peerId, addrs, manga!.title, direction), {
			loading: m['pages.comic.preferences.p2pSync.toast.start'](),
			success: m['pages.comic.preferences.p2pSync.toast.success'](),
			error: (err) =>
				m['pages.comic.preferences.p2pSync.toast.error']({ msg: extractErrorMessage(err) })
		}).catch(() => {
			// Error already shown by the toastAsync above.
		});
	}

	onMount(() => {
		(async () => {
			await Promise.all([
				volumeViewPreference.loadVolumeViewMode(),
				bookmarkStore.loadBookmarks(),
				peers.startListening()
			]);
		})();

		return () => {
			peers.stopListening();
		};
	});

	const comicId = $derived(
		activeComic.item?.relations.directoryId ?? data.comic?.relations.directoryId
	);

	$effect(() => {
		const id = comicId;
		if (!id) return;

		untrack(() => {
			isHistoryLoading = true;
			const fetchHistory = invoke(HISTORY_COMMANDS.getComic, { comicId: id.toString() }).catch(
				() => null
			);
			const fetchRead = invoke<string[]>(HISTORY_COMMANDS.getReadChapters, {
				comicId: id.toString()
			}).catch(() => []);
			const fetchBookmark = bookmarkStore.getComicBookmark(id);

			Promise.all([fetchHistory, fetchRead, fetchBookmark]).then(([history, read, bookmark]) => {
				readingHistory = history;
				readChapters = read;
				currentBookmarkId = bookmark?.id ?? null;
				isHistoryLoading = false;
			});
		});
	});

	function handleReadNow() {
		if (readingHistory) {
			goto('/reader', {
				state: {
					comicDirectoryId: readingHistory.comicDirectoryId.toString(),
					startPage: readingHistory.lastPage,
					chapterScope: readingHistory.comicName,
					chapter: {
						id: readingHistory.chapterArchiveId.toString(),
						name: readingHistory.chapterName,
						path: readingHistory.chapterPath,
						chapterSort: readingHistory.chapterSort,
						isSpecial: readingHistory.isSpecial,
						lastModified: readingHistory.lastModified,
						volumeId: null,
						volumeName: null
					}
				}
			});
		} else {
			// Find first chapter available
			if (manga?.pagesData?.[0]?.items?.[0]) {
				openReader(manga.pagesData[0].items[0] as unknown as Chapter);
			}
		}
	}

	$effect(() => {
		if (data.comic) {
			const comic = data.comic;
			untrack(() => {
				activeComic.set(comic, resolveCover(comic.artwork));
			});
		}
	});

	$effect(() => {
		const value = volumeViewPreference.volumeViewMode;

		untrack(() => {
			volumeViewPreference.saveVolumeViewMode(value);
		});
	});

	$effect(() => {
		const volumeId = expandedVolumeId;
		const query = searchQuery;
		const currentSortBy = sortBy;
		const trigger = syncRefreshTrigger;

		const comic = activeComic.item ?? data.comic;
		if (!comic) return;

		untrack(() => {
			void trigger;
			chapterStore.clear(true);
			chapterStore.fetch(comic.relations.directoryId, currentSortBy, volumeId, query || null);
		});
	});

	// Neither `sync:files:complete` (bulk sync, no comic scope) nor `sync:comic:complete`
	// (scoped, `comicName` in the payload — see `comic_handler.rs::COMPLETE_EVENT`) used to
	// refresh this page: newly arrived chapters, cover/banner and metadata (e.g. a reprocessed
	// `ComicInfo.xml`) stayed stale until the user navigated away and back.
	// `sync:files:complete` doesn't carry which comics were touched, so any bulk sync is
	// treated as potentially relevant; `sync:comic:complete` only fires the refresh if it is
	// about the open comic.
	//
	// The current comic name check (and the firing itself) must stay inside `untrack` — this
	// effect may only depend on `p2pSync.log`. If it read `activeComic.item`/`manga` outside
	// untrack, the very `invalidateAll()`/chapter re-fetch fired here would change those
	// values and re-run the effect again, looping.
	let lastHandledSyncLogId: number | undefined;

	$effect(() => {
		const entry = p2pSync.log[0];
		// `id < 0` marks a line loaded from the persisted history (see `fromPersisted` in
		// `use-network-sync.svelte.ts`), not a live event from this session — without this
		// guard, the most recent "complete" history line fired this refresh as soon as the
		// page mounted, even with no sync actually happening now.
		// `lastHandledSyncLogId` prevents repeated firings or loops on page re-render.
		if (
			!entry ||
			entry.id < 0 ||
			entry.status !== 'complete' ||
			(entry.kind !== 'files' && entry.kind !== 'comic') ||
			entry.id === lastHandledSyncLogId
		)
			return;

		lastHandledSyncLogId = entry.id;

		untrack(() => {
			// `manga.title` is the same value `handleSyncToDevice` sends as `comicName` to the
			// backend (`p2pSync.syncComic(peerId, addrs, manga.title)`) — a symmetric comparison
			// with what `comic_handler.rs` echoes back in the completion payload.
			if (entry.kind === 'comic' && entry.comicName && entry.comicName !== manga?.title) return;

			syncRefreshTrigger++;
			bumpArtworkVersion();
			invalidateAll();
		});
	});

	const manga = $derived.by(() => {
		const item = activeComic.item ?? data.comic;
		if (!item) return null;

		const chaptersData = chapterStore.chapters;

		const allItems = (chaptersData?.archive.items ?? [])
			.map((comic, index) => ({
				id: comic.id.toString(),
				name: comic.name,
				title: comic.name,
				fileName: comic.name,
				isRead: Array.isArray(readChapters) ? readChapters.includes(comic.id.toString()) : false,
				chapterSort: comic.chapterSort,
				path: comic.path,
				volumeId: comic.volumeId,
				volumeName: comic.volumeName,
				isSpecial: comic.isSpecial,
				lastModified: comic.lastModified,
				chapterIndex: index
			}))
			.filter((comic) => !expandedVolumeId || comic.volumeId === expandedVolumeId);

		// Slices the flat list into fixed chunks only for render virtualization
		// (ComicChapterList/ComicVolumeList mount one AcerolaHeroButton per item
		// only when the chunk is near the scroll) — unrelated to the fetch, which
		// already came whole from the backend.
		const pagesData = Array.from(
			{ length: Math.ceil(allItems.length / RENDER_CHUNK_SIZE) || 0 },
			(_, page) => ({
				page,
				items: allItems.slice(page * RENDER_CHUNK_SIZE, (page + 1) * RENDER_CHUNK_SIZE)
			})
		);

		const volumes = (chaptersData?.archive.volumes ?? []).map((volume) => {
			const volCover = volume.coverUri ? resolveArtworkPath(volume.coverUri) : null;
			const volBanner = volume.bannerUri ? resolveArtworkPath(volume.bannerUri) : null;

			const fallbackCover = resolveCover(item.artwork);
			const fallbackBanner = resolveBanner(item.artwork);

			return {
				id: volume.id.toString(),
				title: volume.name,
				totalChapters: volume.chapterCount,
				coverUri: volCover || fallbackCover,
				bannerUri: volBanner || fallbackBanner || fallbackCover,
				hasMore: expandedVolumeId === volume.id.toString() && allItems.length < volume.chapterCount,
				chapters: []
			};
		});

		return {
			id: item.relations.directoryId.toString(),
			title: item.metadata.title || item.filesystem.folderName,
			// The comic's real chapter total (sums volumes) — don't use `totalItems` here,
			// since that value reflects the currently open volume's pagination/filter.
			chaptersCount: item.metadata.chapterCount,
			rating: item.metadata.rating ?? null,
			cover: resolveCover(item.artwork),
			banner: resolveBanner(item.artwork),
			pagesData,
			volumes,
			pageSize: RENDER_CHUNK_SIZE,
			metadata: {
				description:
					item.metadata.description || m['pages.comic.metadata.description.unavailable'](),
				author: item.metadata.author || m['pages.comic.metadata.unknown.author'](),
				status: item.metadata.status || m['pages.comic.metadata.unknown.status'](),
				source: item.metadata.activeSource || 'LOCAL',
				externalSync: item.metadata.externalSync,
				genres: []
			}
		};
	});
</script>

{#if manga}
	<div
		in:fade={{ duration: 200 }}
		out:fade={{ duration: 200 }}
		class="text-text fixed inset-x-0 top-8 bottom-0 z-50 flex overflow-hidden bg-base"
	>
		<div class="pointer-events-none absolute inset-0 overflow-hidden">
			<img
				alt={manga.title}
				src={manga.banner || manga.cover}
				referrerpolicy="no-referrer"
				class="h-full w-full scale-150 object-cover opacity-20 blur-[120px]"
			/>

			<!-- NOTE: This is where the data is injected into the banner -->
			<div class="absolute inset-0 bg-base/40"></div>
		</div>

		<ComicMetadataPanel
			data={{
				title: manga.title,
				author: manga.metadata.author,
				status: manga.metadata.status,
				source: manga.metadata.source,
				chaptersCount: manga.chaptersCount,
				description: manga.metadata.description,
				cover: manga.cover,
				bookmarkColor: bookmarkStore.bookmarks.find((b) => b.id === currentBookmarkId)?.color,
				bookmarkName: bookmarkStore.bookmarks.find((b) => b.id === currentBookmarkId)?.name
			}}
			state={{
				isResuming: !!readingHistory,
				isLoading: isHistoryLoading
			}}
			events={{
				onBack,
				onReadNow: handleReadNow
			}}
		/>

		<div
			class="scrollbar-hide relative z-10 flex flex-1 flex-col overflow-y-auto [overflow-anchor:none]"
		>
			<div
				class="sticky top-0 z-50 flex h-20 items-center justify-between border-b border-surface/30 bg-base/90 px-6 py-2 backdrop-blur-md lg:hidden"
			>
				<AcerolaButtonIcon events={{ onClick: onBack }} ui={{ size: 'sm', tone: 'accent' }}>
					<ArrowLeft size={20} />
				</AcerolaButtonIcon>

				<span class="max-w-50 truncate text-sm font-black tracking-widest uppercase">
					{manga.title}
				</span>

				<div class="w-10"></div>
			</div>

			<ComicHeroBanner
				data={{
					banner: manga.banner,
					genres: manga.metadata.genres,
					rating: manga.rating,
					chapterCount: manga.chaptersCount
				}}
			/>

			<div class="mx-auto w-full max-w-5xl space-y-12 p-8 lg:p-16">
				<div
					class="sticky top-0 z-40 -mx-4 flex items-center justify-between border-b border-surface/30 bg-base/5 px-4 backdrop-blur-3xl"
				>
					<div
						class="relative"
						use:slidingIndicator={{
							selector: '[data-state="on"]',
							indicatorClass: 'bottom-0 h-1 rounded-full bg-primary',
							track: ['x', 'width']
						}}
					>
						<AcerolaToggleGroup
							config={{ type: 'single' }}
							state={{ value: activeTab }}
							events={{
								onValueChange: (value) => {
									if (typeof value === 'string') activeTab = value;
								}
							}}
							ui={{ spacing: 4 }}
						>
							{#snippet children()}
								<ToggleGroupItem
									value="content"
									class="border-none bg-transparent py-6 text-sm font-black tracking-[0.2em] uppercase data-[state=on]:bg-transparent data-[state=on]:text-primary"
								>
									{chapterStore.chapters?.hasVolumeStructure
										? m['pages.comic.tabs.volumes']()
										: m['pages.comic.tabs.chapters']()}
								</ToggleGroupItem>

								<ToggleGroupItem
									value="preferences"
									class="border-none bg-transparent py-6 text-sm font-black tracking-[0.2em] uppercase data-[state=on]:bg-transparent data-[state=on]:text-primary"
								>
									{m['pages.comic.tabs.preferences']()}
								</ToggleGroupItem>
							{/snippet}
						</AcerolaToggleGroup>
					</div>

					{#if activeTab === 'content'}
						<div class="flex items-center gap-2 pr-4">
							{#if !chapterStore.chapters?.hasVolumeStructure}
								<input
									type="text"
									bind:value={searchQuery}
									placeholder={m['pages.comic.filterPlaceholder']()}
									class="w-40 rounded-full border border-surface/30 bg-surface/20 px-6 py-2 text-[10px] font-black tracking-widest transition-all focus:w-60 focus:ring-2 focus:ring-primary/50 focus:outline-none"
								/>
							{/if}

							<AcerolaPopover
								state={{ open: showSortMenu }}
								events={{ onOpenChange: (open) => (showSortMenu = open) }}
								ui={{
									align: 'end',
									side: 'bottom',
									sideOffset: 8,
									contentClass: 'w-64 overflow-hidden rounded-inner p-2'
								}}
							>
								{#snippet trigger()}
									<AcerolaButton ui={{ variant: 'ghost', size: 'sm', class: 'rounded-inner' }}>
										<ArrowUpDown size={16} />
										{m['pages.comic.sort.button']()}
									</AcerolaButton>
								{/snippet}

								{#snippet content()}
									<AcerolaButton
										ui={{ variant: 'ghost', class: 'w-full justify-start rounded-inner' }}
										events={{
											onClick: () => {
												sortBy = 'number_asc';
												showSortMenu = false;
											}
										}}
									>
										{#if sortBy === 'number_asc'}
											<Check size={16} />
										{:else}
											<div class="w-4"></div>
										{/if}
										<span class="min-w-0 flex-1 truncate">{m['pages.comic.sort.number.asc']()}</span
										>
									</AcerolaButton>
									<AcerolaButton
										ui={{ variant: 'ghost', class: 'w-full justify-start rounded-inner' }}
										events={{
											onClick: () => {
												sortBy = 'number_desc';
												showSortMenu = false;
											}
										}}
									>
										{#if sortBy === 'number_desc'}
											<Check size={16} />
										{:else}
											<div class="w-4"></div>
										{/if}
										<span class="min-w-0 flex-1 truncate"
											>{m['pages.comic.sort.number.desc']()}</span
										>
									</AcerolaButton>
									<AcerolaButton
										ui={{ variant: 'ghost', class: 'w-full justify-start rounded-inner' }}
										events={{
											onClick: () => {
												sortBy = 'modified_desc';
												showSortMenu = false;
											}
										}}
									>
										{#if sortBy === 'modified_desc'}
											<Check size={16} />
										{:else}
											<div class="w-4"></div>
										{/if}
										<span class="min-w-0 flex-1 truncate"
											>{m['pages.comic.sort.modified.desc']()}</span
										>
									</AcerolaButton>
									<AcerolaButton
										ui={{ variant: 'ghost', class: 'w-full justify-start rounded-inner' }}
										events={{
											onClick: () => {
												sortBy = 'modified_asc';
												showSortMenu = false;
											}
										}}
									>
										{#if sortBy === 'modified_asc'}
											<Check size={16} />
										{:else}
											<div class="w-4"></div>
										{/if}
										<span class="min-w-0 flex-1 truncate"
											>{m['pages.comic.sort.modified.asc']()}</span
										>
									</AcerolaButton>
								{/snippet}
							</AcerolaPopover>
						</div>
					{/if}
				</div>

				{#if activeTab === 'content' && chapterSelection.isSelectionMode}
					<div class="-mx-4 flex items-center gap-2 border-b border-surface/30 px-4 py-3">
						<span class="text-sm font-medium text-muted-foreground">
							{m['pages.comic.selection.selected']({ count: chapterSelection.selectedCount })}
						</span>
						<AcerolaButton
							ui={{ variant: 'ghost', size: 'sm', class: 'rounded-inner' }}
							events={{ onClick: handleSelectAllChapters }}
						>
							{m['pages.comic.selection.selectAll']()}
						</AcerolaButton>
						<AcerolaButton
							ui={{ variant: 'secondary', size: 'sm', class: 'rounded-inner' }}
							events={{ onClick: handleBatchMarkRead }}
						>
							{m['pages.comic.selection.markRead']()}
						</AcerolaButton>
						<AcerolaButton
							ui={{ variant: 'secondary', size: 'sm', class: 'rounded-inner' }}
							events={{ onClick: handleBatchMarkUnread }}
						>
							{m['pages.comic.selection.markUnread']()}
						</AcerolaButton>
						<AcerolaButton
							ui={{ variant: 'secondary', size: 'sm', class: 'gap-1.5 rounded-inner' }}
							events={{ onClick: () => (sendToPeerPickerOpen = true) }}
						>
							<Share2 size={14} />
							{m['pages.comic.selection.sendToPeer.button']()}
						</AcerolaButton>
						<AcerolaButton
							ui={{ variant: 'ghost', size: 'sm', class: 'rounded-inner' }}
							events={{ onClick: () => chapterSelection.exitSelectionMode() }}
						>
							{m['pages.comic.selection.cancel']()}
						</AcerolaButton>
					</div>
				{/if}

				<AcerolaPeerPicker
					state={{ open: sendToPeerPickerOpen }}
					data={{
						peers: peers.pairedPeers,
						nicknameFor: (peerId) => peers.peerNicknames[peerId]
					}}
					events={{
						onOpenChange: (open) => (sendToPeerPickerOpen = open),
						onSelect: handleSendChaptersToPeer
					}}
				/>

				<div class="min-h-150 pb-8">
					{#if activeTab === 'content'}
						{#if chapterStore.chapters?.hasVolumeStructure}
							<ComicVolumeList
								data={{
									volumes: manga.volumes,
									pagesData: manga.pagesData,
									loading: chapterStore.loading,
									pageSize: manga.pageSize,
									viewMode: volumeViewPreference.volumeViewMode,
									isSelectionMode: chapterSelection.isSelectionMode,
									isSelected: chapterSelection.isSelected
								}}
								events={{
									onExpand: (value) => (expandedVolumeId = value),
									onOpenChapter: openReader,
									onToggleSelect: handleToggleSelect,
									onEnterSelection: handleEnterSelection,
									onMarkRead: handleMarkRead,
									onMarkUnread: handleMarkUnread
								}}
							/>
						{:else}
							<ComicChapterList
								data={{
									pagesData: manga.pagesData,
									totalChapters: manga.chaptersCount,
									pageSize: manga.pageSize,
									isSelectionMode: chapterSelection.isSelectionMode,
									isSelected: chapterSelection.isSelected
								}}
								events={{
									onOpenChapter: openReader,
									onToggleSelect: handleToggleSelect,
									onEnterSelection: handleEnterSelection,
									onMarkRead: handleMarkRead,
									onMarkUnread: handleMarkUnread
								}}
							/>
						{/if}
					{:else if activeTab === 'preferences'}
						<ComicPreferences
							data={{
								hasVolumeStructure: chapterStore.chapters?.hasVolumeStructure ?? false,
								bookmarks: bookmarkStore.bookmarks,
								pairedPeers: pairedPeersForUi
							}}
							state={{
								volumeViewMode: volumeViewPreference.volumeViewMode,
								bookmarkId: currentBookmarkId,
								externalSyncEnabled: manga.metadata.externalSync,
								syncingPeerIds,
								metadataSyncing: metadataSync.isSyncing,
								rescanning,
								regeneratingCover,
								regeneratingVolumeCovers
							}}
							events={{
								onVolumeViewModeChange: (value) => (volumeViewPreference.volumeViewMode = value),
								onBookmarkChange: async (value) => {
									const id =
										activeComic.item?.relations.directoryId ?? data.comic?.relations.directoryId;
									if (!id) return;

									if (value) {
										await bookmarkStore.assignToComic(id, value);
									} else {
										await bookmarkStore.removeComicBookmark(id);
									}
									currentBookmarkId = value;
								},
								onExternalSyncChange: handleExternalSyncChange,
								onSyncMangadex: handleSyncMangadex,
								onSyncAnilist: handleSyncAnilist,
								onSyncComicInfo: handleSyncComicInfo,
								onRescanComic: handleRescanComic,
								onRegenerateCover: handleRegenerateCover,
								onRegenerateVolumeCovers: handleRegenerateVolumeCovers,
								onClearMetadata: handleClearMetadata,
								onSyncToDevice: handleSyncToDevice
							}}
						/>
					{/if}
				</div>
			</div>
		</div>
	</div>
{:else}
	<!-- Loading: skeleton in the shape of the hero and the chapter list, never a loose spinner. -->
	<div class="space-y-6 p-8" role="status" aria-label={m['pages.comic.loading']()}>
		<div class="flex gap-8">
			<div class="aspect-2/3 w-48 animate-pulse rounded-tile bg-surface"></div>
			<div class="flex-1 space-y-4 pt-8">
				<div class="h-12 w-2/3 animate-pulse rounded-full bg-surface"></div>
				<div class="h-5 w-1/3 animate-pulse rounded-full bg-surface"></div>
				<div class="h-20 w-full animate-pulse rounded-tile bg-surface"></div>
			</div>
		</div>
		{#each Array.from({ length: 6 }, (_, i) => i) as i (i)}
			<div class="h-16 w-full animate-pulse rounded-tile bg-surface"></div>
		{/each}
	</div>
{/if}
