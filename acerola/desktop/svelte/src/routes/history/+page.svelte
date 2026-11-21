<script module lang="ts">
	const handledTerminalSyncLogIds = new Set<number>();
</script>

<script lang="ts">
	import { goto } from '$app/navigation';
	import { getContext, onDestroy, onMount } from 'svelte';
	import { m } from '#lib/paraglide/messages.js';
	import { toast } from 'svelte-sonner';

	import PlaceholderManga from '#lib/assets/placeholder/placeholder_manga.svg?component';
	import AcerolaButton from '#lib/components/acerola-button/acerola-button.svelte';
	import AcerolaEmptyState from '#lib/components/acerola-empty-state/acerola-empty-state.svelte';
	import { extractErrorMessage } from '#lib/utils/error.util.ts';
	import AcerolaBookmarkRibbon from '#lib/components/acerola-bookmark-ribbon/acerola-bookmark-ribbon.svelte';
	import AcerolaAlertDialog from '#lib/components/acerola-alert-dialog/acerola-alert-dialog.svelte';
	import AcerolaCardImage from '#lib/components/acerola-card/acerola-card-image.svelte';
	import AcerolaPopover from '#lib/components/acerola-popover/acerola-popover.svelte';
	import { buttonVariants } from '#lib/components/ui/button/index.ts';
	import { cn } from '#lib/utils/cn.util.ts';
	import { useBookmarks } from '#lib/hooks/store/use-bookmarks.svelte.ts';
	import BookOpen from '@lucide/svelte/icons/book-open';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';
	import MonitorIcon from '@lucide/svelte/icons/monitor';

	import { resolveArtworkPath } from '#lib/utils/artwork.util.ts';
	import type { ReadingHistoryPayload } from '#lib/contracts/history/history.payloads.ts';
	import { useHistory } from '#lib/hooks/store/use-history.svelte.ts';
	import { usePeerConnection, shortId } from '#lib/hooks/store/use-peer-connection.svelte.ts';
	import type { useNetworkSync } from '#lib/hooks/store/use-network-sync.svelte.ts';
	import { CONTEXT_KEYS } from '#lib/constants/context-keys.ts';

	const history = useHistory();
	const bookmarkStore = useBookmarks();
	const peers = usePeerConnection();
	// Shared with `+layout.svelte` (never unmounts) via context — doesn't create its own
	// instance. See `CONTEXT_KEYS.networkSync` / `routes/network/+page.svelte` for why: a
	// page-only instance, unmounted when navigating to another screen, rejected in-flight sync
	// promises (`sync.syncHistory`) with "sync cancelled: listener stopped" even though the
	// real sync kept going in the backend.
	const sync = getContext<ReturnType<typeof useNetworkSync>>(CONTEXT_KEYS.networkSync);

	let syncMenuOpen = $state(false);
	// Avoids re-firing `history.fetch()`/toast for the same event — both on re-render and
	// when navigating away and back to the History screen. Sessions already finished before
	// mount are ignored so no ghost toast shows on startup.
	let lastHandledSyncKey: string | undefined =
		sync.log[0] && sync.log[0].status !== 'started'
			? `${sync.log[0].id}:${sync.log[0].status}`
			: undefined;

	type DisplayPeer = {
		peerId: string;
		deviceName: string | null;
		nickname: string | null;
		connected: boolean;
	};

	// Same merge of persisted paired + live connected peers used in `/network` — see the
	// comment there for why both are needed.
	const uniquePeers = $derived.by(() => {
		const byId = new Map<string, DisplayPeer>();
		for (const peer of peers.pairedPeers) {
			byId.set(peer.peerId, {
				peerId: peer.peerId,
				deviceName: peer.deviceName,
				nickname: peers.peerNicknames[peer.peerId] ?? null,
				connected: false
			});
		}

		for (const peer of peers.status?.peers ?? []) {
			byId.set(peer.peerId, {
				peerId: peer.peerId,
				deviceName: peer.device?.name ?? byId.get(peer.peerId)?.deviceName ?? null,
				nickname: peers.peerNicknames[peer.peerId] ?? null,
				connected: true
			});
		}
		return [...byId.values()];
	});

	const anyHistorySyncing = $derived(
		uniquePeers.some((peer) => sync.isSyncing(peer.peerId, 'history'))
	);

	function peerStatusLabel(peer: DisplayPeer): string {
		if (peer.connected) return m['pages.network.peers.online']();
		const timestamp = sync.lastSyncedAt(peer.peerId);

		if (!timestamp) return m['pages.network.peers.neverSynced']();
		return m['pages.network.peers.lastSynced']({ when: new Date(timestamp).toLocaleString() });
	}

	async function handleSyncHistory(peerId: string) {
		const addrs = peers.getKnownAddr(peerId);
		if (!addrs) return;
		try {
			await sync.syncHistory(peerId, addrs);
		} catch (err) {
			toast.error(extractErrorMessage(err));
		}
	}

	// `sync.syncHistory` only resolves when the connection opens, not when the session actually
	// ends (see `sync:history:*` in the backend) — the real outcome arrives here via
	// `sync.log`, fed by the same events the Network screen listens to.
	$effect(() => {
		const entry = sync.log[0];
		// `id < 0` marks a line loaded from the persisted history (see `fromPersisted` in
		// `use-network-sync.svelte.ts`), not a live event from this session — without this
		// guard, the most recent "complete" history line fired this toast as soon as the
		// History screen mounted, even with no sync actually happening now.
		if (!entry || entry.id < 0 || entry.kind !== 'history') return;

		const currentKey = `${entry.id}:${entry.status}`;
		if (currentKey === lastHandledSyncKey || handledTerminalSyncLogIds.has(entry.id)) {
			return;
		}

		// Same pattern as the Home: one toast per session, created on 'started' and replaced
		// (same `id`) on 'complete'/'error' instead of stacking a new one.
		const toastId = `sync-history-${entry.peerId}`;

		if (entry.status === 'started') {
			lastHandledSyncKey = currentKey;
			toast.loading(
				m['pages.network.transfers.historyStarted']({
					peer: peers.peerLabel(entry.message)
				}),
				{ id: toastId }
			);
			return;
		}

		if (entry.status === 'complete') {
			lastHandledSyncKey = currentKey;
			handledTerminalSyncLogIds.add(entry.id);
			toast.success(
				m['pages.history.sync.success']({ peer: peers.peerLabel(entry.message) }),
				{
					id: toastId,
					duration: 4000
				}
			);
			history.fetch();
			return;
		}

		if (entry.status === 'error') {
			lastHandledSyncKey = currentKey;
			handledTerminalSyncLogIds.add(entry.id);
			toast.error(m['pages.history.sync.error']({ msg: entry.message }), {
				id: toastId,
				duration: 5000
			});
			return;
		}
	});

	function resumeReading(item: ReadingHistoryPayload) {
		goto('/reader', {
			state: {
				comicDirectoryId: item.comicDirectoryId,
				startPage: item.lastPage,
				chapterScope: item.comicName,
				chapter: {
					id: item.chapterArchiveId,
					name: item.chapterName,
					path: item.chapterPath,
					chapterSort: item.chapterSort,
					isSpecial: item.isSpecial,
					lastModified: item.lastModified,
					volumeId: null,
					volumeName: null
				}
			}
		});
	}

	function openComic(item: ReadingHistoryPayload) {
		goto(`/comic/${item.folderName}`, {
			state: {
				comicDirectoryId: item.comicDirectoryId
			}
		});
	}

	onMount(() => {
		history.fetch();
		peers.startListening();
	});

	onDestroy(() => {
		peers.stopListening();
	});
