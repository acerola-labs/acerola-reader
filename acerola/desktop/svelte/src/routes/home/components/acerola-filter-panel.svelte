<script module lang="ts">
	import type { MetadataSource, SortBy, SortOrder } from '#lib/contracts/home/home.payloads.ts';
	import type { Category } from '#lib/contracts/bookmarks/bookmarks.payloads.ts';

	/** 'all' shows every comic, 'none' shows comics without a bookmark, a number filters by category id. */
	export type BookmarkFilter = 'all' | 'none' | number;

	export type FilterPanelProps = {
		state?: { open?: boolean };
		data: {
			sortBy: SortBy;
			sortOrder: SortOrder;
			showHidden: boolean;
			metadataSource: MetadataSource;
			bookmarkFilter: BookmarkFilter;
			bookmarks: Category[];
		};
		events: {
			onApply: (params: {
				sortBy: SortBy;
				sortOrder: SortOrder;
				showHidden: boolean;
				metadataSource: MetadataSource;
				bookmarkFilter: BookmarkFilter;
			}) => void;
			onClose: () => void;
		};
	};
</script>

<script lang="ts">
	import { untrack } from 'svelte';
	import { fly, fade } from 'svelte/transition';
	import { cubicOut } from 'svelte/easing';
	import ArrowUp from '@lucide/svelte/icons/arrow-up';
	import ArrowDown from '@lucide/svelte/icons/arrow-down';
	import X from '@lucide/svelte/icons/x';
	import Eye from '@lucide/svelte/icons/eye';
	import Bookmark from '@lucide/svelte/icons/bookmark';
	import AcerolaSwitch from '#lib/components/acerola-switch/acerola-switch.svelte';
	import AcerolaButton from '#lib/components/acerola-button/acerola-button.svelte';
	import AcerolaButtonIcon from '#lib/components/acerola-button/acerola-button-icon.svelte';
	import { m } from '#lib/paraglide/messages.js';

	let { state: controlState, data, events }: FilterPanelProps = $props();

	let localSortBy = $state<SortBy>('title');
	let localSortOrder = $state<SortOrder>('asc');
	let localShowHidden = $state(false);
	let localMetadataSource = $state<MetadataSource>('all');
	let localBookmarkFilter = $state<BookmarkFilter>('all');
	let wasOpen = false;

	$effect(() => {
		const isOpen = Boolean(controlState?.open);
		if (isOpen && !wasOpen) {
			localSortBy = data.sortBy;
			localSortOrder = data.sortOrder;
			localShowHidden = data.showHidden;
			localMetadataSource = data.metadataSource;
			localBookmarkFilter = data.bookmarkFilter;
		}
		wasOpen = isOpen;
	});

	const sortOptions = $derived<{ value: SortBy; labelKey: string }[]>([
		{ value: 'title', labelKey: m['pages.home.filterPanel.sortOptions.title']() },
		{
			value: 'chapterCount',
			labelKey: m['pages.home.filterPanel.sortOptions.chapterCount']()
		},
		{ value: 'lastUpdated', labelKey: m['pages.home.filterPanel.sortOptions.lastUpdated']() }
	]);

	const metadataSources = $derived<{ value: MetadataSource; label: string }[]>([
		{ value: 'all', label: m['pages.home.filterPanel.metadataSources.all']() },
		{ value: 'comicinfo', label: m['pages.home.filterPanel.metadataSources.comicinfo']() },
		{ value: 'mangadex', label: m['pages.home.filterPanel.metadataSources.mangadex']() },
		{ value: 'anilist', label: m['pages.home.filterPanel.metadataSources.anilist']() },
		{ value: 'no_metadata', label: m['pages.home.filterPanel.metadataSources.noMetadata']() }
	]);

	function selectSort(value: SortBy) {
		if (localSortBy === value) {
			localSortOrder = localSortOrder === 'asc' ? 'desc' : 'asc';
		} else {
			localSortBy = value;
			localSortOrder = 'asc';
		}
	}

	function handleApply() {
		events.onApply({
			sortBy: localSortBy,
			sortOrder: localSortOrder,
			showHidden: localShowHidden,
			metadataSource: localMetadataSource,
			bookmarkFilter: localBookmarkFilter
		});
	}

	function handleReset() {
		localSortBy = 'title';
		localSortOrder = 'asc';
		localShowHidden = false;
		localMetadataSource = 'all';
		localBookmarkFilter = 'all';
	}

	/** Brand book chip (`.ac-chip`): the selected one is bg-foreground, the rest sit on bg-surface. */
	function chip(isSelected: boolean) {
		return [
			'flex h-9 items-center gap-1.5 rounded-full px-3.5 text-sm font-bold transition-colors duration-300 ease-brand focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none',
			isSelected
				? 'bg-foreground text-background'
				: 'bg-surface text-muted-foreground hover:text-foreground'
		];
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape' && controlState?.open) {
			events.onClose();
		}
	}

	const hasChanges = $derived(
		localSortBy !== data.sortBy ||
			localSortOrder !== data.sortOrder ||
			localShowHidden !== data.showHidden ||
			localMetadataSource !== data.metadataSource ||
			localBookmarkFilter !== data.bookmarkFilter
	);

	const hasActiveFilters = $derived(
		localShowHidden || localMetadataSource !== 'all' || localBookmarkFilter !== 'all'
	);
