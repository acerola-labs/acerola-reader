<script module lang="ts">
	import type { ComicSummary } from '#lib/contracts/network/network.payloads.ts';

	export type AcerolaRemoteLibraryDialogProps = {
		state: {
			open: boolean;
		};
		data: {
			peerLabel: string;
			comics: ComicSummary[];
			isLoading: boolean;
			errorMessage?: string;
			coverPathFor: (comicName: string) => string | undefined;
			isSyncing: (comicName: string) => boolean;
		};
		events: {
			onOpenChange: (open: boolean) => void;
			onSelectComic: (comicName: string) => void;
		};
	};
</script>

<script lang="ts">
	import { m } from '#lib/paraglide/messages.js';
	import AcerolaDialog from '#lib/components/acerola-dialog/acerola-dialog.svelte';
	import AcerolaCardImage from '#lib/components/acerola-card/acerola-card-image.svelte';
	import AcerolaEmptyState from '#lib/components/acerola-empty-state/acerola-empty-state.svelte';
	import PlaceholderManga from '#lib/assets/placeholder/placeholder_manga.svg?component';
	import SearchIcon from '@lucide/svelte/icons/search';
	import BookOpenIcon from '@lucide/svelte/icons/book-open';
	import Loader2Icon from '@lucide/svelte/icons/loader-2';

	// Renamed from `state` in the destructuring — a local binding named `state` in the same
	// scope as the `$state(...)` rune below collides with Svelte's store auto-subscribe sugar
	// (`$name`), and the compiler tries to treat `state` as a store.
	let { state: dialogState, data, events }: AcerolaRemoteLibraryDialogProps = $props();

	let searchQuery = $state('');

	// On story teardown (Storybook + vitest browser mode), this template's reactive effect
	// runs once more with `data`/`events` already undefined before the component is actually
	// destroyed — without the fallback here this leaks as an unhandled error and brings down
	// the suite even with every assert passing (same pattern as
	// acerola-network-my-device-card.svelte).
	let safeData = $derived(
		data ?? {
			peerLabel: '',
			comics: [],
			isLoading: false,
			errorMessage: undefined,
			coverPathFor: () => undefined,
			isSyncing: () => false
		}
	);
	let safeEvents = $derived(events ?? { onOpenChange: () => {}, onSelectComic: () => {} });

	// The dialog is reused across different peers (not remounted on every open) — without
	// this, a previous session's filter would leak into the next time the user opens it for
	// another peer.
	$effect(() => {
		if (dialogState?.open) searchQuery = '';
	});

	const filteredComics = $derived.by(() => {
		const query = searchQuery.trim().toLowerCase();
		if (!query) return safeData.comics;
		return safeData.comics.filter((comic) => comic.comicName.toLowerCase().includes(query));
	});
</script>

<AcerolaDialog
	state={{ open: dialogState?.open ?? false }}
	data={{
		title: m['pages.network.remoteLibrary.title'](),
		description: safeData.peerLabel
	}}
	events={{ onOpenChange: safeEvents.onOpenChange }}
	ui={{ contentClass: 'sm:max-w-2xl' }}
>
	<div class="flex w-full min-w-0 flex-col gap-4">
		<!-- Search field (brand book): pill on bg-surface, no border. -->
		<div class="relative w-full">
			<SearchIcon
				size={16}
				class="absolute top-1/2 left-4 shrink-0 -translate-y-1/2 text-muted-foreground"
			/>
			<input
				type="text"
				placeholder={m['pages.network.remoteLibrary.searchPlaceholder']()}
				aria-label={m['pages.network.remoteLibrary.searchPlaceholder']()}
				bind:value={searchQuery}
				class="h-11 w-full rounded-full bg-input pr-4 pl-11 text-sm text-foreground placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
			/>
		</div>

		{#if safeData.isLoading}
			<!-- Loading: skeleton in the shape of the cover grid. -->
			<div
				class="grid w-full grid-cols-[repeat(auto-fill,minmax(8.5rem,1fr))] gap-4"
				role="status"
				aria-label={m['pages.network.remoteLibrary.loading']()}
			>
				{#each Array.from({ length: 8 }, (_, i) => i) as i (i)}
					<div class="aspect-2/3 animate-pulse rounded-inner bg-surface"></div>
				{/each}
			</div>
		{:else if safeData.errorMessage}
			<div
				class="rounded-card bg-destructive-soft p-5 text-sm font-semibold text-destructive"
			>
				{safeData.errorMessage}
			</div>
		{:else if filteredComics.length === 0}
			<AcerolaEmptyState
				data={{ title: m['pages.network.remoteLibrary.empty'](), icon: BookOpenIcon }}
				ui={{ class: 'py-6' }}
			/>
		{:else}
			<div
				class="grid max-h-[28rem] w-full grid-cols-[repeat(auto-fill,minmax(8.5rem,1fr))] gap-4 overflow-y-auto p-1"
			>
				{#each filteredComics as comic (comic.comicName)}
					{@const comicSyncing = safeData.isSyncing(comic.comicName)}
					{@const coverPath = safeData.coverPathFor(comic.comicName)}
					<AcerolaCardImage
						data={{
							title: comic.comicName,
							cover: coverPath
						}}
						ui={{ class: 'w-full' }}
						events={{
							onClick: () => {
								if (comicSyncing) return;
								safeEvents.onSelectComic(comic.comicName);
							}
						}}
					>
						{#snippet footer()}
							<span
								class="flex items-center gap-1 text-[10px] font-black tracking-wider text-muted-foreground uppercase"
							>
								<BookOpenIcon size={10} />
								{m['pages.network.remoteLibrary.chapterCount']({
									count: comic.chapterCount
								})}
							</span>
						{/snippet}

						{#snippet placeholder()}
							<div class="h-full w-full bg-surface">
								<PlaceholderManga class="h-full w-full" />
							</div>
						{/snippet}

						{#snippet overlay()}
							{#if comicSyncing}
								<div
									class="absolute inset-0 flex items-center justify-center bg-scrim/50"
								>
									<Loader2Icon size={24} class="animate-spin text-primary" />
								</div>
							{/if}
						{/snippet}
					</AcerolaCardImage>
				{/each}
			</div>
		{/if}
	</div>
</AcerolaDialog>
