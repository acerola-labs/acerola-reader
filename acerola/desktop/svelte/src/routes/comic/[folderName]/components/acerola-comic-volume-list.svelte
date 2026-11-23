<script lang="ts" module>
	export type VolumeChapter = {
		id: string;
		name?: string;
		title: string;
		fileName: string;
		isRead: boolean;
		chapterSort?: string;
		path?: string;
		volumeId?: string | null;
		volumeName?: string | null;
		isSpecial?: boolean;
		lastModified?: number;
		chapterIndex?: number;
	};

	export type Volume = {
		id: string;
		title: string;
		chapters: VolumeChapter[];
		totalChapters: number;
		hasMore: boolean;
		coverUri?: string | null;
		bannerUri?: string | null;
	};

	export type VolumePage = {
		page: number;
		items: VolumeChapter[];
	};

	export type ComicVolumeListProps = {
		data: {
			volumes: Volume[];
			pagesData?: VolumePage[];
			loading?: boolean;
			pageSize?: number;
			viewMode?: 'cover' | 'banner';
			isSelectionMode?: boolean;
			isSelected?: (id: string) => boolean;
		};
		events: {
			onExpand: (volumeId: string | null) => void;
			onOpenChapter?: (chapter: VolumeChapter) => void;
			onToggleSelect?: (chapter: VolumeChapter) => void;
			onEnterSelection?: (chapter: VolumeChapter) => void;
			onMarkRead?: (chapter: VolumeChapter) => void;
			onMarkUnread?: (chapter: VolumeChapter) => void;
		};
	};
</script>

<script lang="ts">
	import { cn } from '#lib/utils/cn.util.ts';
	import { slide } from 'svelte/transition';
	import AcerolaHeroButton from '#lib/components/acerola-hero-button/acerola-hero-button.svelte';
	import AcerolaButton from '#lib/components/acerola-button/acerola-button.svelte';
	import AcerolaPopover from '#lib/components/acerola-popover/acerola-popover.svelte';
	import ComicVolumeButton from './acerola-comic-volume-button.svelte';
	import Folder from '@lucide/svelte/icons/folder';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';
	import BookOpen from '@lucide/svelte/icons/book-open';
	import Check from '@lucide/svelte/icons/check';
	import CheckSquare from '@lucide/svelte/icons/check-square';
	import MoreVertical from '@lucide/svelte/icons/more-vertical';
	import RefreshCw from '@lucide/svelte/icons/refresh-cw';
	import { onMount, untrack } from 'svelte';
	import { SvelteSet } from 'svelte/reactivity';
	import { m } from '#lib/paraglide/messages.js';

	let { data, events }: ComicVolumeListProps = $props();

	let openMenuId = $state<string | null>(null);
	let openingChapterId = $state<string | null>(null);

	const volumes = $derived(data.volumes ?? []);
	const pagesData = $derived(data.pagesData ?? []);
	const loading = $derived(data.loading ?? false);
	const pageSize = $derived(data.pageSize ?? 25);
	const viewMode = $derived(data.viewMode ?? 'cover');

	let expandedVolumeId = $state<string | null>(null);

	// INFO: Reactive on purpose — decides what stays mounted in the DOM, not just what
	// fires a fetch. See the same reasoning in acerola-comic-chapter-list.svelte.
	let visiblePages = new SvelteSet<number>();

	const ITEM_HEIGHT = 80;
	const BUTTON_HEIGHT = 72;

	function rowClass(chapter: VolumeChapter, isSelected: boolean, isOpening: boolean): string {
		const base = 'h-full flex-nowrap overflow-hidden';
		const selectedRing = isSelected ? ' bg-primary-soft!' : '';
		const openingState = isOpening ? ' pointer-events-none opacity-70' : '';
		if (chapter.isRead) {
			return `${base} bg-primary-soft hover:brightness-110${selectedRing}${openingState}`;
		}
		return `${base} bg-surface hover:brightness-110${selectedRing}${openingState}`;
	}

	const toggleVolume = (volumeId: string) => {
		if (expandedVolumeId === volumeId) {
			expandedVolumeId = null;
			visiblePages.clear();
		} else {
			expandedVolumeId = volumeId;
		}

		events.onExpand(expandedVolumeId);
	};

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
	 * DOM node recycling — same reasoning as acerola-comic-chapter-list.svelte.
	 * Only one volume is expanded at a time, so a single shared pool serves
	 * whichever volume is currently open.
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