</script>

<svelte:window onkeydown={handleKeydown} />

{#if controlState?.open}
	<!-- Backdrop -->
	<div
		role="presentation"
		class="fixed inset-x-0 top-8 bottom-0 z-40 bg-scrim/50"
		transition:fade={{ duration: 200 }}
		onclick={events.onClose}
	></div>

	<!-- Panel: brand book sheet (rounded-sheet on the open side, bg-card, shadow-float). -->
	<div
		role="dialog"
		aria-modal="true"
		aria-label={m['pages.home.filterPanel.ariaLabel']()}
		class="fixed top-8 right-0 bottom-0 z-50 flex w-full max-w-full flex-col rounded-l-sheet bg-card shadow-float sm:max-w-sm"
		transition:fly={{ x: 400, duration: 320, easing: cubicOut }}
	>
		<!-- Header -->
		<div class="flex items-start justify-between gap-3 px-6 pt-6 pb-4">
			<div>
				{#if hasActiveFilters}
					<p class="text-xs font-semibold text-primary">
						{m['pages.home.filterPanel.activeFilters']()}
					</p>
				{/if}
				<h2 class="text-xl font-black tracking-tight text-foreground">
					{m['pages.home.filterPanel.ariaLabel']()}
				</h2>
			</div>
			<AcerolaButtonIcon
				events={{ onClick: events.onClose }}
				ui={{
					variant: 'secondary',
					class: 'size-11 shrink-0',
					title: m['pages.home.filterPanel.closeAria'](),
					'aria-label': m['pages.home.filterPanel.closeAria']()
				}}
			>
				<X size={18} />
			</AcerolaButtonIcon>
		</div>

		<!-- Content - scrollable -->
		<div class="flex-1 space-y-7 overflow-y-auto px-6 pb-6">
			<!-- Sort Section -->
			<section>
				<h3 class="mb-3 text-sm font-black tracking-tight text-foreground">
					{m['pages.home.filterPanel.sortSectionTitle']()}
				</h3>
				<div
					class="space-y-2"
					role="radiogroup"
					aria-label={m['pages.home.filterPanel.sortRadiogroupAria']()}
				>
					{#each sortOptions as option (option.value)}
						{@const isSelected = localSortBy === option.value}
						<button
							type="button"
							role="radio"
							aria-checked={isSelected}
							class={[
								'flex min-h-11 w-full items-center justify-between gap-2 rounded-full px-4 text-sm font-bold transition-colors duration-300 ease-brand focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none',
								isSelected
									? 'bg-foreground text-background'
									: 'bg-surface text-muted-foreground hover:text-foreground'
							]}
							onclick={() => selectSort(option.value)}
						>
							<span class="truncate">{option.labelKey}</span>
							{#if isSelected}
								<span class="flex shrink-0 items-center gap-1 text-xs font-black">
									{#if localSortOrder === 'asc'}
										<ArrowUp size={14} />
										A–Z
									{:else}
										<ArrowDown size={14} />
										Z–A
									{/if}
								</span>
							{/if}
						</button>
					{/each}
				</div>
			</section>

			<!-- Filter Section -->
			<section>
				<h3 class="mb-3 text-sm font-black tracking-tight text-foreground">
					{m['pages.home.filterPanel.filterSectionTitle']()}
				</h3>

				<!-- Show Hidden Toggle Row -->
				<button
					type="button"
					role="switch"
					aria-checked={localShowHidden}
					class="flex w-full cursor-pointer items-center justify-between gap-3 rounded-tile bg-surface p-3 text-left transition-[filter] duration-200 ease-brand hover:brightness-110 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
					onclick={() => (localShowHidden = !localShowHidden)}
				>
					<span class="flex min-w-0 items-center gap-3">
						<span
							class="flex size-10 shrink-0 items-center justify-center rounded-full bg-card text-muted-foreground"
						>
							<Eye size={18} />
						</span>
						<span class="min-w-0">
							<span class="block text-sm font-black tracking-tight text-foreground">
								{m['pages.home.filterPanel.showHidden.title']()}
							</span>
							<span class="block text-xs text-muted-foreground">
								{m['pages.home.filterPanel.showHidden.desc']()}
							</span>
						</span>
					</span>
					<span class="pointer-events-none">
						<AcerolaSwitch state={{ checked: localShowHidden }} ui={{ size: 'sm' }} />
					</span>
				</button>
			</section>

			<!-- Metadata Source Section -->
			<section>
				<h3 class="mb-3 text-sm font-black tracking-tight text-foreground">
					{m['pages.home.filterPanel.metadataSourcesTitle']()}
				</h3>
				<div
					class="flex flex-wrap gap-2"
					role="group"
					aria-label={m['pages.home.filterPanel.metadataSourcesAria']()}
				>
					{#each metadataSources as source (source.value)}
						<button
							type="button"
							aria-pressed={localMetadataSource === source.value}
							class={chip(localMetadataSource === source.value)}
							onclick={() => (localMetadataSource = source.value)}
						>
							{source.label}
						</button>
					{/each}
				</div>
			</section>

			<!-- Bookmark Filter Section -->
			<section>
				<h3 class="mb-3 text-sm font-black tracking-tight text-foreground">
					{m['pages.home.filterPanel.bookmarkFilterTitle']()}
				</h3>
				<div
					class="flex flex-wrap gap-2"
					role="group"
					aria-label={m['pages.home.filterPanel.bookmarkFilterAria']()}
				>
					<button
						type="button"
						aria-pressed={localBookmarkFilter === 'all'}
						class={chip(localBookmarkFilter === 'all')}
						onclick={() => (localBookmarkFilter = 'all')}
					>
						{m['pages.home.filterPanel.bookmarkFilter.all']()}
					</button>
					<button
						type="button"
						aria-pressed={localBookmarkFilter === 'none'}
						class={chip(localBookmarkFilter === 'none')}
						onclick={() => (localBookmarkFilter = 'none')}
					>
						<Bookmark size={14} />
						{m['pages.home.filterPanel.bookmarkFilter.none']()}
					</button>
					{#each data.bookmarks as category (category.id)}
						{@const hexColor =
							'#' + (category.color & 0xffffff).toString(16).padStart(6, '0')}
						<button
							type="button"
							aria-pressed={localBookmarkFilter === category.id}
							class={chip(localBookmarkFilter === category.id)}
							onclick={() => (localBookmarkFilter = category.id)}
						>
							<span
								class="size-2.5 shrink-0 rounded-full"
								style="background-color: {hexColor};"
							></span>
							{category.name}
						</button>
					{/each}
				</div>
			</section>
		</div>

		<!-- Footer Actions -->
		<div class="flex gap-3 px-6 pt-2 pb-6">
			<AcerolaButton
				ui={{
					variant: 'secondary',
					class: 'flex-1',
					disabled: !hasChanges && !hasActiveFilters
				}}
				events={{ onClick: handleReset }}
			>
				{m['pages.home.filterPanel.reset']()}
			</AcerolaButton>
			<AcerolaButton ui={{ class: 'flex-1' }} events={{ onClick: handleApply }}>
				{m['pages.home.filterPanel.apply']()}
			</AcerolaButton>
		</div>
	</div>
{/if}
