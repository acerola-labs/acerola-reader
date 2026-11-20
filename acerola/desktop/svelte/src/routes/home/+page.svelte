<script module lang="ts">
	const handledTerminalSyncLogIds = new Set<number>();
</script>

<script lang="ts">
	import { goto } from '$app/navigation';
	import PlaceholderManga from '#lib/assets/placeholder/placeholder_manga.svg?component';
	import AcerolaButton from '#lib/components/acerola-button/acerola-button.svelte';
	import AcerolaButtonIcon from '#lib/components/acerola-button/acerola-button-icon.svelte';
	import AcerolaCardImage from '#lib/components/acerola-card/acerola-card-image.svelte';
	import AcerolaBookmarkRibbon from '#lib/components/acerola-bookmark-ribbon/acerola-bookmark-ribbon.svelte';
	import AcerolaEmptyState from '#lib/components/acerola-empty-state/acerola-empty-state.svelte';
	import AcerolaComicActionDialog from './components/acerola-comic-action-dialog.svelte';
	import AcerolaFilterPanel, {
		type BookmarkFilter
	} from './components/acerola-filter-panel.svelte';
	import AcerolaPeerPicker from '#lib/components/acerola-peer-picker/acerola-peer-picker.svelte';
	import AcerolaRemoteLibraryDialog from '#lib/components/acerola-remote-library-dialog/acerola-remote-library-dialog.svelte';
	import { useBookmarks } from '#lib/hooks/store/use-bookmarks.svelte.ts';
	import { useComicSelection } from '#lib/hooks/store/use-comic-selection.svelte.ts';
	import { useSelectFolder } from '#lib/hooks/store/use-select-folder.svelte.ts';
	import { usePeerConnection } from '#lib/hooks/store/use-peer-connection.svelte.ts';
	import type { useNetworkSync } from '#lib/hooks/store/use-network-sync.svelte.ts';
	import { useRemoteLibrary } from '#lib/hooks/store/use-remote-library.svelte.ts';
	import { CONTEXT_KEYS } from '#lib/constants/context-keys.ts';
	import MoreVertical from '@lucide/svelte/icons/more-vertical';
	import BookOpen from '@lucide/svelte/icons/book-open';
	import Check from '@lucide/svelte/icons/check';
	import SlidersHorizontal from '@lucide/svelte/icons/sliders-horizontal';
	import FolderPlus from '@lucide/svelte/icons/folder-plus';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';
	import SearchX from '@lucide/svelte/icons/search-x';
	import MonitorIcon from '@lucide/svelte/icons/monitor';
	import { LIBRARY_EVENTS } from '#lib/contracts/library/library.events.ts';
	import { DIRECTORY_SCAN_COMMANDS } from '#lib/contracts/library/library.commands.ts';
	import { useLibraryScanner } from '#lib/hooks/store/use-comic-scanner.svelte.ts';
	import { useComicSummary } from '#lib/hooks/store/use-comic-summary.svelte.ts';
	import { useComicContext } from '#lib/state/comic-context.svelte.ts';
	import { resolveCover } from '#lib/utils/artwork.util.ts';
	import { listen } from '@tauri-apps/api/event';
	import { getContext, onDestroy, onMount } from 'svelte';
	import { m } from '#lib/paraglide/messages.js';
	import { toast } from 'svelte-sonner';
	import type {
		ComicSummaryItemPayload,
		MetadataSource,
		SortBy,
		SortOrder
	} from '#lib/contracts/home/home.payloads.ts';
	import type { PairedPeerPayload } from '#lib/contracts/network/network.payloads.ts';
	import { extractErrorMessage } from '#lib/utils/error.util.ts';

	const summary = useComicSummary();
	const activeComic = useComicContext();
	const bookmarkStore = useBookmarks();
	const selection = useComicSelection();
	const folderStore = useSelectFolder();
	const peers = usePeerConnection();
	// Shared with `+layout.svelte` (never unmounts) via context — doesn't create its own
	// instance. See `CONTEXT_KEYS.networkSync` / `routes/network/+page.svelte` for why: a
	// page-only instance, unmounted when navigating to another screen, rejected in-flight sync
	// promises with "sync cancelled: listener stopped" even though the real sync kept going in
	// the backend.
	const sync = getContext<ReturnType<typeof useNetworkSync>>(CONTEXT_KEYS.networkSync);
	const remoteLibrary = useRemoteLibrary();

	const refreshScanner = useLibraryScanner(
		DIRECTORY_SCAN_COMMANDS.refreshLibrary,
		() => folderStore.folderPath
	);

	let unlistenScan: (() => void) | undefined;
	let showFilterPanel = $state(false);
	let showActionDialog = $state(false);
	let bookmarkFilter = $state<BookmarkFilter>('all');
	let showPeerPicker = $state(false);
	let browsingPeerId = $state<string | null>(null);
	// Avoids re-firing `summary.fetch()`/toast for the same event — both on re-render and
	// when navigating away and back to Home while a finished session is still the first log
	// line (`sync.log[0]`). Sessions already finished before mount are ignored right away so
	// no ghost toast shows on page startup.
	let lastHandledSyncKey: string | undefined =
		sync.log[0] && sync.log[0].status !== 'started'
			? `${sync.log[0].id}:${sync.log[0].status}`
			: undefined;

	onMount(async () => {
		await folderStore.loadSavedPath();
		await bookmarkStore.loadBookmarks();
		unlistenScan = await listen(LIBRARY_EVENTS.scanComplete, async () => {
			await summary.fetch();
		});

		await summary.fetch();

		peers.startListening();
		remoteLibrary.startListening();
	});

	onDestroy(() => {
		unlistenScan?.();
		peers.stopListening();
		remoteLibrary.stopListening();
	});

	// `sync.syncComic`/`sync.syncFiles` only resolve when the connection opens, not when the
	// session actually ends — the real outcome (and so the moment to reload the local library)
	// arrives here via `sync.log`, fed by the same `sync:comic:*`/`sync:files:*` events the
	// Network screen listens to. `sync:files:*` (bulk sync — may bring whole new comics) must
	// also reload the Home; before, only `'comic'` fired this refresh, leaving the library
	// stale until the user manually navigated away and back.
	$effect(() => {
		const entry = sync.log[0];
		// `id < 0` marks a line loaded from the persisted history (see `fromPersisted` in
		// `use-network-sync.svelte.ts`), not a live event from this session — without this
		// guard, the most recent "complete" history line fired this toast/refresh as soon as
		// the Home mounted, even with no sync actually happening now.
		if (!entry || entry.id < 0 || (entry.kind !== 'comic' && entry.kind !== 'files')) return;

		const currentKey = `${entry.id}:${entry.status}`;
		if (currentKey === lastHandledSyncKey || handledTerminalSyncLogIds.has(entry.id)) {
			return;
		}

		// A single toast per session (kind+peer), created on 'started' and reused (same `id`)
		// on 'complete'/'error' — svelte-sonner replaces the existing toast's content instead of
		// stacking a new one, same pattern as the `toastAsync` already used on the comic screen.
		const toastId = `sync-${entry.kind}-${entry.peerId}`;

		if (entry.status === 'started') {
			lastHandledSyncKey = currentKey;
			const peer = peers.peerLabel(entry.peerId);
			if (entry.kind === 'comic') {
				toast.loading(m['pages.network.transfers.comicStarted']({ peer }), { id: toastId });
			} else {
				toast.loading(m['pages.network.transfers.filesStarted']({ peer }), { id: toastId });
			}
			return;
		}

		if (entry.status === 'complete') {
			lastHandledSyncKey = currentKey;
			handledTerminalSyncLogIds.add(entry.id);
			const peer = peers.peerLabel(entry.peerId);
			const opts = { id: toastId, duration: 4000 };
			if (entry.kind === 'comic') {
				toast.success(m['pages.network.transfers.comicComplete']({ peer }), opts);
			} else {
				toast.success(m['pages.network.transfers.filesComplete']({ peer }), opts);
			}
			summary.fetch();
			return;
		}

		if (entry.status === 'error') {
			lastHandledSyncKey = currentKey;
			handledTerminalSyncLogIds.add(entry.id);
			const msg = entry.message;
			const opts = { id: toastId, duration: 5000 };
			if (entry.kind === 'comic') {
				toast.error(m['pages.network.transfers.comicError']({ msg }), opts);
			} else {
				toast.error(m['pages.network.transfers.filesError']({ msg }), opts);
			}
			// `sync:files:error`/`sync:comic:error` also covers a session that ended with
			// missing chapters (`Ok(skipped) => ... Err(...)` in file_handler.rs/
			// comic_handler.rs) — the chapters/comics ALREADY received successfully before the
			// abort are persisted in the database by the time this event arrives, so without
			// this fetch the Home kept showing the old library until the user navigated away
			// and back (that was exactly the "syncs but the comic doesn't show up" in a large
			// session, where the chance of at least 1 item hitting a race is higher).
			summary.fetch();
			return;
		}
	});

	function selectPeerForBrowsing(peer: PairedPeerPayload) {
		showPeerPicker = false;
		browsingPeerId = peer.peerId;
		remoteLibrary.queryRemoteLibrary(peer.peerId, peer.addrs);
	}

	function syncRemoteComic(comicName: string) {
		if (!browsingPeerId) return;
		const addrs = peers.getKnownAddr(browsingPeerId);
		if (!addrs) return;
		// Comes from remote library browsing (`RemoteLibrarySheet`-like flow) — the user can
		// only pick a comic they don't have yet, so it is always a pull.
		sync
			.syncComic(browsingPeerId, addrs, comicName, 'pull')
			.catch((err) => toast.error(extractErrorMessage(err)));
	}

	async function handleHide(ids: (string | number)[]) {
		const validIds = ids.filter((id) => id != null && String(id).trim() !== '');
		if (validIds.length === 0) return;
		const count = await summary.updateVisibility(validIds, true);

		toast.success(m['pages.home.toast.hidden']({ count }));
		selection.exitSelectionMode();

		showActionDialog = false;
	}

	async function handleDelete(ids: (string | number)[]) {
		const validIds = ids.filter((id) => id != null && String(id).trim() !== '');
		if (validIds.length === 0) return;
		const count = await summary.deleteComics(validIds);

		toast.success(m['pages.home.toast.deleted']({ count }));
		selection.exitSelectionMode();

		showActionDialog = false;
	}

	async function handleClearMetadata(ids: (string | number)[]) {
		const validIds = ids.filter((id) => id != null && String(id).trim() !== '');
		if (validIds.length === 0) return;
		const count = await summary.clearMetadata(validIds);

		toast.success(m['pages.home.toast.metadataCleared']({ count }));
		selection.exitSelectionMode();

		showActionDialog = false;
	}

	async function handleBookmark(ids: (string | number)[], categoryId: number) {
		const validIds = ids.filter((id) => id != null && String(id).trim() !== '');
		if (validIds.length === 0) return;
		let successCount = 0;
		let failCount = 0;

		for (const id of validIds) {
			try {
				await bookmarkStore.assignToComic(id, categoryId);
				successCount++;
			} catch (err) {
				failCount++;
				console.error(`Failed to assign bookmark to comic ${id}:`, err);
			}
		}

		if (successCount > 0) {
			toast.success(m['pages.home.toast.bookmarked']({ count: successCount }));
			await summary.fetch();
		}

		if (failCount > 0) {
			toast.error(m['pages.home.toast.error.bookmark']());
			await summary.fetch();
		}

		selection.exitSelectionMode();
		showActionDialog = false;
	}

	function handleCardClick(comic: ComicSummaryItemPayload, cover: string | null) {
		if (selection.isSelectionMode) {
			selection.toggleSelection(comic.relations.directoryId);
		} else {
			activeComic.set(comic, cover);
			goto(`/comic/${comic.filesystem.folderName}`);
		}
	}

	function handleActionClick(event: MouseEvent, comicId: string | number) {
		event.stopPropagation();
		selection.toggleSelection(comicId);
	}

	function handleSelectAllToggle() {
		const allIds = visibleComics.map((comicItem) => comicItem.relations.directoryId);
		if (selection.selectedCount === allIds.length && allIds.length > 0) {
			selection.deselectAll();
		} else {
			selection.selectAll(allIds);
		}
	}

	function handleFilterApply(params: {
		sortBy: SortBy;
		sortOrder: SortOrder;
		showHidden: boolean;
		metadataSource: MetadataSource;
		bookmarkFilter: BookmarkFilter;
	}) {
		summary.setSorting(params.sortBy, params.sortOrder);
		summary.setFilters(params.showHidden, params.metadataSource);
		bookmarkFilter = params.bookmarkFilter;
		summary.fetch();
		showFilterPanel = false;
	}

	const activeFiltersCount = $derived(
		(summary.showHidden ? 1 : 0) +
			(summary.metadataSource !== 'all' ? 1 : 0) +
			(bookmarkFilter !== 'all' ? 1 : 0)
	);

	const visibleComics = $derived(
		bookmarkFilter === 'all'
			? (summary.comics?.comics ?? [])
			: bookmarkFilter === 'none'
				? (summary.comics?.comics.filter((comic) => comic.bookmark == null) ?? [])
				: (summary.comics?.comics.filter((comic) => comic.bookmark?.id === bookmarkFilter) ?? [])
	);
