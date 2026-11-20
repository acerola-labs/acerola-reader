<script lang="ts" module>
	import type { SyncDirection } from '#lib/hooks/store/use-network-sync.svelte.ts';

	export type ComicPreferencesProps = {
		data?: {
			hasVolumeStructure?: boolean;
			bookmarks?: { id: number; name: string; color: number }[];
			/** Already paired devices (see `usePeerConnection().pairedPeers`), resolved by the
			 *  caller with a display-ready label and the `addrs` already available to fire
			 *  `syncComic` without this screen needing to know the network hook. */
			pairedPeers?: { peerId: string; label: string; addrs: number[] }[];
		};
		state: {
			volumeViewMode: 'cover' | 'banner';
			bookmarkId: number | null;
			externalSyncEnabled: boolean;
			/** `peerId`s with an individual sync running right now (may be more than one in
			 *  parallel, one per device) — only to disable the right list item and show the
			 *  spinner. */
			syncingPeerIds?: string[];
			/** The caller's `useMetadataSync().isSyncing` — true during mangadex/anilist/comicInfo,
			 *  disables the three buttons to avoid a double fire while one is already running. */
			metadataSyncing?: boolean;
			/** Each reflects the loading state of the matching `on*` below — without it the
			 *  refresh icon stays still the whole time, with no feedback that the action is running. */
			rescanning?: boolean;
			regeneratingCover?: boolean;
			regeneratingVolumeCovers?: boolean;
		};
		events: {
			onVolumeViewModeChange: (value: 'cover' | 'banner') => void;
			onBookmarkChange: (value: number | null) => void;
			onExternalSyncChange: (value: boolean) => void;
			onSyncMangadex?: () => void;
			onSyncAnilist?: () => void;
			onSyncComicInfo?: () => void;
			onRescanComic?: () => void;
			onRegenerateCover?: () => void;
			onRegenerateVolumeCovers?: () => void;
			onClearMetadata?: () => Promise<void> | void;
			/** Fires `syncComic` for the chosen `peerId`, in the explicitly clicked direction — see
			 *  `use-network-sync.svelte.ts`. */
			onSyncToDevice?: (
				peerId: string,
				addrs: number[],
				direction: SyncDirection
			) => Promise<void> | void;
		};
	};
</script>

<script lang="ts">
	import { SvelteSet } from 'svelte/reactivity';
	import AcerolaAccordionCard from '#lib/components/acerola-accordion-card/acerola-accordion-card.svelte';
	import AcerolaHeroButton from '#lib/components/acerola-hero-button/acerola-hero-button.svelte';
	import AcerolaSelect from '#lib/components/acerola-select/acerola-select.svelte';
	import AcerolaToggleGroup from '#lib/components/acerola-toggle-group/acerola-toggle-group.svelte';
	import AcerolaToggleCard from '#lib/components/acerola-toggle-card/acerola-toggle-card.svelte';
	import AcerolaAlertDialog from '#lib/components/acerola-alert-dialog/acerola-alert-dialog.svelte';
	import AcerolaButtonIcon from '#lib/components/acerola-button/acerola-button-icon.svelte';
	import AcerolaPopover from '#lib/components/acerola-popover/acerola-popover.svelte';
	import { buttonVariants } from '#lib/components/ui/button/index.ts';
	import { ToggleGroupItem } from '#lib/components/ui/toggle-group/index.ts';
	import { m } from '#lib/paraglide/messages.js';

	import Layers from '@lucide/svelte/icons/layers';
	import Layers2 from '@lucide/svelte/icons/layers-2';
	import Settings2 from '@lucide/svelte/icons/settings-2';
	import BookmarkIcon from '@lucide/svelte/icons/bookmark';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';
	import CloudSync from '@lucide/svelte/icons/cloud-sync';
	import Link from '@lucide/svelte/icons/link';
	import FileText from '@lucide/svelte/icons/file-text';
	import MangaDexIcon from '#lib/assets/icons/mangadex.svg?component';
	import AniListIcon from '#lib/assets/icons/anilist.svg?component';
	import FolderSync from '@lucide/svelte/icons/folder-sync';
	import Image from '@lucide/svelte/icons/image';
	import Eraser from '@lucide/svelte/icons/eraser';
	import Share2 from '@lucide/svelte/icons/share-2';
	import Upload from '@lucide/svelte/icons/upload';
	import Download from '@lucide/svelte/icons/download';
	import Loader2 from '@lucide/svelte/icons/loader-2';
	import MonitorSmartphone from '@lucide/svelte/icons/monitor-smartphone';

	let { data, events, state: preferences }: ComicPreferencesProps = $props();

	let showClearMetadataDialog = $state(false);
	let showPeerMenu = $state(false);

	// 3 categories (Reading / Sync / Advanced) instead of the previous 6 tabs, expanding inline
	// in the list itself — without navigating to another screen. More than one can be open at
	// the same time. Sync is always open (flat, `ui.collapsible: false` below) — it is the most
	// used category (metadata, files and paired devices), so it isn't worth hiding behind a
	// click; Reading and Advanced stay collapsed by default. Same UI/UX mix already used in
	// the config/ root (Appearance/Bookmarks flat, the rest collapsed).
	const expandedCategories = new SvelteSet<string>();

	function toggleCategory(id: string) {
		if (expandedCategories.has(id)) {
			expandedCategories.delete(id);
		} else {
			expandedCategories.add(id);
		}
	}

	// Hover border color of each collapsible category — matches the icon color (same chart-N
	// palette), reinforcing each card's visual identity on hover. Sync isn't included: it is
	// always open, with no click to highlight on hover.
	const CATEGORY_HOVER_BORDER: Record<string, string> = {
		reading: '',
		advanced: ''
	};
