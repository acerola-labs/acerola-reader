<script lang="ts" module>
	export type Chapter = {
		id: string;
		name?: string;
		title: string;
		fileName: string;
		isRead: boolean;
		hasConflict?: boolean;
		chapterSort: string;
		path?: string;
		volumeId?: string | null;
		volumeName?: string | null;
		isSpecial?: boolean;
		lastModified?: number;
		chapterIndex?: number;
	};

	export type ChapterPage = {
		page: number;
		items: Chapter[];
	};

	export type ComicChapterListProps = {
		data: {
			pagesData: ChapterPage[];
			totalChapters: number;
			pageSize: number;
			isSelectionMode?: boolean;
			isSelected?: (id: string) => boolean;
		};
		events?: {
			onOpenChapter?: (chapter: Chapter) => void;
			onToggleSelect?: (chapter: Chapter) => void;
			onEnterSelection?: (chapter: Chapter) => void;
			onMarkRead?: (chapter: Chapter) => void;
			onMarkUnread?: (chapter: Chapter) => void;
		};
	};
</script>

<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import { SvelteSet } from 'svelte/reactivity';
	import AcerolaHeroButton from '#lib/components/acerola-hero-button/acerola-hero-button.svelte';
	import AcerolaButton from '#lib/components/acerola-button/acerola-button.svelte';
	import AcerolaPopover from '#lib/components/acerola-popover/acerola-popover.svelte';
	import BookOpen from '@lucide/svelte/icons/book-open';
	import Check from '@lucide/svelte/icons/check';
	import CheckSquare from '@lucide/svelte/icons/check-square';
	import MoreVertical from '@lucide/svelte/icons/more-vertical';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';
	import TriangleAlert from '@lucide/svelte/icons/triangle-alert';
	import { m } from '#lib/paraglide/messages.js';

	let { data, events }: ComicChapterListProps = $props();

	let openMenuId = $state<string | null>(null);
	let openingChapterId = $state<string | null>(null);

	/**
	 * ITEM_HEIGHT (112px): Total slot height (item + gap).
	 * BUTTON_HEIGHT (100px): Real button height to fit Title + Description + Padding.
	 */
	const ITEM_HEIGHT = 112;
	const BUTTON_HEIGHT = 100;

	const totalPages = $derived(Math.ceil(data.totalChapters / data.pageSize));

	// Same highlight color the icon circle/badge uses for each state — conflict takes
	// priority over read (it needs the user to resolve it, more urgent than an already read
	// chapter).
	function rowClass(chapter: Chapter, isSelected: boolean, isOpening: boolean): string {
		const base = 'h-full flex-nowrap overflow-hidden';
		const selectedRing = isSelected
			? ' ring-2 ring-primary ring-offset-2 ring-offset-background'
			: '';
		const openingState = isOpening ? ' pointer-events-none opacity-70' : '';
		if (chapter.hasConflict) {
			return `${base} bg-destructive/10 hover:bg-destructive/20${selectedRing}${openingState}`;
		}
		if (chapter.isRead) {
			return `${base} bg-primary/10 hover:bg-primary/20${selectedRing}${openingState}`;
		}
		return `${base} border-surface/40 bg-mantle/40 hover:bg-surface/30${selectedRing}${openingState}`;
	}

	// INFO: Reactive on purpose — decides what stays mounted in the DOM. Chapter data
	// stays cached forever (we no longer evict), but keeping every loaded
	// AcerolaHeroButton mounted forever produces tens of thousands of DOM nodes in a
	// large series — each AcerolaHeroButton mounts about 40 nodes. Rendering only
	// what is near the current viewport solves this without flicker: since the data
	// is already cached, leaving and re-entering the window is instant, no refetch.
	let visiblePages = new SvelteSet<number>();

	/**
	 * Created eagerly (not in onMount): the `use:trackPage` actions of the
	 * {#each} blocks run during the initial mount, before the component's
	 * onMount fires — if the observer were created in onMount, the pages
	 * already mounted would never be observed.
	 */
	const observer = new IntersectionObserver(
		(entries) => {
			entries.forEach((entry) => {
				const pageStr = (entry.target as HTMLElement).dataset.page;
				if (!pageStr) return;

				const page = parseInt(pageStr, 10);

				if (entry.isIntersecting) {
					visiblePages.add(page);
				} else {
					visiblePages.delete(page);
				}
			});
		},
		{ rootMargin: '1200px 0px' }
	);

	onMount(() => {
		return () => observer.disconnect();
	});

	function trackPage(node: HTMLElement, page: number) {
		node.dataset.page = page.toString();
		observer.observe(node);
		return {
			destroy() {
				observer.unobserve(node);
			}
		};
	}

	/**
	 * DOM node recycling, like RecyclerView/LazyColumn: a FIXED pool of reused
	 * "seats" instead of mounting/unmounting an AcerolaHeroButton per page every
	 * time it enters/leaves the render window. Each seat is keyed by its own
	 * position in the pool (not by chapter id), so changing which page it
	 * represents only rebinds the props — the DOM node is never destroyed, which
	 * removes the pop-in/fade on every scroll.
	 */
	const POOL_SIZE = 6;

	let slots = $state<Array<{ pageIndex: number | null }>>(
		Array.from({ length: POOL_SIZE }, () => ({ pageIndex: null }))
	);

	const visiblePageList = $derived(Array.from(visiblePages).sort((a, b) => a - b));

	$effect(() => {
		const visible = visiblePageList;

		untrack(() => {
			const currentlyAssigned = new Set(
				slots.map((slot) => slot.pageIndex).filter((page): page is number => page !== null)
			);

			for (const slot of slots) {
				if (slot.pageIndex !== null && !visible.includes(slot.pageIndex)) {
					slot.pageIndex = null;
				}
			}

			const freeSlots = slots.filter((slot) => slot.pageIndex === null);
			const unassignedPages = visible.filter((page) => !currentlyAssigned.has(page));

			for (let i = 0; i < unassignedPages.length && i < freeSlots.length; i++) {
				freeSlots[i].pageIndex = unassignedPages[i];
			}
		});
	});