<div class="space-y-4">
	{#if volumes && volumes.length > 0}
		{#each volumes as volume}
			<ComicVolumeButton
				data={{
					title: volume.title,
					totalChapters: volume.totalChapters,
					viewMode,
					coverUri: volume.coverUri,
					bannerUri: volume.bannerUri,
					isExpanded: expandedVolumeId === volume.id
				}}
				events={{
					onClick: () => toggleVolume(volume.id)
				}}
			/>

			{#if expandedVolumeId === volume.id}
				{@const totalPages = Math.ceil(volume.totalChapters / pageSize)}

				<div transition:slide class="ml-6 p-4 pt-0">
					<div class="relative flex w-full flex-col">
						{#if volume.totalChapters > 0}
							{#each Array(totalPages) as _, pageIndex}
								<!-- prettier-ignore -->
								{@const itemsInThisPage = pageIndex === totalPages - 1? volume.totalChapters % pageSize || pageSize: pageSize}

								<!-- Spacer: only reserves the height in the document flow and triggers the
								     IntersectionObserver — the real content comes from the recycled pool below. -->
								<div
									use:trackPage={pageIndex}
									style:height="{itemsInThisPage * ITEM_HEIGHT}px"
								></div>
							{/each}

							{#each slots as slot, slotIndex (slotIndex)}
								{@const blockData =
									slot.pageIndex !== null
										? pagesData.find((page) => page.page === slot.pageIndex)
										: undefined}

								{#each Array(pageSize) as _, itemIndex (itemIndex)}
									{@const chapter = blockData?.items[itemIndex]}

									{#if chapter}
										<div
											class="absolute right-0 left-0 animate-in duration-300 fade-in"
											style:top="{(slot.pageIndex ?? 0) *
												pageSize *
												ITEM_HEIGHT +
												itemIndex * ITEM_HEIGHT}px"
											style:height="{BUTTON_HEIGHT}px"
										>
											<AcerolaHeroButton
												data={{
													title: chapter.title,
													description:
														chapter.fileName !== chapter.title
															? chapter.fileName
															: undefined
												}}
												events={{
													onClick: () => {
														if (data.isSelectionMode) {
															events.onToggleSelect?.(chapter);
														} else {
															openingChapterId = chapter.id;
															events.onOpenChapter?.(chapter);
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
															<RefreshCw
																size={24}
																class="animate-spin"
															/>
														</div>
													{:else if data.isSelectionMode}
														{#if data.isSelected?.(chapter.id)}
															<div
																class="flex h-6 w-6 items-center justify-center rounded-full bg-primary text-primary-foreground"
															>
																<Check size={16} strokeWidth={3} />
															</div>
														{:else}
															<div
																class="flex h-6 w-6 items-center justify-center"
															>
																<div
																	class="size-5 rounded-full border-2 border-border"
																></div>
															</div>
														{/if}
													{:else}
														<div
															class={chapter.isRead
																? ''
																: 'text-primary'}
														>
															{#if chapter.isRead}
																<div
																	class="flex h-6 w-6 items-center justify-center rounded-full bg-primary text-primary-foreground"
																>
																	<Check
																		size={16}
																		strokeWidth={3}
																	/>
																</div>
															{:else}
																<BookOpen size={18} />
															{/if}
														</div>
													{/if}
												{/snippet}

												{#snippet action()}
													<div class="flex items-center gap-2">
														{#if chapter.isRead}
															<span
																class="rounded-full bg-primary-soft px-3 py-1 text-[11px] font-black text-primary"
															>
																{m[
																	'pages.comic.metadata.completed'
																]()}
															</span>
														{/if}
														<div
															role="presentation"
															onclick={(event) =>
																event.stopPropagation()}
														>
															<AcerolaPopover
																state={{
																	open: openMenuId === chapter.id
																}}
																events={{
																	onOpenChange: (open) =>
																		(openMenuId = open
																			? chapter.id
																			: null)
																}}
																ui={{
																	align: 'end',
																	contentClass: 'w-52 p-1.5'
																}}
															>
																{#snippet trigger()}
																	<span
																		class="flex size-10 items-center justify-center rounded-full bg-card text-foreground transition-[filter] hover:brightness-110"
																	>
																		<MoreVertical size={20} />
																	</span>
																{/snippet}

																{#snippet content()}
																	<div
																		class="flex flex-col gap-0.5"
																	>
																		{#if chapter.isRead}
																			<AcerolaButton
																				ui={{
																					variant:
																						'ghost',
																					class: 'h-10 w-full justify-start gap-2.5 px-3 text-sm font-bold text-warning hover:bg-warning-soft hover:text-warning'
																				}}
																				events={{
																					onClick: () => {
																						openMenuId =
																							null;
																						events.onMarkUnread?.(
																							chapter
																						);
																					}
																				}}
																			>
																				<BookOpen
																					size={16}
																					class="shrink-0"
																				/>
																				{m[
																					'pages.comic.selection.markUnread'
																				]()}
																			</AcerolaButton>
																		{:else}
																			<AcerolaButton
																				ui={{
																					variant:
																						'ghost',
																					class: 'h-10 w-full justify-start gap-2.5 px-3 text-sm font-bold text-primary hover:bg-primary-soft'
																				}}
																				events={{
																					onClick: () => {
																						openMenuId =
																							null;
																						events.onMarkRead?.(
																							chapter
																						);
																					}
																				}}
																			>
																				<Check
																					size={16}
																					class="shrink-0"
																				/>
																				{m[
																					'pages.comic.selection.markRead'
																				]()}
																			</AcerolaButton>
																		{/if}

																		<div
																			class="mx-3 my-1 h-px bg-border"
																		></div>

																		<AcerolaButton
																			ui={{
																				variant: 'ghost',
																				class: 'h-10 w-full justify-start gap-2.5 px-3 text-sm font-bold'
																			}}
																			events={{
																				onClick: () => {
																					openMenuId =
																						null;
																					events.onEnterSelection?.(
																						chapter
																					);
																				}
																			}}
																		>
																			<CheckSquare
																				size={16}
																				class="shrink-0 text-muted-foreground"
																			/>
																			{m[
																				'pages.comic.selection.select'
																			]()}
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
						{/if}

						{#if loading}
							<!-- Loading: chapter-row skeleton, never a loose spinner. -->
							<div
								class="space-y-3 py-4"
								role="status"
								aria-label={m['pages.comic.loading']()}
							>
								{#each Array.from({ length: 3 }, (_, i) => i) as i (i)}
									<div
										class="h-16 w-full animate-pulse rounded-tile bg-surface"
									></div>
								{/each}
							</div>
						{/if}

						{#if volume.totalChapters === 0 && !loading}
							<p class="py-4 text-center text-sm font-semibold text-muted-foreground">
								{m['pages.comic.volume.noChapters']()}
							</p>
						{/if}
					</div>
				</div>
			{/if}
		{/each}
	{:else}
		<div
			class="flex flex-col items-center gap-3 rounded-card bg-card py-16 text-center text-muted-foreground shadow-lift"
		>
			<span class="flex size-14 items-center justify-center rounded-full bg-surface">
				<BookOpen size={18} />
			</span>
			<p class="text-base font-black tracking-tight text-foreground">
				{m['pages.comic.volume.noVolumes']()}
			</p>
		</div>
	{/if}
</div>
