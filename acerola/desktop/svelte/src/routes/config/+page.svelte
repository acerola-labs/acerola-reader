<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { fade } from 'svelte/transition';
	import { invoke } from '@tauri-apps/api/core';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { toast } from 'svelte-sonner';
	import AcerolaButton from '#lib/components/acerola-button/acerola-button.svelte';
	import AcerolaCommand from '#lib/components/acerola-command/acerola-command.svelte';
	import AcerolaPopover from '#lib/components/acerola-popover/acerola-popover.svelte';
	import AcerolaSettingRow from '#lib/components/acerola-setting-row/acerola-setting-row.svelte';
	import AcerolaSettingSection from '#lib/components/acerola-setting-section/acerola-setting-section.svelte';
	import AcerolaSwitch from '#lib/components/acerola-switch/acerola-switch.svelte';
	import { Button } from '#lib/components/ui/button/index.ts';
	import * as Command from '#lib/components/ui/command/index.ts';
	import AcerolaBookmarkManager from './components/acerola-bookmark-manager.svelte';
	import { useComicInfoPreference } from '#lib/hooks/preferences/use-comic-info.svelte.ts';
	import { useMetadataLanguage } from '#lib/hooks/preferences/use-metadata-language.svelte.ts';
	import { useLibraryScanner } from '#lib/hooks/store/use-comic-scanner.svelte.ts';
	import { useSelectFolder } from '#lib/hooks/store/use-select-folder.svelte.ts';
	import { DIRECTORY_SCAN_COMMANDS } from '#lib/contracts/library/library.commands.ts';
	import { METADATA_COMMANDS } from '#lib/contracts/metadata/metadata.commands.ts';
	import { notificationStore } from '#lib/components/acerola-notification/acerola-notification.svelte';
	import { extractErrorMessage } from '#lib/utils/error.util.ts';
	import { bumpArtworkVersion } from '#lib/state/artwork-version.svelte.ts';
	import { setSyncingAll } from '#lib/state/metadata-sync-all.svelte.ts';
	import { LANGUAGES, type LanguageCode } from '#lib/constants/languages.ts';
	import { m } from '#lib/paraglide/messages.js';

	import AniListIcon from '#lib/assets/icons/anilist.svg?component';
	import MangaDexIcon from '#lib/assets/icons/mangadex.svg?component';
	import ChevronDownIcon from '@lucide/svelte/icons/chevron-down';
	import ChevronRightIcon from '@lucide/svelte/icons/chevron-right';
	import DatabaseIcon from '@lucide/svelte/icons/database';
	import FileCode2 from '@lucide/svelte/icons/file-code-2';
	import FileTextIcon from '@lucide/svelte/icons/file-text';
	import FolderIcon from '@lucide/svelte/icons/folder';
	import FolderSync from '@lucide/svelte/icons/folder-sync';
	import LanguagesIcon from '@lucide/svelte/icons/languages';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';

	const { notify } = notificationStore;

	const folder = useSelectFolder();
	const comicInfoPreference = useComicInfoPreference();
	const metadataLanguageStore = useMetadataLanguage();

	let metadataLanguagePopoverOpen = $state(false);
	// `metadata:sync_all:complete`/`:error` don't say which source finished (see
	// sync_all_metadata_mangadex/anilist in the backend, which emit the same event for both) —
	// that's why only one "all" sync runs at a time, never mangadex and anilist together.
	let syncingSource = $state<'mangadex' | 'anilist' | null>(null);
	// Mirrors `syncingSource` into a global state — lets the comic screen
	// (`useMetadataSync()`) disable the individual sync while the batch runs, without needing
	// any other sync point between the two screens.
	$effect(() => {
		setSyncingAll(syncingSource !== null);
	});
	// Toast mirroring the same sync — updated in place (same `id`) instead of stacking one
	// toast per progress event. Without it, syncing the whole library's metadata only showed
	// up in the notification bell, not in a toast.
	let syncToastId: string | number | undefined;

	const selectedMetadataLanguageLabel = $derived(
		LANGUAGES.find((lang) => lang.code === metadataLanguageStore.metadataLanguage)?.label ??
			metadataLanguageStore.metadataLanguage
	);

	const refreshScanner = useLibraryScanner(
		DIRECTORY_SCAN_COMMANDS.refreshLibrary,
		() => folder.folderPath
	);

	const rebuildScanner = useLibraryScanner(
		DIRECTORY_SCAN_COMMANDS.rebuildLibrary,
		() => folder.folderPath
	);

	function selectMetadataLanguage(code: LanguageCode) {
		metadataLanguageStore.selectMetadataLanguage(code);
		metadataLanguagePopoverOpen = false;
	}

	async function handleSyncAllMangadex() {
		if (syncingSource) return;
		syncingSource = 'mangadex';
		try {
			notify.info(m['pages.config.toast.sync.mangadex.start'](), { duration: 0 });
			syncToastId = toast.loading(m['pages.config.toast.sync.mangadex.start']());
			await invoke(METADATA_COMMANDS.syncAllMangadex, {
				language: metadataLanguageStore.metadataLanguage,
				generateComicInfo: comicInfoPreference.comicInfoPreference ?? false
			});
		} catch (error: unknown) {
			syncingSource = null;
			const msg = extractErrorMessage(error);
			notify.error(m['pages.config.toast.sync.mangadex.error']({ msg }), { duration: 0 });
			toast.error(m['pages.config.toast.sync.mangadex.error']({ msg }), { id: syncToastId });
			syncToastId = undefined;
		}
	}

	async function handleSyncAllAnilist() {
		if (syncingSource) return;
		syncingSource = 'anilist';
		try {
			notify.info(m['pages.config.toast.sync.anilist.start'](), { duration: 0 });
			syncToastId = toast.loading(m['pages.config.toast.sync.anilist.start']());
			await invoke(METADATA_COMMANDS.syncAllAnilist, {
				language: metadataLanguageStore.metadataLanguage,
				generateComicInfo: comicInfoPreference.comicInfoPreference ?? false
			});
		} catch (error: unknown) {
			syncingSource = null;
			const msg = extractErrorMessage(error);
			notify.error(m['pages.config.toast.sync.anilist.error']({ msg }), { duration: 0 });
			toast.error(m['pages.config.toast.sync.anilist.error']({ msg }), { id: syncToastId });
			syncToastId = undefined;
		}
	}

	onMount(() => {
		let unlistenProgress: UnlistenFn | undefined;
		let unlistenComplete: UnlistenFn | undefined;
		let unlistenError: UnlistenFn | undefined;

		(async () => {
			unlistenProgress = await listen<string>('metadata:sync_all:progress', (event) => {
				const msg = m['pages.config.toast.sync.progress']({ name: event.payload });
				notify.info(msg, { duration: 0 });
				syncToastId = toast.loading(msg, { id: syncToastId });
			});

			unlistenComplete = await listen('metadata:sync_all:complete', () => {
				syncingSource = null;
				// A batch sync downloads a new cover for each synced comic, but the path on disk
				// doesn't change — without this the Home (and any other screen with the comic
				// list already mounted) kept showing the old cached cover until a manual reload.
				bumpArtworkVersion();
				notify.success(m['pages.config.toast.sync.complete'](), { duration: 0 });
				toast.success(m['pages.config.toast.sync.complete'](), { id: syncToastId });
				syncToastId = undefined;
			});

			unlistenError = await listen<any>('metadata:sync_all:error', (event) => {
				syncingSource = null;
				const msg = extractErrorMessage(event.payload);
				notify.error(m['pages.config.toast.sync.error']({ msg }), { duration: 0 });
				toast.error(m['pages.config.toast.sync.error']({ msg }), { id: syncToastId });
				syncToastId = undefined;
			});
		})();

		return () => {
			if (unlistenProgress) unlistenProgress();
			if (unlistenComplete) unlistenComplete();
			if (unlistenError) unlistenError();
		};
	});

	$effect(() => {
		comicInfoPreference.loadSavedComicInfoPreference();
	});

	$effect(() => {
		metadataLanguageStore.loadSavedMetadataLanguage();
	});