</script>

<div class="relative flex w-full flex-col">
	{#if totalPages > 0}
		{#each Array(totalPages) as _, pageIndex}
			<!-- prettier-ignore -->
			{@const itemsInThisPage = pageIndex === totalPages - 1 ? data.totalChapters % data.pageSize || data.pageSize : data.pageSize}

			<!-- Spacer: only reserves the height in the document flow and triggers the
			     IntersectionObserver — the real content comes from the recycled pool below. -->
			<div use:trackPage={pageIndex} style:height="{itemsInThisPage * ITEM_HEIGHT}px"></div>
		{/each}

		{#each slots as slot, slotIndex (slotIndex)}
			{@const blockData =
				slot.pageIndex !== null
					? data.pagesData.find((page) => page.page === slot.pageIndex)
					: undefined}

			{#each Array(data.pageSize) as _, itemIndex (itemIndex)}
				{@const chapter = blockData?.items[itemIndex]}

				{#if chapter}
					<div
						class="absolute right-0 left-0 animate-in duration-300 fade-in"
						style:top="{(slot.pageIndex ?? 0) * data.pageSize * ITEM_HEIGHT +
							itemIndex * ITEM_HEIGHT}px"
						style:height="{BUTTON_HEIGHT}px"
					>
						<AcerolaHeroButton
							data={{
								title: chapter.title,
								description: chapter.fileName
							}}
							events={{
								onClick: () => {
									if (data.isSelectionMode) {
										events?.onToggleSelect?.(chapter);
									} else {
										openingChapterId = chapter.id;
										events?.onOpenChapter?.(chapter);
									}
								}
							}}
							ui={{
								class: rowClass(
									chapter,
									data.isSelected?.(chapter.id) ?? false,
									openingChapterId === chapter.id
								)
							}}
						>
							{#snippet icon()}
								{#if openingChapterId === chapter.id && !data.isSelectionMode}
									<div class="text-primary">
										<RefreshCw size={24} class="animate-spin" />
									</div>
								{:else if data.isSelectionMode}
									{#if data.isSelected?.(chapter.id)}
										<div
											class="flex h-6 w-6 items-center justify-center rounded-full bg-primary text-crust"
										>
											<Check size={16} strokeWidth={3} />
										</div>
									{:else}
										<div class="flex h-6 w-6 items-center justify-center">
											<div class="h-5 w-5 rounded-full border-2"></div>
										</div>
									{/if}
								{:else if chapter.hasConflict}
									<!-- Same idea as the "read" circle (main icon + different color), but
									     with bg-destructive/15 (tinted) instead of solid: there is no
									     --destructive-foreground in the theme (nowhere in the app uses
									     `destructive` as a solid background with text/icon on top), so contrast
									     isn't guaranteed for that combination — tinted, the icon only uses the
									     `text-destructive` color over the background behind, which is already
									     used elsewhere in the app. -->
									<div
										class="flex h-6 w-6 items-center justify-center rounded-full bg-destructive/15 text-destructive"
									>
										<TriangleAlert size={16} />
									</div>
								{:else if chapter.isRead}
									<div
										class="flex h-6 w-6 items-center justify-center rounded-full bg-primary text-crust"
									>
										<Check size={16} strokeWidth={3} />
									</div>
								{:else}
									<div class="text-primary">
										<BookOpen size={24} />
									</div>
								{/if}
							{/snippet}

							{#snippet action()}
								<div class="flex items-center gap-2">
									{#if chapter.hasConflict}
										<span
											class="rounded-full bg-muted px-3 py-1 text-[10px] font-black tracking-widest text-muted-foreground uppercase"
										>
											{m['pages.comic.metadata.conflict']()}
										</span>
									{/if}
									{#if chapter.isRead}
										<span
											class="rounded-full bg-primary/10 px-3 py-1 text-[10px] font-black tracking-widest text-primary uppercase"
										>
											{m['pages.comic.metadata.completed']
												? m['pages.comic.metadata.completed']()
												: 'LIDO'}
										</span>
									{/if}
									<div role="presentation" onclick={(event) => event.stopPropagation()}>
										<AcerolaPopover
											state={{ open: openMenuId === chapter.id }}
											events={{
												onOpenChange: (open) => (openMenuId = open ? chapter.id : null)
											}}
											ui={{
												align: 'end',
												contentClass:
													'w-52 overflow-hidden rounded-inner border-border/40 bg-card/95 p-1.5 shadow-float backdrop-blur-md'
											}}
										>
											{#snippet trigger()}
												<span
													class="flex size-10 items-center justify-center rounded-inner bg-accent-hero text-accent-hero-foreground transition-[filter] hover:brightness-110"
												>
													<MoreVertical size={20} />
												</span>
											{/snippet}

											{#snippet content()}
												<div class="flex flex-col gap-0.5">
													{#if chapter.isRead}
														<AcerolaButton
															ui={{
																variant: 'ghost',
																class:
																	'h-9 w-full justify-start gap-2.5 rounded-inner px-2.5 text-sm font-medium text-warning hover:bg-warning-soft hover:text-warning'
															}}
															events={{
																onClick: () => {
																	openMenuId = null;
																	events?.onMarkUnread?.(chapter);
																}
															}}
														>
															<BookOpen size={16} class="shrink-0" />
															{m['pages.comic.selection.markUnread']()}
														</AcerolaButton>
													{:else}
														<AcerolaButton
															ui={{
																variant: 'ghost',
																class:
																	'h-9 w-full justify-start gap-2.5 rounded-inner px-2.5 text-sm font-medium text-primary hover:bg-primary/10'
															}}
															events={{
																onClick: () => {
																	openMenuId = null;
																	events?.onMarkRead?.(chapter);
																}
															}}
														>
															<Check size={16} class="shrink-0" />
															{m['pages.comic.selection.markRead']()}
														</AcerolaButton>
													{/if}

													<div class="mx-1 my-1 h-px bg-border/60"></div>

													<AcerolaButton
														ui={{
															variant: 'ghost',
															class:
																'h-9 w-full justify-start gap-2.5 rounded-inner px-2.5 text-sm font-medium'
														}}
														events={{
															onClick: () => {
																openMenuId = null;
																events?.onEnterSelection?.(chapter);
															}
														}}
													>
														<CheckSquare size={16} class="shrink-0 text-muted-foreground" />
														{m['pages.comic.selection.select']()}
													</AcerolaButton>
												</div>
											{/snippet}
										</AcerolaPopover>
									</div>
								</div>
							{/snippet}
						</AcerolaHeroButton>
					</div>
				{/if}
			{/each}
		{/each}
	{:else}
		<div class="space-y-4 py-20 text-center opacity-50">
			<RefreshCw size={48} class="mx-auto animate-spin text-primary" />
			<p class="text-sm font-black tracking-widest uppercase">{m['pages.comic.loading']()}</p>
		</div>
	{/if}
</div>