</script>

<div class="grid gap-4">
	<!-- Reading -->
	<AcerolaAccordionCard
		data={{
			title: m['pages.comic.preferences.reading'](),
			description: m['pages.comic.preferences.categories.reading.desc']()
		}}
		state={{ expanded: expandedCategories.has('reading') }}
		events={{ onToggle: () => toggleCategory('reading') }}
		ui={{ class: CATEGORY_HOVER_BORDER.reading, iconClass: 'bg-chart-2 text-primary-foreground' }}
	>
		{#snippet icon()}
			<Settings2 size={24} />
		{/snippet}

		{#snippet children()}
			<!-- Volume Highlight (Conditional) -->
			{#if data?.hasVolumeStructure}
				<AcerolaHeroButton
					data={{
						title: m['pages.comic.preferences.volumeHighlight.title'](),
						description: m['pages.comic.preferences.volumeHighlight.desc']()
					}}
					ui={{ iconClass: 'bg-chart-2 text-primary-foreground' }}
				>
					{#snippet icon()}
						<Layers size={24} />
					{/snippet}

					{#snippet action()}
						<AcerolaToggleGroup
							config={{ type: 'single' }}
							state={{ value: preferences.volumeViewMode }}
							events={{
								onValueChange: (value) => {
									if (value === 'cover' || value === 'banner') {
										events.onVolumeViewModeChange(value);
									}
								}
							}}
						>
							{#snippet children()}
								<ToggleGroupItem value="cover" class="px-4 py-1.5 text-[10px] font-black uppercase">
									{m['pages.comic.preferences.volumeHighlight.cover']()}
								</ToggleGroupItem>

								<ToggleGroupItem
									value="banner"
									class="px-4 py-1.5 text-[10px] font-black uppercase"
								>
									{m['pages.comic.preferences.volumeHighlight.banner']()}
								</ToggleGroupItem>
							{/snippet}
						</AcerolaToggleGroup>
					{/snippet}
				</AcerolaHeroButton>
			{/if}

			<!-- Bookmark Assignment -->
			<AcerolaHeroButton
				data={{
					title: m['pages.comic.preferences.bookmark.title'](),
					description: m['pages.comic.preferences.bookmark.desc']()
				}}
				ui={{ iconClass: 'bg-chart-4 text-primary-foreground' }}
			>
				{#snippet icon()}
					<BookmarkIcon size={24} />
				{/snippet}

				{#snippet action()}
					<AcerolaSelect
						data={{
							options: [
								{ value: 'none', label: m['pages.comic.preferences.bookmark.none']() },
								...(data?.bookmarks ?? []).map((b) => ({
									value: b.id.toString(),
									label: b.name,
									color: b.color
								}))
							]
						}}
						state={{ value: preferences.bookmarkId ? preferences.bookmarkId.toString() : 'none' }}
						events={{
							onValueChange: (v) => events.onBookmarkChange(v === 'none' ? null : parseInt(v))
						}}
					/>
				{/snippet}
			</AcerolaHeroButton>
		{/snippet}
	</AcerolaAccordionCard>

	<!-- Sync — always open (flat): metadata, files and paired devices are the most used part
	     of this screen, not worth hiding behind a click. -->
	<AcerolaAccordionCard
		data={{
			title: m['pages.comic.preferences.categories.sync.title'](),
			description: m['pages.comic.preferences.categories.sync.desc']()
		}}
		state={{ expanded: true }}
		events={{ onToggle: () => {} }}
		ui={{ collapsible: false, iconClass: 'bg-chart-1 text-primary-foreground' }}
	>
		{#snippet icon()}
			<CloudSync size={24} />
		{/snippet}

		{#snippet children()}
			<section class="space-y-4">
				<div
					class="flex items-center gap-3 text-xs font-bold tracking-widest text-muted-foreground uppercase"
				>
					<CloudSync size={16} />
					{m['pages.config.metadata.title']()}
				</div>

				<div class="grid gap-4">
					<!-- Toggle + reveals the sync buttons together, in a single card — same shape as
					     the Network screen's "Iroh public network" card. -->
					<AcerolaToggleCard
						data={{
							title: m['pages.comic.preferences.externalSync.title'](),
							subtitle: m['pages.comic.preferences.externalSync.desc']()
						}}
						state={{
							active: preferences.externalSyncEnabled,
							expanded: preferences.externalSyncEnabled
						}}
						events={{
							onClick: () => events.onExternalSyncChange(!preferences.externalSyncEnabled)
						}}
						ui={{ iconClass: 'bg-chart-4 text-primary-foreground' }}
					>
						{#snippet icon()}
							<Link size={18} />
						{/snippet}

						{#snippet children()}
							<AcerolaHeroButton
								data={{
									title: m['pages.config.metadata.mangadex.title'](),
									description: m['pages.config.metadata.mangadex.desc']()
								}}
								events={{
									onClick: preferences.metadataSyncing ? undefined : events.onSyncMangadex
								}}
							>
								{#snippet icon()}
									<span style="all: unset; display: inline-flex;">
										<MangaDexIcon class="h-6 w-6 rounded-inner" />
									</span>
								{/snippet}

								{#snippet action()}
									<AcerolaButtonIcon
										ui={{
											tone: 'accent',
											class: 'rounded-full',
											disabled: preferences.metadataSyncing
										}}
									>
										<RefreshCw class={preferences.metadataSyncing ? 'animate-spin' : ''} />
									</AcerolaButtonIcon>
								{/snippet}
							</AcerolaHeroButton>

							<AcerolaHeroButton
								data={{
									title: m['pages.config.metadata.anilist.title'](),
									description: m['pages.config.metadata.anilist.desc']()
								}}
								events={{
									onClick: preferences.metadataSyncing ? undefined : events.onSyncAnilist
								}}
							>
								{#snippet icon()}
									<span style="all: unset; display: inline-flex;">
										<AniListIcon class="h-6 w-6 rounded-inner" />
									</span>
								{/snippet}

								{#snippet action()}
									<AcerolaButtonIcon
										ui={{
											tone: 'accent',
											class: 'rounded-full',
											disabled: preferences.metadataSyncing
										}}
									>
										<RefreshCw class={preferences.metadataSyncing ? 'animate-spin' : ''} />
									</AcerolaButtonIcon>
								{/snippet}
							</AcerolaHeroButton>
						{/snippet}
					</AcerolaToggleCard>

					<AcerolaHeroButton
						data={{
							title: m['pages.comic.toast.comicInfo.title'](),
							description: m['pages.comic.toast.comicInfo.desc']()
						}}
						events={{
							onClick: preferences.metadataSyncing ? undefined : events.onSyncComicInfo
						}}
						ui={{ iconClass: 'bg-chart-5 text-primary-foreground' }}
					>
						{#snippet icon()}
							<span style="all: unset; display: inline-flex;">
								<FileText class="h-6 w-6" />
							</span>
						{/snippet}

						{#snippet action()}
							<AcerolaButtonIcon
								ui={{
									tone: 'accent',
									class: 'rounded-full',
									disabled: preferences.metadataSyncing
								}}
							>
								<RefreshCw class={preferences.metadataSyncing ? 'animate-spin' : ''} />
							</AcerolaButtonIcon>
						{/snippet}
					</AcerolaHeroButton>
				</div>
			</section>

			<section class="space-y-4">
				<div
					class="flex items-center gap-3 text-xs font-bold tracking-widest text-muted-foreground uppercase"
				>
					<FolderSync size={16} />
					{m['pages.comic.preferences.fileSync.title']()}
				</div>

				<div class="grid gap-4">
					<AcerolaHeroButton
						data={{
							title: m['pages.comic.preferences.fileSync.rescan.title'](),
							description: m['pages.comic.preferences.fileSync.rescan.desc']()
						}}
						events={{ onClick: preferences.rescanning ? undefined : events.onRescanComic }}
						ui={{ iconClass: 'bg-chart-1 text-primary-foreground' }}
					>
						{#snippet icon()}
							<FolderSync size={24} />
						{/snippet}

						{#snippet action()}
							<AcerolaButtonIcon
								ui={{ tone: 'accent', class: 'rounded-full', disabled: preferences.rescanning }}
							>
								<RefreshCw class={preferences.rescanning ? 'animate-spin' : ''} />
							</AcerolaButtonIcon>
						{/snippet}
					</AcerolaHeroButton>
				</div>
			</section>

			<section class="space-y-4">
				<div
					class="flex items-center gap-3 text-xs font-bold tracking-widest text-muted-foreground uppercase"
				>
					<Share2 size={16} />
					{m['pages.comic.preferences.p2pSync.title']()}
				</div>

				<div class="grid gap-4">
					{#if !data?.pairedPeers || data.pairedPeers.length === 0}
						<p class="px-2 text-xs text-muted-foreground">
							{m['pages.comic.preferences.p2pSync.empty']()}
						</p>
					{:else}
						<AcerolaHeroButton
							data={{
								title: m['pages.comic.preferences.p2pSync.send.title'](),
								description: m['pages.comic.preferences.p2pSync.send.desc']()
							}}
							ui={{ iconClass: 'bg-chart-1 text-primary-foreground' }}
						>
							{#snippet icon()}
								<Share2 size={24} />
							{/snippet}

							{#snippet action()}
								<AcerolaPopover
									state={{ open: showPeerMenu }}
									events={{ onOpenChange: (open) => (showPeerMenu = open) }}
									ui={{
										align: 'end',
										contentClass:
											'w-80 overflow-hidden rounded-inner border-border/40 bg-card/95 p-2 shadow-float backdrop-blur-md'
									}}
								>
									{#snippet trigger()}
										<span
											class={buttonVariants({ variant: 'outline', size: 'sm', class: 'gap-2' })}
										>
											<Share2 size={14} />
											{m['pages.comic.preferences.p2pSync.send.button']()}
										</span>
									{/snippet}

									{#snippet content()}
										<div class="flex flex-col gap-2">
											{#each data.pairedPeers as peer (peer.peerId)}
												{@const syncing = (preferences.syncingPeerIds ?? []).includes(peer.peerId)}
												<div
													class="flex items-center gap-3 rounded-inner border border-border/50 bg-muted/30 p-3"
												>
													<div
														class="flex h-10 w-10 shrink-0 items-center justify-center rounded-inner bg-accent-hero text-accent-hero-foreground"
													>
														<MonitorSmartphone size={20} />
													</div>

													<span class="min-w-0 flex-1 truncate text-sm font-semibold"
														>{peer.label}</span
													>

													{#if syncing}
														<Loader2
															size={20}
															class="shrink-0 animate-spin text-muted-foreground"
														/>
													{:else}
														<div class="flex shrink-0 gap-1.5">
															<AcerolaButtonIcon
																ui={{
																	variant: 'ghost',
																	class:
																		'size-9 rounded-inner bg-chart-1 text-primary-foreground transition-colors hover:bg-chart-1/85',
																	title: m['pages.comic.preferences.p2pSync.send.push']()
																}}
																events={{
																	onClick: () => {
																		showPeerMenu = false;
																		events.onSyncToDevice?.(peer.peerId, peer.addrs, 'push');
																	}
																}}
															>
																<Upload size={18} />
															</AcerolaButtonIcon>
															<AcerolaButtonIcon
																ui={{
																	variant: 'ghost',
																	class:
																		'size-9 rounded-inner bg-chart-3 text-primary-foreground transition-colors hover:bg-chart-3/85',
																	title: m['pages.comic.preferences.p2pSync.send.pull']()
																}}
																events={{
																	onClick: () => {
																		showPeerMenu = false;
																		events.onSyncToDevice?.(peer.peerId, peer.addrs, 'pull');
																	}
																}}
															>
																<Download size={18} />
															</AcerolaButtonIcon>
														</div>
													{/if}
												</div>
											{/each}
										</div>
									{/snippet}
								</AcerolaPopover>
							{/snippet}
						</AcerolaHeroButton>
					{/if}
				</div>
			</section>
		{/snippet}
	</AcerolaAccordionCard>

	<!-- Advanced -->
	<AcerolaAccordionCard
		data={{
			title: m['pages.comic.preferences.categories.advanced.title'](),
			description: m['pages.comic.preferences.categories.advanced.desc']()
		}}
		state={{ expanded: expandedCategories.has('advanced') }}
		events={{ onToggle: () => toggleCategory('advanced') }}
		ui={{ class: CATEGORY_HOVER_BORDER.advanced, iconClass: 'bg-chart-3 text-primary-foreground' }}
	>
		{#snippet icon()}
			<Image size={24} />
		{/snippet}

		{#snippet children()}
			<section class="space-y-4">
				<div
					class="flex items-center gap-3 text-xs font-bold tracking-widest text-muted-foreground uppercase"
				>
					<Image size={16} />
					{m['pages.comic.preferences.cover.title']()}
				</div>

				<div class="grid gap-4">
					<AcerolaHeroButton
						data={{
							title: m['pages.comic.preferences.cover.regenerate.title'](),
							description: m['pages.comic.preferences.cover.regenerate.desc']()
						}}
						events={{
							onClick: preferences.regeneratingCover ? undefined : events.onRegenerateCover
						}}
						ui={{ iconClass: 'bg-chart-2 text-primary-foreground' }}
					>
						{#snippet icon()}
							<Image size={24} />
						{/snippet}

						{#snippet action()}
							<AcerolaButtonIcon
								ui={{
									tone: 'accent',
									class: 'rounded-full',
									disabled: preferences.regeneratingCover
								}}
							>
								<RefreshCw class={preferences.regeneratingCover ? 'animate-spin' : ''} />
							</AcerolaButtonIcon>
						{/snippet}
					</AcerolaHeroButton>

					{#if data?.hasVolumeStructure}
						<AcerolaHeroButton
							data={{
								title: m['pages.comic.preferences.cover.regenerateVolumes.title'](),
								description: m['pages.comic.preferences.cover.regenerateVolumes.desc']()
							}}
							events={{
								onClick: preferences.regeneratingVolumeCovers
									? undefined
									: events.onRegenerateVolumeCovers
							}}
							ui={{ iconClass: 'bg-chart-3 text-primary-foreground' }}
						>
							{#snippet icon()}
								<Layers2 size={24} />
							{/snippet}

							{#snippet action()}
								<AcerolaButtonIcon
									ui={{
										tone: 'accent',
										class: 'rounded-full',
										disabled: preferences.regeneratingVolumeCovers
									}}
								>
									<RefreshCw class={preferences.regeneratingVolumeCovers ? 'animate-spin' : ''} />
								</AcerolaButtonIcon>
							{/snippet}
						</AcerolaHeroButton>
					{/if}
				</div>
			</section>

			<section class="space-y-4">
				<div
					class="flex items-center gap-3 text-xs font-bold tracking-widest text-muted-foreground uppercase"
				>
					<Eraser size={16} />
					{m['pages.comic.preferences.dangerZone.title']()}
				</div>

				<div class="grid gap-4">
					<AcerolaHeroButton
						data={{
							title: m['pages.comic.preferences.dangerZone.clearMetadata.title'](),
							description: m['pages.comic.preferences.dangerZone.clearMetadata.desc']()
						}}
						events={{ onClick: () => (showClearMetadataDialog = true) }}
						ui={{ iconClass: 'bg-destructive text-destructive-foreground' }}
					>
						{#snippet icon()}
							<Eraser size={24} />
						{/snippet}
					</AcerolaHeroButton>
				</div>
			</section>
		{/snippet}
	</AcerolaAccordionCard>
</div>

<AcerolaAlertDialog
	state={{ open: showClearMetadataDialog }}
	data={{
		title: m['pages.comic.preferences.dangerZone.clearMetadata.confirm.title'](),
		description: m['pages.comic.preferences.dangerZone.clearMetadata.confirm.desc'](),
		cancelText: m['pages.comic.preferences.dangerZone.clearMetadata.confirm.cancel'](),
		actionText: m['pages.comic.preferences.dangerZone.clearMetadata.confirm.action']()
	}}
	events={{
		onAction: async () => {
			await events.onClearMetadata?.();
			showClearMetadataDialog = false;
		},
		onCancel: () => (showClearMetadataDialog = false)
	}}
	ui={{ variant: 'destructive' }}
/>
