<script lang="ts" module>
	import type { AcerolaSelectOption } from '#lib/components/acerola-select/acerola-select.svelte';
	import type { SidebarItem } from '#lib/components/acerola-sidebar/acerola-sidebar.svelte';
	import type { Locale } from '#lib/paraglide/runtime.js';

	import { m } from '#lib/paraglide/messages.js';
	import { locales } from '#lib/paraglide/runtime.js';

	import HistoryIcon from '@lucide/svelte/icons/history';
	import HouseIcon from '@lucide/svelte/icons/house';
	import Share2Icon from '@lucide/svelte/icons/share-2';
	import SettingsIcon from '@lucide/svelte/icons/settings';

	import AcerolaLogo from '#lib/assets/icons/acerola-logo-foreground.svg?component';
	import MinusIcon from '@lucide/svelte/icons/minus';
	import SquareIcon from '@lucide/svelte/icons/square';
	import XIcon from '@lucide/svelte/icons/x';

	const localeLabels: Record<string, string> = {
		'pt-br': 'Português',
		en: 'English'
	};

	const localeOptions: AcerolaSelectOption[] = locales.map((locale) => ({
		value: locale,
		label: localeLabels[locale] || locale.toUpperCase()
	}));

	const navItems: SidebarItem[] = $derived([
		{ label: m['routes.home'](), href: '/home', icon: HouseIcon },
		{ label: m['routes.history'](), href: '/history', icon: HistoryIcon },
		{ label: m['routes.network'](), href: '/network', icon: Share2Icon },
		{ label: m['routes.config'](), href: '/config', icon: SettingsIcon }
	]);
</script>

