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
	import AcerolaSelect from '#lib/components/acerola-select/acerola-select.svelte';
	import AcerolaAlertDialog from '#lib/components/acerola-alert-dialog/acerola-alert-dialog.svelte';
	import { m } from '#lib/paraglide/messages.js';

	import Layers from '@lucide/svelte/icons/layers';
	import Layers2 from '@lucide/svelte/icons/layers-2';
	import BookmarkIcon from '@lucide/svelte/icons/bookmark';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';
	import Link from '@lucide/svelte/icons/link';
	import FileText from '@lucide/svelte/icons/file-text';
	import MangaDexIcon from '#lib/assets/icons/mangadex.svg?component';
	import AniListIcon from '#lib/assets/icons/anilist.svg?component';
	import FolderSync from '@lucide/svelte/icons/folder-sync';
	import Image from '@lucide/svelte/icons/image';
	import Eraser from '@lucide/svelte/icons/eraser';
	import Upload from '@lucide/svelte/icons/upload';
	import Download from '@lucide/svelte/icons/download';
	import Loader2 from '@lucide/svelte/icons/loader-2';
	import MonitorSmartphone from '@lucide/svelte/icons/monitor-smartphone';
	import AcerolaButton from '#lib/components/acerola-button/acerola-button.svelte';
	import AcerolaEmptyState from '#lib/components/acerola-empty-state/acerola-empty-state.svelte';
	import AcerolaSettingRow from '#lib/components/acerola-setting-row/acerola-setting-row.svelte';
	import AcerolaSettingSection from '#lib/components/acerola-setting-section/acerola-setting-section.svelte';
	import AcerolaSwitch from '#lib/components/acerola-switch/acerola-switch.svelte';

	let { data, events, state: preferences }: ComicPreferencesProps = $props();

	// Same layout as Config and Network: one AcerolaSettingSection per subject, all open, so
	// nothing hides behind a click.
	let showClearMetadataDialog = $state(false);
</script>