</script>

{#if summary.loading && (!summary.comics || summary.comics.total === 0)}
	<!-- Loading: skeleton in the shape of the cover grid. -->
	<div class="px-8 pt-8 pb-8" role="status" aria-label={m['pages.home.loading']()}>
		<div class="mb-4 h-10 w-48 animate-pulse rounded-full bg-surface"></div>
		<div class="grid grid-cols-[repeat(auto-fill,minmax(13rem,1fr))] gap-6">
			{#each Array.from({ length: 12 }, (_, i) => i) as i (i)}
				<div class="aspect-2/3 animate-pulse rounded-tile bg-surface"></div>
			{/each}
		</div>
	</div>
{:else if summary.failed && !summary.comics}
	<!-- Error: what happened and what to do. -->
	<div class="p-8">
		<div class="flex items-center gap-4 rounded-card bg-destructive-soft p-5 text-destructive">
			<p class="flex-1 text-sm font-semibold">{m['pages.home.error.text']()}</p>
			<AcerolaButton ui={{ variant: 'secondary', class: 'rounded-full font-black' }} events={{ onClick: () => summary.fetch() }}>
				{m['pages.home.error.retry']()}
			</AcerolaButton>
		</div>
	</div>
{:else if summary.comics && (summary.comics.total > 0 || activeFiltersCount > 0)}
	<div class="px-8 pt-8 pb-8">
		<div class="mb-4 flex items-center justify-between">
			{#if selection.isSelectionMode}
				<div class="flex items-center gap-2">
					<span class="text-sm font-medium text-muted-foreground">
						{m['pages.home.selection.selected']({ count: selection.selectedCount })}
					</span>
					<AcerolaButton
						ui={{ variant: 'ghost', size: 'sm', class: 'rounded-inner' }}
						events={{ onClick: handleSelectAllToggle }}
					>
						{selection.selectedCount === visibleComics.length
							? m['pages.home.selection.all.deselect']()
							: m['pages.home.selection.all.select']()}
					</AcerolaButton>

					<AcerolaButton
						ui={{ variant: 'secondary', size: 'sm', class: 'rounded-inner font-semibold gap-1.5' }}
						events={{ onClick: () => (showActionDialog = true) }}
					>
						<SlidersHorizontal size={14} />
						{m['pages.home.selection.actionsButton']({ count: selection.selectedCount })}
					</AcerolaButton>

					<AcerolaButton
						ui={{ variant: 'ghost', size: 'sm', class: 'rounded-inner' }}
						events={{ onClick: () => selection.exitSelectionMode() }}
					>
						{m['pages.home.selection.cancel']()}
					</AcerolaButton>
				</div>
			{:else}
				<div class="flex items-center gap-2">
					<!-- Filter & Sort Button -->
					<AcerolaButton
						ui={{ variant: 'ghost', class: 'rounded-inner gap-2' }}
						events={{ onClick: () => (showFilterPanel = !showFilterPanel) }}
					>
						<SlidersHorizontal size={16} />
						{m['pages.home.filterButton']()}
						{#if activeFiltersCount > 0}
							<span
								class="flex h-5 min-w-5 items-center justify-center rounded-full bg-primary px-1 text-[10px] font-black text-primary-foreground"
							>
								{activeFiltersCount}
							</span>
						{/if}
					</AcerolaButton>

					<!-- Fetch/sync comics from another paired device -->
					<AcerolaButton
						ui={{ variant: 'ghost', class: 'rounded-inner gap-2' }}
						events={{ onClick: () => (showPeerPicker = true) }}
					>
						<MonitorIcon size={16} />
						{m['pages.home.browseRemoteButton']()}
					</AcerolaButton>

					<!-- Active sort indicator -->
					{#if summary.sortBy !== 'title' || summary.sortOrder !== 'asc'}
						<span
							class="rounded-inner bg-primary/15 px-2.5 py-1 text-[11px] font-semibold text-primary"
						>
							{summary.sortBy === 'title'
								? m['pages.home.sort.indicator.title']()
								: summary.sortBy === 'chapterCount'
									? m['pages.home.sort.indicator.chapterCount']()
									: m['pages.home.sort.indicator.lastUpdated']()}
							{summary.sortOrder === 'asc' ? '↑' : '↓'}
						</span>
					{/if}
				</div>
			{/if}
		</div>

		{#if visibleComics.length === 0}
			<AcerolaEmptyState
				data={{
					title: m['pages.home.noResultsFilteredTitle'](),
					hint: m['pages.home.noResultsFiltered'](),
					icon: SearchX
				}}
				ui={{ class: 'min-h-[40vh] justify-center' }}
			/>
		{/if}

		<div class="grid grid-cols-[repeat(auto-fill,minmax(13rem,1fr))] gap-6">
			{#each visibleComics as comic (comic.relations.directoryId)}
				{@const cover = resolveCover(comic.artwork)}
				{@const bookmarkColor = comic.bookmark?.color}
				{@const bookmarkName = comic.bookmark?.name}
				{@const isSelected = selection.isSelected(comic.relations.directoryId)}
				<AcerolaCardImage
					data={{
						title: comic.metadata.title || comic.filesystem.folderName,
						cover
					}}
					ui={{ class: 'w-full' }}
					events={{
						onClick: () => handleCardClick(comic, cover)
					}}
				>
					{#snippet floatingBadge()}
						{#if bookmarkColor != null}
							<AcerolaBookmarkRibbon
								color={bookmarkColor}
								name={bookmarkName}
								class="-top-1.5 left-5 h-7 w-4"
							/>
						{/if}
					{/snippet}

					{#snippet footer()}
						<div class="mt-1 flex items-center justify-between">
							<span
								class="text-overlay flex items-center gap-1 text-[10px] font-black tracking-wider uppercase"
							>
								<BookOpen size={10} />
								{comic.metadata.chapterCount}
							</span>
						</div>
					{/snippet}

					{#snippet action()}
						<AcerolaButtonIcon
							ui={{
								variant: 'ghost',
								class: 'size-8 translate-x-1.5 -mr-1.5 text-muted-foreground hover:text-foreground'
							}}
							events={{
								onClick: (event) => handleActionClick(event, comic.relations.directoryId)
							}}
						>
							<MoreVertical size={16} />
						</AcerolaButtonIcon>
					{/snippet}

					{#snippet placeholder()}
						<div class="h-full w-full bg-surface">
							<PlaceholderManga class="h-full w-full" />
						</div>
					{/snippet}

					{#snippet overlay()}
						{#if isSelected}
							<div
								class="absolute inset-0 flex items-center justify-center rounded-inner bg-primary/30"
							>
								<div class="rounded-full bg-primary p-2">
									<Check size={24} class="text-primary-foreground" />
								</div>
							</div>
						{:else if selection.isSelectionMode}
							<div
								class="absolute inset-0 flex items-center justify-center rounded-inner bg-surface/50"
							>
								<div class="rounded-full border-2 p-2">
									<div class="h-6 w-6"></div>
								</div>
							</div>
						{/if}
					{/snippet}
				</AcerolaCardImage>
			{/each}
		</div>
	</div>

	<AcerolaComicActionDialog
		state={{ open: showActionDialog }}
		data={{
			selectedIds: selection.selectedIdsArray,
			totalCount: visibleComics.length,
			bookmarks: bookmarkStore.bookmarks
		}}
		events={{
			onHide: handleHide,
			onDelete: handleDelete,
			onClearMetadata: handleClearMetadata,
			onBookmark: handleBookmark,
			onSelectAll: handleSelectAllToggle,
			onClose: () => (showActionDialog = false)
		}}
	/>
{:else}
	<!-- Empty: mascot + title + one phrase + one action (rescan when a folder exists, pick one otherwise). -->
	<AcerolaEmptyState
		data={{ title: m['pages.home.noComics'](), hint: m['pages.home.empty.desc']() }}
		ui={{ class: 'min-h-[60vh] justify-center p-8' }}
	>
		{#snippet action()}
			{#if folderStore.folderPath}
				<AcerolaButton
					ui={{ variant: 'default', class: 'h-12 rounded-full font-black gap-2' }}
					events={{ onClick: () => refreshScanner.start() }}
				>
					<RefreshCw size={18} class={refreshScanner.scanning ? 'animate-spin' : ''} />
					{m['pages.home.empty.quickSync']()}
				</AcerolaButton>
			{:else}
				<AcerolaButton
					ui={{ variant: 'default', class: 'h-12 rounded-full font-black gap-2' }}
					events={{
						onClick: async () => {
							await folderStore.selectFolder();
							await summary.fetch();
						}
					}}
				>
					<FolderPlus size={18} />
					{m['pages.home.empty.selectFolder']()}
				</AcerolaButton>
			{/if}
		{/snippet}
	</AcerolaEmptyState>
{/if}

<!-- Filter Panel (slide-in drawer) -->
<AcerolaFilterPanel
	state={{ open: showFilterPanel }}
	data={{
		sortBy: summary.sortBy,
		sortOrder: summary.sortOrder,
		showHidden: summary.showHidden,
		metadataSource: summary.metadataSource,
		bookmarkFilter,
		bookmarks: bookmarkStore.bookmarks
	}}
	events={{
		onApply: handleFilterApply,
		onClose: () => (showFilterPanel = false)
	}}
/>

<!-- Pick a paired device to fetch comics from -->
<AcerolaPeerPicker
	state={{ open: showPeerPicker }}
	data={{ peers: peers.pairedPeers, nicknameFor: (peerId) => peers.peerNicknames[peerId] }}
	events={{
		onOpenChange: (open) => (showPeerPicker = open),
		onSelect: selectPeerForBrowsing
	}}
/>

<!-- Remote library of the chosen device -->
<AcerolaRemoteLibraryDialog
	state={{ open: browsingPeerId !== null }}
	data={{
		peerLabel: browsingPeerId ? peers.peerLabel(browsingPeerId) : '',
		comics: browsingPeerId ? remoteLibrary.comicsFor(browsingPeerId) : [],
		isLoading: browsingPeerId ? remoteLibrary.isLoading(browsingPeerId) : false,
		errorMessage: browsingPeerId ? remoteLibrary.errorFor(browsingPeerId) : undefined,
		coverPathFor: (comicName) =>
			browsingPeerId ? remoteLibrary.coverPathFor(browsingPeerId, comicName) : undefined,
		isSyncing: () => (browsingPeerId ? sync.isSyncing(browsingPeerId, 'comic') : false)
	}}
	events={{
		onOpenChange: (open) => {
			if (!open) browsingPeerId = null;
		},
		onSelectComic: syncRemoteComic
	}}
/>