<script lang="ts">
	import { DIRECTORY_SCAN_COMMANDS } from '#lib/contracts/library/library.commands.ts';
	import { useLibraryScanner } from '#lib/hooks/store/use-comic-scanner.svelte.ts';
	import { useComicSummary } from '#lib/hooks/store/use-comic-summary.svelte.ts';
	import { useSelectFolder } from '#lib/hooks/store/use-select-folder.svelte.ts';
	import { useBookmarks } from '#lib/hooks/store/use-bookmarks.svelte.ts';
	import { useNetworkSync } from '#lib/hooks/store/use-network-sync.svelte.ts';
	import { usePeerConnection } from '#lib/hooks/store/use-peer-connection.svelte.ts';
	import { useOnboarding } from '#lib/hooks/onboarding/use-onboarding.svelte.ts';
	import { setComicContext } from '#lib/state/comic-context.svelte.ts';
	import { CONTEXT_KEYS } from '#lib/constants/context-keys.ts';
	import { getLocale, setLocale } from '#lib/paraglide/runtime.js';
	import { onDestroy, onMount, setContext } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { resolveCover } from '#lib/utils/artwork.util.ts';

	import AcerolaModePicker from '#lib/components/acerola-mode-picker/acerola-mode-picker.svelte';
	import AcerolaButtonIcon from '#lib/components/acerola-button/acerola-button-icon.svelte';
	import AcerolaPopover from '#lib/components/acerola-popover/acerola-popover.svelte';
	import AcerolaSidebar from '#lib/components/acerola-sidebar/acerola-sidebar.svelte';
	import AcerolaSonner from '#lib/components/acerola-sonner/acerola-sonner.svelte';
	import AcerolaDialog from '#lib/components/acerola-dialog/acerola-dialog.svelte';
	import AcerolaCommand from '#lib/components/acerola-command/acerola-command.svelte';
	import * as Command from '#lib/components/ui/command/index.ts';
	import { Command as CommandPrimitive } from 'bits-ui';

	import AcerolaNotification from '#lib/components/acerola-notification/acerola-notification.svelte';
	import AcerolaOnboarding from './(onboarding)/acerola-onboarding.svelte';
	import '$theme/layout.css';
	import Search from '@lucide/svelte/icons/search';
	import BookOpenIcon from '@lucide/svelte/icons/book-open';
	import CheckIcon from '@lucide/svelte/icons/check';
	import GlobeIcon from '@lucide/svelte/icons/globe';
	import RefreshCwIcon from '@lucide/svelte/icons/refresh-cw';

	setComicContext();

	let currentLocale = $state(getLocale());
	let appWindow = $state<any>(null);
	let packageIdentity = $state<string | null>(null);
	let isSearchDialogOpen = $state(false);

	const folder = useSelectFolder();
	const summary = useComicSummary();
	const bookmarkStore = useBookmarks();
	const onboarding = useOnboarding();
	// The single `useNetworkSync()` instance of the whole app — it must stay alive regardless of
	// which route is mounted (for the global header indicator below, which reads `activeSession()`)
	// and is shared via context with `routes/network/+page.svelte` (see `CONTEXT_KEYS.networkSync`).
	// Before, the Network page created its OWN instance and unmounted it (`stopListening()`) when
	// leaving the screen — `syncComic()` only resolves via a listener of that instance, so
	// navigating to another screen with a sync still running rejected the promise with "sync
	// cancelled: listener stopped" even though the real sync kept going (and finished fine) in
	// the backend.
	const headerSync = useNetworkSync();
	const headerPeers = usePeerConnection();
	setContext(CONTEXT_KEYS.networkSync, headerSync);

	const incrementalScanner = useLibraryScanner(
		DIRECTORY_SCAN_COMMANDS.incrementalScan,
		() => folder.folderPath
	);

	onMount(async () => {
		// INFO: Dynamic import to avoid breaking during SSR (Server-Side Rendering)
		const { getCurrentWindow } = await import('@tauri-apps/api/window');
		const { invoke } = await import('@tauri-apps/api/core');
		const { error: logError } = await import('@tauri-apps/plugin-log');

		window.addEventListener('error', (event) => {
			logError(
				`[Frontend Error] ${event.message} at ${event.filename}:${event.lineno}:${event.colno}`
			);
		});
		window.addEventListener('unhandledrejection', (event) => {
			logError(`[Frontend UnhandledRejection] ${event.reason}`);
		});

		await folder.loadSavedPath();
		await bookmarkStore.loadBookmarks();

		if (import.meta.env.DEV) {
			try {
				const packageFamilyName = await invoke<string>('get_package_family_name');
				if (
					packageFamilyName !== 'No package identity' &&
					!packageFamilyName.startsWith('Error')
				) {
					packageIdentity = `Package: ${packageFamilyName}`;
				} else {
					packageIdentity = `Not running with package identity`;
				}
			} catch (error) {
				logError(`Failed to check package identity: ${error}`);
			}
		}

		appWindow = getCurrentWindow();

		if (folder.folderPath) {
			incrementalScanner.start();
		}

		headerSync.startListening();
		headerPeers.startListening();
	});

	onDestroy(() => {
		headerSync.stopListening();
		headerPeers.stopListening();
	});

	function minimize() {
		appWindow?.minimize();
	}

	function toggleMaximize() {
		appWindow?.toggleMaximize();
	}

	function closeWindow() {
		appWindow?.close();
	}

	function handleGlobalKeydown(event: KeyboardEvent) {
		const key = event.key.toLowerCase();
		if ((event.ctrlKey || event.metaKey) && key === 'k') {
			if ($page.url.pathname.startsWith('/reader')) return;
			event.preventDefault();
			isSearchDialogOpen = !isSearchDialogOpen;
			if (isSearchDialogOpen && !summary.comics) summary.fetch();
		}
	}

	$effect(() => {
		setLocale(currentLocale as Locale);
	});

	const { children } = $props();
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