</script>

<div class="flex h-full flex-col overflow-hidden">
	<div class="flex shrink-0 items-center justify-between px-8 py-6">
		<div>
			<p class="text-xs font-semibold text-muted-foreground">
				{m['pages.history.eyebrow']()}
			</p>
			<h2 class="text-[44px] leading-none font-black tracking-tighter">
				{m['pages.history.title']()}
			</h2>
		</div>
		<div class="flex items-center gap-2">
			<AcerolaPopover
				state={{ open: syncMenuOpen }}
				events={{ onOpenChange: (open) => (syncMenuOpen = open) }}
				ui={{
					align: 'end',
					contentClass: 'w-72 p-2'
				}}
			>
				{#snippet trigger()}
					<span class={cn(buttonVariants({ variant: 'secondary' }), 'gap-2')}>
						<RefreshCwIcon size={14} class={anyHistorySyncing ? 'animate-spin' : ''} />
						{m['pages.history.sync.button']()}
					</span>
				{/snippet}

				{#snippet content()}
					<div class="flex flex-col gap-0.5">
						<p class="px-3 pt-1.5 pb-2 text-xs font-semibold text-muted-foreground">
							{m['pages.history.sync.title']()}
						</p>

						{#if uniquePeers.length === 0}
							<p class="px-3 pb-2 text-sm text-muted-foreground">
								{m['pages.history.sync.empty']()}
							</p>
						{:else}
							{#each uniquePeers as peer (peer.peerId)}
								{@const addrs = peers.getKnownAddr(peer.peerId)}
								{@const syncing = sync.isSyncing(peer.peerId, 'history')}
								<button
									type="button"
									class="flex items-center gap-3 rounded-tile p-3 text-left text-sm transition-colors duration-200 ease-brand hover:bg-surface focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none disabled:cursor-not-allowed disabled:opacity-60"
									disabled={!addrs || syncing}
									onclick={() => {
										syncMenuOpen = false;
										handleSyncHistory(peer.peerId);
									}}
								>
									<span
										class="flex size-10 shrink-0 items-center justify-center rounded-full bg-primary-soft text-primary"
									>
										<MonitorIcon size={18} />
									</span>
									<span class="min-w-0 flex-1">
										<span
											class="block truncate font-black tracking-tight text-foreground"
										>
											{peer.nickname ??
												peer.deviceName ??
												shortId(peer.peerId)}
										</span>
										<span class="block truncate text-xs text-muted-foreground">
											{peerStatusLabel(peer)}
										</span>
									</span>
									<RefreshCwIcon
										size={16}
										class={cn(
											'shrink-0 text-muted-foreground',
											syncing && 'animate-spin'
										)}
									/>
								</button>
							{/each}
						{/if}
					</div>
				{/snippet}
			</AcerolaPopover>

			{#if history.items.length > 0}
				<AcerolaAlertDialog
					data={{
						title: m['pages.history.clear.title'](),
						description: m['pages.history.clear.desc'](),
						cancelText: m['pages.history.clear.cancel'](),
						actionText: m['pages.history.clear.confirm']()
					}}
					ui={{ variant: 'destructive' }}
					events={{
						onAction: () => history.clear()
					}}
				>
					<AcerolaButton
						ui={{
							variant: 'destructive',
							class: 'gap-2'
						}}
					>
						<Trash2 size={16} />
						{m['pages.history.clear.button']()}
					</AcerolaButton>
				</AcerolaAlertDialog>
			{/if}
		</div>
	</div>

	{#if history.loading}
		<!-- Loading: skeleton in the shape of the hero and the list. -->
		<div class="flex-1 space-y-6 p-8" role="status" aria-label={m['pages.history.loading']()}>
			<div class="h-100 w-full animate-pulse rounded-b-tile bg-surface"></div>
			{#each Array.from({ length: 4 }, (_, i) => i) as i (i)}
				<div class="h-20 w-full animate-pulse rounded-tile bg-surface"></div>
			{/each}
		</div>
	{:else if history.failed && history.items.length === 0}
		<!-- Error: what happened and what to do. -->
		<div class="p-8">
			<div
				class="flex items-center gap-4 rounded-card bg-destructive-soft p-5 text-destructive"
			>
				<p class="flex-1 text-sm font-semibold">{m['pages.history.error.text']()}</p>
				<AcerolaButton
					ui={{ variant: 'secondary' }}
					events={{ onClick: () => history.fetch() }}
				>
					{m['pages.history.error.retry']()}
				</AcerolaButton>
			</div>
		</div>
	{:else if history.items.length > 0}
		{@const heroItem = history.items[0]}
		{@const heroCover = resolveArtworkPath(heroItem.comicCover)}

		<div class="flex-1 overflow-auto pb-8">
			<div class="relative mb-8 h-100 w-full shrink-0 overflow-hidden rounded-b-tile">
				{#if heroCover}
					<img
						src={heroCover}
						class="absolute inset-0 h-full w-full object-cover"
						referrerpolicy="no-referrer"
						alt=""
					/>
				{:else}
					<div class="absolute inset-0 bg-surface"></div>
				{/if}
				<div
					class="absolute inset-0 bg-linear-to-t from-scrim/80 via-scrim/40 to-transparent"
				></div>
				<div
					class="absolute inset-0 hidden bg-linear-to-l from-transparent via-transparent to-scrim/60 lg:block"
				></div>

				<div class="absolute right-8 bottom-8 left-8 flex items-end gap-8">
					{#if heroCover}
						<button
							class="relative h-64 w-48 overflow-hidden rounded-tile shadow-float transition-transform duration-300 ease-brand hover:scale-105"
							onclick={() => openComic(heroItem)}
						>
							<img
								src={heroCover}
								class="h-full w-full object-cover"
								referrerpolicy="no-referrer"
								alt={heroItem.comicName}
							/>
						</button>
					{:else}
						<button
							class="flex h-64 w-48 items-center justify-center overflow-hidden rounded-tile bg-surface text-muted-foreground transition-transform duration-300 ease-brand hover:scale-105"
							onclick={() => openComic(heroItem)}
						>
							<PlaceholderManga class="h-full w-full" preserveAspectRatio="none" />
						</button>
					{/if}

					<div class="flex-1 space-y-4 pb-2">
						<button
							class="text-left transition-opacity hover:opacity-80"
							onclick={() => openComic(heroItem)}
						>
							<h3 class="text-5xl font-black tracking-tight text-on-scrim">
								{heroItem.comicName}
							</h3>
						</button>

						<div class="flex items-center gap-2 text-lg font-semibold text-on-scrim">
							<BookOpen size={20} />
							<span
								>{m['pages.history.chapterLabel']({
									chapter: heroItem.chapterName
								})}</span
							>
							<span class="mx-2 opacity-50">•</span>
							<span
								>{m['pages.history.pageLabel']({
									page: heroItem.lastPage + 1
								})}</span
							>
						</div>

						<AcerolaButton
							events={{ onClick: () => resumeReading(heroItem) }}
							ui={{
								size: 'lg',
								class: 'mt-4 px-8'
							}}
						>
							{m['pages.history.resume']()}
						</AcerolaButton>
					</div>
				</div>
			</div>

			{#if history.items.length > 1}
				<div class="px-8">
					<h3 class="mb-6 text-lg font-black tracking-tight">
						{m['pages.history.older']()}
					</h3>
					<div class="grid grid-cols-[repeat(auto-fill,minmax(13rem,1fr))] gap-6">
						{#each history.items.slice(1) as item (item.comicDirectoryId)}
							{@const cover = resolveArtworkPath(item.comicCover) || undefined}
							{@const comicBookmark = bookmarkStore.getBookmarkForComic(
								item.comicDirectoryId
							)}
							<AcerolaCardImage
								data={{
									title: item.comicName,
									cover
								}}
								ui={{ class: 'w-full' }}
								events={{ onClick: () => openComic(item) }}
							>
								{#snippet footer()}
									<div class="mt-1 flex flex-col gap-1">
										<span
											class="flex items-center gap-1 text-[10px] font-black tracking-wider text-muted-foreground uppercase"
										>
											<BookOpen size={10} class="shrink-0" />
											<span class="truncate">
												{m['pages.history.chapterLabel']({
													chapter: item.chapterName
												})}
											</span>
										</span>

										<span class="text-[10px] text-muted-foreground opacity-80">
											{m['pages.history.pageLabel']({
												page: item.lastPage + 1
											})}
										</span>
									</div>
								{/snippet}

								{#snippet floatingBadge()}
									{#if comicBookmark != null}
										<AcerolaBookmarkRibbon
											color={comicBookmark.color}
											name={comicBookmark.name}
										/>
									{/if}
								{/snippet}

								{#snippet placeholder()}
									<div class="h-full w-full bg-surface">
										<PlaceholderManga class="h-full w-full" />
									</div>
								{/snippet}
							</AcerolaCardImage>
						{/each}
					</div>
				</div>
			{/if}
		</div>
	{:else}
		<!-- Empty: mascot + title + one phrase + one action. -->
		<AcerolaEmptyState
			data={{ title: m['pages.history.noEvents'](), hint: m['pages.history.emptyHint']() }}
			ui={{ class: 'flex-1 justify-center p-8' }}
		>
			{#snippet action()}
				<AcerolaButton
					ui={{ variant: 'default', class: 'h-12' }}
					events={{ onClick: () => goto('/home') }}
				>
					{m['pages.history.emptyAction']()}
				</AcerolaButton>
			{/snippet}
		</AcerolaEmptyState>
	{/if}
</div>