</script>

<div in:fade={{ duration: 150 }} class="mx-auto grid w-full max-w-3xl gap-6 px-8 pt-8 pb-14">
	<header class="grid gap-2">
		<p class="text-xs font-semibold text-muted-foreground">{m['pages.config.eyebrow']()}</p>
		<h1 class="text-[44px] leading-none font-black tracking-tighter text-foreground">
			{m['pages.config.title']()}
		</h1>
		<p class="text-sm text-muted-foreground">{m['pages.config.desc']()}</p>
	</header>

	<AcerolaSettingSection
		data={{
			id: 'files',
			title: m['pages.config.fileSystem.title'](),
			lead: m['pages.config.categories.files.desc']()
		}}
	>
		<AcerolaSettingRow
			data={{
				title: m['pages.config.fileSystem.comicPath.title'](),
				description: folder.folderPath || m['pages.config.fileSystem.comicPath.empty']()
			}}
		>
			{#snippet icon()}<FolderIcon size={18} />{/snippet}
			{#snippet control()}
				<AcerolaButton
					ui={{ variant: 'secondary' }}
					events={{ onClick: folder.selectFolder }}
				>
					{m['pages.config.actions.chooseFolder']()}
				</AcerolaButton>
			{/snippet}
		</AcerolaSettingRow>

		<AcerolaSettingRow
			data={{
				title: m['pages.config.fileSystem.comicInfo.title'](),
				description: m['pages.config.fileSystem.comicInfo.desc']()
			}}
		>
			{#snippet icon()}<FileTextIcon size={18} />{/snippet}
			{#snippet control()}
				<AcerolaSwitch
					state={{ checked: comicInfoPreference.comicInfoPreference ?? false }}
					events={{
						onCheckedChange: async (checked) => {
							await comicInfoPreference.selectComicInfoPreference(checked);
						}
					}}
					ui={{ 'aria-label': m['pages.config.fileSystem.comicInfo.title']() }}
				/>
			{/snippet}
		</AcerolaSettingRow>
	</AcerolaSettingSection>

	<AcerolaSettingSection
		data={{
			id: 'library',
			title: m['pages.config.library.title'](),
			lead: m['pages.config.categories.library.desc']()
		}}
	>
		<AcerolaSettingRow
			data={{
				title: m['pages.config.fileSystem.sync.fast.title'](),
				description: m['pages.config.fileSystem.sync.fast.desc']()
			}}
		>
			{#snippet icon()}<FolderSync size={18} />{/snippet}
			{#snippet control()}
				<AcerolaButton
					ui={{ variant: 'secondary', disabled: refreshScanner.scanning }}
					events={{ onClick: () => refreshScanner.start() }}
				>
					<RefreshCw class={refreshScanner.scanning ? 'animate-spin' : ''} />
					{refreshScanner.scanning
						? m['pages.config.actions.syncing']()
						: m['pages.config.actions.sync']()}
				</AcerolaButton>
			{/snippet}
		</AcerolaSettingRow>

		<AcerolaSettingRow
			data={{
				title: m['pages.config.fileSystem.sync.deep.title'](),
				description: m['pages.config.fileSystem.sync.deep.desc']()
			}}
		>
			{#snippet icon()}<DatabaseIcon size={18} />{/snippet}
			{#snippet control()}
				<AcerolaButton
					ui={{ variant: 'secondary', disabled: rebuildScanner.scanning }}
					events={{ onClick: () => rebuildScanner.start() }}
				>
					<RefreshCw class={rebuildScanner.scanning ? 'animate-spin' : ''} />
					{rebuildScanner.scanning
						? m['pages.config.actions.syncing']()
						: m['pages.config.actions.rebuild']()}
				</AcerolaButton>
			{/snippet}
		</AcerolaSettingRow>

		<AcerolaSettingRow
			data={{
				title: m['pages.config.templates.nav.title'](),
				description: m['pages.config.templates.nav.desc']()
			}}
		>
			{#snippet icon()}<FileCode2 size={18} />{/snippet}
			{#snippet control()}
				<AcerolaButton
					ui={{ variant: 'secondary' }}
					events={{ onClick: () => goto('/config/templates') }}
				>
					{m['pages.config.actions.open']()}
					<ChevronRightIcon />
				</AcerolaButton>
			{/snippet}
		</AcerolaSettingRow>
	</AcerolaSettingSection>

	<AcerolaSettingSection
		data={{
			id: 'metadata',
			title: m['pages.config.metadata.title'](),
			lead: m['pages.config.categories.metadata.desc']()
		}}
	>
		<AcerolaSettingRow
			data={{
				title: m['pages.config.metadata.lang.title'](),
				description: m['pages.config.metadata.lang.desc']()
			}}
		>
			{#snippet icon()}<LanguagesIcon size={18} />{/snippet}
			{#snippet control()}
				<AcerolaPopover
					state={{ open: metadataLanguagePopoverOpen }}
					events={{ onOpenChange: (open) => (metadataLanguagePopoverOpen = open) }}
					ui={{ align: 'end', contentClass: 'w-80 overflow-hidden p-0' }}
				>
					{#snippet trigger()}
						<Button variant="secondary" class="min-w-24 gap-2">
							<span class="max-w-36 truncate">{selectedMetadataLanguageLabel}</span>
							<ChevronDownIcon class="size-4" />
						</Button>
					{/snippet}

					{#snippet content()}
						<div class="grid gap-1 px-4 pt-4 pb-2">
							<h3 class="text-sm font-black tracking-tight text-foreground">
								{m['pages.config.metadata.lang.popoverTitle']()}
							</h3>
							<p class="text-xs text-muted-foreground">
								{m['pages.config.metadata.lang.popoverDesc']({
									language: selectedMetadataLanguageLabel
								})}
							</p>
						</div>

						<AcerolaCommand state={{ value: metadataLanguageStore.metadataLanguage }}>
							<div class="px-2">
								<Command.Input
									placeholder={m[
										'pages.config.metadata.lang.searchPlaceholder'
									]()}
								/>
							</div>

							<Command.List class="max-h-72 p-2">
								<Command.Empty
									class="px-4 py-8 text-center text-sm text-muted-foreground"
								>
									{m['pages.config.metadata.lang.empty']()}
								</Command.Empty>

								<Command.Group
									heading={m['pages.config.metadata.lang.groupTitle']()}
								>
									{#each LANGUAGES as lang (lang.code)}
										<Command.Item
											value={lang.code}
											class="cursor-pointer rounded-full px-3 py-2.5 data-[checked=true]:bg-primary-soft"
											onSelect={() => selectMetadataLanguage(lang.code)}
										>
											<span class="min-w-0 flex-1 truncate font-semibold">
												{lang.label}
											</span>
											<span class="text-xs text-muted-foreground"
												>{lang.code}</span
											>
										</Command.Item>
									{/each}
								</Command.Group>
							</Command.List>
						</AcerolaCommand>
					{/snippet}
				</AcerolaPopover>
			{/snippet}
		</AcerolaSettingRow>

		<AcerolaSettingRow
			data={{
				title: m['pages.config.metadata.mangadex.title'](),
				description: m['pages.config.metadata.mangadex.desc']()
			}}
		>
			{#snippet icon()}<MangaDexIcon class="size-5" />{/snippet}
			{#snippet control()}
				<AcerolaButton
					ui={{ variant: 'secondary', disabled: syncingSource !== null }}
					events={{ onClick: handleSyncAllMangadex }}
				>
					<RefreshCw class={syncingSource === 'mangadex' ? 'animate-spin' : ''} />
					{syncingSource === 'mangadex'
						? m['pages.config.actions.syncing']()
						: m['pages.config.actions.sync']()}
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
					ui={{ variant: 'secondary', disabled: syncingSource !== null }}
					events={{ onClick: handleSyncAllAnilist }}
				>
					<RefreshCw class={syncingSource === 'anilist' ? 'animate-spin' : ''} />
					{syncingSource === 'anilist'
						? m['pages.config.actions.syncing']()
						: m['pages.config.actions.sync']()}
				</AcerolaButton>
			{/snippet}
		</AcerolaSettingRow>
	</AcerolaSettingSection>

	<AcerolaSettingSection
		data={{
			id: 'bookmarks',
			title: m['pages.config.bookmarks.title'](),
			lead: m['pages.config.categories.bookmarks.desc']()
		}}
	>
		<AcerolaBookmarkManager />
	</AcerolaSettingSection>
</div>