<div class="flex h-screen w-full flex-col overflow-hidden">
	<!-- Titlebar Global -->
	<div
		data-tauri-drag-region
		class="z-100 flex h-8 shrink-0 items-center justify-between bg-mantle select-none"
	>
		<div
			data-tauri-drag-region
			class="pointer-events-none flex h-full flex-1 items-center pl-4 text-xs font-semibold text-muted-foreground"
		>
			Acerola
		</div>

		<div class="flex h-full text-foreground">
			<button
				class="inline-flex h-full w-11.5 cursor-pointer items-center justify-center transition-colors hover:bg-muted"
				aria-label={m['layout.buttons.minimize']()}
				onclick={minimize}
			>
				<MinusIcon size={16} strokeWidth={1.5} />
			</button>

			<button
				class="inline-flex h-full w-11.5 cursor-pointer items-center justify-center transition-colors hover:bg-muted"
				aria-label={m['layout.buttons.maximize']()}
				onclick={toggleMaximize}
			>
				<SquareIcon size={14} strokeWidth={1.5} />
			</button>

			<button
				class="inline-flex h-full w-11.5 cursor-pointer items-center justify-center transition-colors hover:bg-destructive hover:text-destructive-foreground"
				aria-label={m['layout.buttons.close']()}
				onclick={closeWindow}
			>
				<XIcon size={18} strokeWidth={1.5} />
			</button>
		</div>
	</div>

	<!-- Main Application Content -->
	<div class="relative flex flex-1 overflow-hidden">
		<AcerolaSonner />

		{#if onboarding.isCompleted && !$page.error}
			<AcerolaSidebar data={{ items: navItems }}>
				{#snippet brand()}
					<AcerolaLogo class="h-full w-full" />
				{/snippet}
			</AcerolaSidebar>
		{/if}

		<main class="h-full flex-1 overflow-y-auto">
			{#if $page.error}
				<!-- Acerola standard: only the error on screen, no navigation; its two actions are the way out. -->
				{@render children()}
			{:else if onboarding.isLoading}
				<div
					class="flex h-full w-full flex-col gap-6 p-8"
					role="status"
					aria-label={m['layout.loading']()}
				>
					<div class="h-16 w-full max-w-xl animate-pulse rounded-full bg-surface"></div>
					<div class="grid flex-1 grid-cols-2 gap-6 sm:grid-cols-4 lg:grid-cols-6">
						{#each Array.from({ length: 12 }, (_, i) => i) as i (i)}
							<div class="aspect-2/3 animate-pulse rounded-tile bg-surface"></div>
						{/each}
					</div>
				</div>
			{:else if !onboarding.isCompleted}
				<AcerolaOnboarding />
			{:else}
				<header
					class="sticky top-0 z-40 flex h-18 items-center justify-between bg-background px-8"
				>
					<div class="max-w-xl flex-1">
						<button
							aria-label={m['layout.searchPlaceholder']()}
							class="group relative w-full cursor-text text-left"
							onclick={() => {
								isSearchDialogOpen = true;
								if (!summary.comics) summary.fetch();
							}}
						>
							<Search
								class="absolute top-1/2 left-4 -translate-y-1/2 text-muted-foreground transition-colors group-hover:text-foreground"
								size={18}
							/>
							<div
								class="flex h-11 w-full items-center justify-between rounded-full bg-input pr-2 pl-11 text-sm text-muted-foreground"
							>
								<span>{m['layout.searchPlaceholder']()}</span>
								<kbd
									class="pointer-events-none hidden items-center gap-1 rounded-full bg-card px-2.5 py-1 text-[10px] font-black text-muted-foreground select-none sm:flex"
								>
									Ctrl+K
								</kbd>
							</div>
						</button>
					</div>

					<div class="mx-8 flex items-center gap-3">
						{#if packageIdentity}
							<span class="text-xs text-muted-foreground">{packageIdentity}</span>
						{/if}

						{#if headerSync.activeSession()}
							{@const session = headerSync.activeSession()}
							{@const progress = headerSync.activeProgressMessage()}
							<button
								type="button"
								class="flex h-11 max-w-56 items-center gap-2 rounded-full bg-primary-soft px-4 text-xs font-black text-primary transition-[filter] hover:brightness-110"
								title={progress ?? m['layout.syncIndicator.tooltip']()}
								onclick={() => goto('/network')}
							>
								<RefreshCwIcon size={14} class="shrink-0 animate-spin" />
								<span class="truncate">
									{m['layout.syncIndicator.syncing']({
										peer: session ? headerPeers.peerLabel(session.peerId) : ''
									})}
								</span>
							</button>
						{/if}

						<AcerolaModePicker />

						<AcerolaPopover
							ui={{
								align: 'end',
								side: 'bottom',
								sideOffset: 8,
								contentClass: 'w-56 p-2'
							}}
						>
							{#snippet trigger()}
								<AcerolaButtonIcon
									ui={{
										title: m['layout.languagePicker.title'](),
										variant: 'secondary'
									}}
								>
									<GlobeIcon size={16} />
								</AcerolaButtonIcon>
							{/snippet}

							{#snippet content()}
								<!-- The languages as a plain list: one click picks, the current one is marked. -->
								<div
									class="grid gap-0.5"
									role="radiogroup"
									aria-label={m['layout.languagePicker.title']()}
								>
									{#each localeOptions as option (option.value)}
										<button
											type="button"
											role="radio"
											aria-checked={currentLocale === option.value}
											class={[
												'flex h-11 w-full items-center justify-between gap-3 rounded-full px-4 text-left text-sm font-bold transition-colors duration-200 ease-brand focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none',
												currentLocale === option.value
													? 'bg-primary-soft text-primary'
													: 'text-foreground hover:bg-surface'
											]}
											onclick={() => (currentLocale = option.value as Locale)}
										>
											{option.label}
											{#if currentLocale === option.value}
												<CheckIcon size={16} />
											{/if}
										</button>
									{/each}
								</div>
							{/snippet}
						</AcerolaPopover>

						<AcerolaNotification />
					</div>
				</header>

				{@render children()}
			{/if}
		</main>
	</div>
</div>

<AcerolaDialog
	state={{ open: isSearchDialogOpen }}
	events={{ onOpenChange: (open) => (isSearchDialogOpen = open) }}
	ui={{
		contentClass:
			'p-0 gap-0 overflow-hidden w-full max-w-4xl sm:max-w-4xl top-10 portrait:top-4 sm:top-10 translate-y-0',
		showCloseButton: false
	}}
>
	<AcerolaCommand>
		<div class="p-4 pb-2">
			<div class="group relative">
				<Search
					class="absolute top-1/2 left-5 -translate-y-1/2 text-muted-foreground transition-colors group-focus-within:text-foreground"
					size={20}
				/>
				<CommandPrimitive.Input
					placeholder={m['layout.searchPlaceholder']()}
					class="h-14 w-full rounded-full bg-input pr-5 pl-13 text-lg font-semibold outline-none placeholder:text-muted-foreground"
				/>
			</div>
		</div>
		<Command.List class="max-h-[70vh] overflow-y-auto p-4">
			<Command.Empty class="py-24 text-center text-lg text-muted-foreground">
				{m['layout.command.noResults']()}
			</Command.Empty>

			<Command.Group
				heading={m['layout.command.navigationGroup']()}
				class="px-2 text-muted-foreground"
			>
				{#each navItems as item (item.href)}
					<Command.Item
						value={item.label}
						onSelect={() => {
							isSearchDialogOpen = false;
							goto(item.href);
						}}
						class="flex cursor-pointer items-center gap-3 rounded-full px-4 py-3 transition-colors data-[selected=true]:bg-surface"
					>
						<item.icon size={18} />
						<span class="text-sm font-semibold text-foreground">{item.label}</span>
					</Command.Item>
				{/each}
			</Command.Group>

			{#if summary.comics && summary.comics.total > 0}
				<Command.Group
					heading={m['layout.command.libraryGroup']()}
					class="px-2 text-muted-foreground"
				>
					<!-- Compact list row: small cover, title, chapter count and the bookmark dot. -->
					<div class="flex flex-col gap-1">
						{#each summary.comics.comics as comic (comic.relations.directoryId)}
							{@const cover = resolveCover(comic.artwork)}
							{@const comicBookmark = bookmarkStore.getBookmarkForComic(
								comic.relations.directoryId
							)}
							<Command.Item
								value={`${comic.metadata.title ?? ''} ${comic.filesystem.folderName}`}
								onSelect={() => {
									isSearchDialogOpen = false;
									import('$app/navigation').then((n) =>
										n.goto(`/comic/${comic.filesystem.folderName}`)
									);
								}}
								class="flex cursor-pointer items-center gap-4 rounded-tile p-2 pr-4 transition-colors data-[selected=true]:bg-surface"
							>
								<span
									class="flex aspect-2/3 w-10 shrink-0 items-center justify-center overflow-hidden rounded-inner bg-surface text-muted-foreground"
								>
									{#if cover}
										<img src={cover} alt="" class="size-full object-cover" />
									{:else}
										<BookOpenIcon size={16} />
									{/if}
								</span>
								<span class="flex min-w-0 flex-1 flex-col">
									<span
										class="truncate text-sm font-black tracking-tight text-foreground"
									>
										{comic.metadata.title || comic.filesystem.folderName}
									</span>
									<span class="text-xs text-muted-foreground tabular-nums">
										{m['layout.command.chapterCount']({
											count: comic.metadata.chapterCount
										})}
									</span>
								</span>
								{#if comicBookmark != null}
									<span
										class="size-2.5 shrink-0 rounded-full"
										style="background-color: #{(comicBookmark.color & 0xffffff)
											.toString(16)
											.padStart(6, '0')}"
										title={comicBookmark.name}
									></span>
								{/if}
							</Command.Item>
						{/each}
					</div>
				</Command.Group>
			{/if}
		</Command.List>
	</AcerolaCommand>
</AcerolaDialog>