<div class="grid gap-6">
	<!-- Reading -->
	<AcerolaSettingSection
		data={{
			id: 'reading',
			title: m['pages.comic.preferences.reading'](),
			lead: m['pages.comic.preferences.categories.reading.desc']()
		}}
	>
		{#if data?.hasVolumeStructure}
			<AcerolaSettingRow
				data={{
					title: m['pages.comic.preferences.volumeHighlight.title'](),
					description: m['pages.comic.preferences.volumeHighlight.desc']()
				}}
			>
				{#snippet icon()}<Layers size={18} />{/snippet}
				{#snippet control()}
					<!-- Brand book chip tabs: track on bg-card, the chosen one in bg-foreground. -->
					<div
						class="flex gap-0.5 rounded-full bg-card p-1"
						role="radiogroup"
						aria-label={m['pages.comic.preferences.volumeHighlight.title']()}
					>
						{#each [{ value: 'cover', label: m['pages.comic.preferences.volumeHighlight.cover']() }, { value: 'banner', label: m['pages.comic.preferences.volumeHighlight.banner']() }] as option (option.value)}
							<button
								type="button"
								role="radio"
								aria-checked={preferences.volumeViewMode === option.value}
								class={[
									'h-9 rounded-full px-4 text-sm font-bold transition-colors duration-300 ease-brand focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none',
									preferences.volumeViewMode === option.value
										? 'bg-foreground text-background'
										: 'text-muted-foreground hover:text-foreground'
								]}
								onclick={() =>
									events.onVolumeViewModeChange(
										option.value as 'cover' | 'banner'
									)}
							>
								{option.label}
							</button>
						{/each}
					</div>
				{/snippet}
			</AcerolaSettingRow>
		{/if}

		<AcerolaSettingRow
			data={{
				title: m['pages.comic.preferences.bookmark.title'](),
				description: m['pages.comic.preferences.bookmark.desc']()
			}}
		>
			{#snippet icon()}<BookmarkIcon size={18} />{/snippet}
			{#snippet control()}
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
					state={{
						value: preferences.bookmarkId ? preferences.bookmarkId.toString() : 'none'
					}}
					events={{
						onValueChange: (v) =>
							events.onBookmarkChange(v === 'none' ? null : parseInt(v))
					}}
				/>
			{/snippet}
		</AcerolaSettingRow>
	</AcerolaSettingSection>

	<!-- Metadata -->
	<AcerolaSettingSection
		data={{
			id: 'metadata',
			title: m['pages.config.metadata.title'](),
			lead: m['pages.comic.preferences.categories.metadata.desc']()
		}}
	>
		<AcerolaSettingRow
			data={{
				title: m['pages.comic.preferences.externalSync.title'](),
				description: m['pages.comic.preferences.externalSync.desc']()
			}}
		>
			{#snippet icon()}<Link size={18} />{/snippet}
			{#snippet control()}
				<AcerolaSwitch
					state={{ checked: preferences.externalSyncEnabled }}
					events={{ onCheckedChange: (checked) => events.onExternalSyncChange(checked) }}
					ui={{ 'aria-label': m['pages.comic.preferences.externalSync.title']() }}
				/>
			{/snippet}
		</AcerolaSettingRow>

		{#if preferences.externalSyncEnabled}
			<AcerolaSettingRow
				data={{
					title: m['pages.config.metadata.mangadex.title'](),
					description: m['pages.config.metadata.mangadex.desc']()
				}}
			>
				{#snippet icon()}<MangaDexIcon class="size-5" />{/snippet}
				{#snippet control()}
					<AcerolaButton
						ui={{
							variant: 'secondary',
							class: 'bg-card',
							disabled: preferences.metadataSyncing
						}}
						events={{ onClick: () => events.onSyncMangadex?.() }}
					>
						<RefreshCw class={preferences.metadataSyncing ? 'animate-spin' : ''} />
						{m['pages.comic.preferences.actions.sync']()}
					</AcerolaButton>
				{/snippet}
			</AcerolaSettingRow>

			<AcerolaSettingRow
				data={{
					title: m['pages.config.metadata.anilist.title'](),
					description: m['pages.config.metadata.anilist.desc']()
				}}
			>
				{#snippet icon()}<AniListIcon class="size-5" />{/snippet}
				{#snippet control()}
					<AcerolaButton
						ui={{
							variant: 'secondary',
							class: 'bg-card',
							disabled: preferences.metadataSyncing
						}}
						events={{ onClick: () => events.onSyncAnilist?.() }}
					>
						<RefreshCw class={preferences.metadataSyncing ? 'animate-spin' : ''} />
						{m['pages.comic.preferences.actions.sync']()}
					</AcerolaButton>
				{/snippet}
			</AcerolaSettingRow>
		{/if}

		<AcerolaSettingRow
			data={{
				title: m['pages.comic.toast.comicInfo.title'](),
				description: m['pages.comic.toast.comicInfo.desc']()
			}}
		>
			{#snippet icon()}<FileText size={18} />{/snippet}
			{#snippet control()}
				<AcerolaButton
					ui={{
						variant: 'secondary',
						class: 'bg-card',
						disabled: preferences.metadataSyncing
					}}
					events={{ onClick: () => events.onSyncComicInfo?.() }}
				>
					<RefreshCw class={preferences.metadataSyncing ? 'animate-spin' : ''} />
					{m['pages.comic.preferences.actions.read']()}
				</AcerolaButton>
			{/snippet}
		</AcerolaSettingRow>
	</AcerolaSettingSection>

	<!-- Files -->
	<AcerolaSettingSection
		data={{
			id: 'files',
			title: m['pages.comic.preferences.fileSync.title'](),
			lead: m['pages.comic.preferences.categories.files.desc']()
		}}
	>
		<AcerolaSettingRow
			data={{
				title: m['pages.comic.preferences.fileSync.rescan.title'](),
				description: m['pages.comic.preferences.fileSync.rescan.desc']()
			}}
		>
			{#snippet icon()}<FolderSync size={18} />{/snippet}
			{#snippet control()}
				<AcerolaButton
					ui={{
						variant: 'secondary',
						class: 'bg-card',
						disabled: preferences.rescanning
					}}
					events={{ onClick: () => events.onRescanComic?.() }}
				>
					<RefreshCw class={preferences.rescanning ? 'animate-spin' : ''} />
					{m['pages.comic.preferences.actions.sync']()}
				</AcerolaButton>
			{/snippet}
		</AcerolaSettingRow>
	</AcerolaSettingSection>

	<!-- Devices -->
	<AcerolaSettingSection
		data={{
			id: 'devices',
			title: m['pages.comic.preferences.p2pSync.title'](),
			lead: m['pages.comic.preferences.categories.devices.desc']()
		}}
	>
		{#if !data?.pairedPeers || data.pairedPeers.length === 0}
			<AcerolaEmptyState
				data={{
					title: m['pages.comic.preferences.p2pSync.emptyTitle'](),
					hint: m['pages.comic.preferences.p2pSync.empty'](),
					icon: MonitorSmartphone
				}}
				ui={{ class: 'py-4' }}
			/>
		{:else}
			{#each data.pairedPeers as peer (peer.peerId)}
				{@const syncing = (preferences.syncingPeerIds ?? []).includes(peer.peerId)}
				<AcerolaSettingRow data={{ title: peer.label, description: null }}>
					{#snippet icon()}
						{#if syncing}
							<Loader2 size={18} class="animate-spin" />
						{:else}
							<MonitorSmartphone size={18} />
						{/if}
					{/snippet}
					{#snippet control()}
						<AcerolaButton
							ui={{
								variant: 'secondary',
								class: 'bg-card',
								disabled: syncing,
								title: m['pages.comic.preferences.p2pSync.send.push']()
							}}
							events={{
								onClick: () =>
									events.onSyncToDevice?.(peer.peerId, peer.addrs, 'push')
							}}
						>
							<Upload size={16} />
							{m['pages.comic.preferences.p2pSync.send.pushShort']()}
						</AcerolaButton>
						<AcerolaButton
							ui={{
								variant: 'secondary',
								class: 'bg-card',
								disabled: syncing,
								title: m['pages.comic.preferences.p2pSync.send.pull']()
							}}
							events={{
								onClick: () =>
									events.onSyncToDevice?.(peer.peerId, peer.addrs, 'pull')
							}}
						>
							<Download size={16} />
							{m['pages.comic.preferences.p2pSync.send.pullShort']()}
						</AcerolaButton>
					{/snippet}
				</AcerolaSettingRow>
			{/each}
		{/if}
	</AcerolaSettingSection>

	<!-- Cover -->
	<AcerolaSettingSection
		data={{
			id: 'cover',
			title: m['pages.comic.preferences.cover.title'](),
			lead: m['pages.comic.preferences.categories.cover.desc']()
		}}
	>
		<AcerolaSettingRow
			data={{
				title: m['pages.comic.preferences.cover.regenerate.title'](),
				description: m['pages.comic.preferences.cover.regenerate.desc']()
			}}
		>
			{#snippet icon()}<Image size={18} />{/snippet}
			{#snippet control()}
				<AcerolaButton
					ui={{
						variant: 'secondary',
						class: 'bg-card',
						disabled: preferences.regeneratingCover
					}}
					events={{ onClick: () => events.onRegenerateCover?.() }}
				>
					<RefreshCw class={preferences.regeneratingCover ? 'animate-spin' : ''} />
					{m['pages.comic.preferences.actions.generate']()}
				</AcerolaButton>
			{/snippet}
		</AcerolaSettingRow>

		{#if data?.hasVolumeStructure}
			<AcerolaSettingRow
				data={{
					title: m['pages.comic.preferences.cover.regenerateVolumes.title'](),
					description: m['pages.comic.preferences.cover.regenerateVolumes.desc']()
				}}
			>
				{#snippet icon()}<Layers2 size={18} />{/snippet}
				{#snippet control()}
					<AcerolaButton
						ui={{
							variant: 'secondary',
							class: 'bg-card',
							disabled: preferences.regeneratingVolumeCovers
						}}
						events={{ onClick: () => events.onRegenerateVolumeCovers?.() }}
					>
						<RefreshCw
							class={preferences.regeneratingVolumeCovers ? 'animate-spin' : ''}
						/>
						{m['pages.comic.preferences.actions.generate']()}
					</AcerolaButton>
				{/snippet}
			</AcerolaSettingRow>
		{/if}
	</AcerolaSettingSection>

	<!-- Danger zone -->
	<AcerolaSettingSection
		data={{
			id: 'danger',
			title: m['pages.comic.preferences.dangerZone.title'](),
			lead: m['pages.comic.preferences.categories.danger.desc']()
		}}
	>
		<AcerolaSettingRow
			data={{
				title: m['pages.comic.preferences.dangerZone.clearMetadata.title'](),
				description: m['pages.comic.preferences.dangerZone.clearMetadata.desc']()
			}}
			ui={{ tone: 'destructive' }}
		>
			{#snippet icon()}<Eraser size={18} />{/snippet}
			{#snippet control()}
				<AcerolaButton
					ui={{ variant: 'destructive' }}
					events={{ onClick: () => (showClearMetadataDialog = true) }}
				>
					{m['pages.comic.preferences.actions.clear']()}
				</AcerolaButton>
			{/snippet}
		</AcerolaSettingRow>
	</AcerolaSettingSection>
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
