<script lang="ts">
	import EyeOff from '@lucide/svelte/icons/eye-off';
	import Trash2 from '@lucide/svelte/icons/trash-2';
	import Eraser from '@lucide/svelte/icons/eraser';
	import Bookmark from '@lucide/svelte/icons/bookmark';
	import CheckSquare from '@lucide/svelte/icons/check-square';
	import Square from '@lucide/svelte/icons/square';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';
	import ChevronUp from '@lucide/svelte/icons/chevron-up';
	import Loader2 from '@lucide/svelte/icons/loader-2';
	import Layers from '@lucide/svelte/icons/layers';
	import Check from '@lucide/svelte/icons/check';
	import Search from '@lucide/svelte/icons/search';
	import { slide } from 'svelte/transition';
	import { error } from '@tauri-apps/plugin-log';
	import { toast } from 'svelte-sonner';
	import AcerolaDialog from '#lib/components/acerola-dialog/acerola-dialog.svelte';
	import AcerolaAlertDialog from '#lib/components/acerola-alert-dialog/acerola-alert-dialog.svelte';
	import AcerolaButton from '#lib/components/acerola-button/acerola-button.svelte';
	import { m } from '#lib/paraglide/messages.js';
	import type { Category } from '#lib/contracts/bookmarks/bookmarks.payloads.ts';
	import { extractErrorMessage } from '#lib/utils/error.util.ts';

	export type ComicActionDialogProps = {
		state?: { open?: boolean };
		data: {
			selectedIds: (string | number)[];
			totalCount?: number;
			bookmarks: Category[];
		};
		events: {
			onHide: (ids: (string | number)[]) => Promise<void>;
			onDelete: (ids: (string | number)[]) => Promise<void>;
			onClearMetadata: (ids: (string | number)[]) => Promise<void>;
			onBookmark: (ids: (string | number)[], categoryId: number) => Promise<void>;
			onSelectAll?: () => void;
			onClose: () => void;
		};
	};

	let { state: controlState, data, events }: ComicActionDialogProps = $props();

	const open = $derived(controlState?.open ?? false);
	const selectedIds = $derived(data.selectedIds);
	const totalCount = $derived(data.totalCount ?? 0);
	const bookmarks = $derived(data.bookmarks);

	let showHideDialog = $state(false);
	let showDeleteDialog = $state(false);
	let showClearMetadataDialog = $state(false);
	let showBookmarkMenu = $state(false);
	let isProcessing = $state(false);
	let activeAction = $state<'hide' | 'delete' | 'clearMetadata' | 'bookmark' | null>(null);
	let activeCategoryId = $state<number | null>(null);
	let searchQuery = $state('');

	const isAllSelected = $derived(totalCount > 0 && selectedIds.length === totalCount);

	const filteredBookmarks = $derived(
		searchQuery.trim() === ''
			? bookmarks
			: bookmarks.filter((b) =>
					b.name.toLowerCase().includes(searchQuery.toLowerCase().trim())
				)
	);

	async function handleHide() {
		if (isProcessing) return;
		isProcessing = true;
		activeAction = 'hide';
		try {
			await events.onHide(selectedIds);
			showHideDialog = false;
			events.onClose();
		} catch (err: unknown) {
			const msg = extractErrorMessage(err);
			error(`Failed to hide comics: ${msg}`);
			toast.error(m['pages.home.toast.error.hide']());
		} finally {
			isProcessing = false;
			activeAction = null;
		}
	}

	async function handleDelete() {
		if (isProcessing) return;
		isProcessing = true;
		activeAction = 'delete';
		try {
			await events.onDelete(selectedIds);
			showDeleteDialog = false;
			events.onClose();
		} catch (err: unknown) {
			const msg = extractErrorMessage(err);
			error(`Failed to delete comics: ${msg}`);
			toast.error(m['pages.home.toast.error.delete']());
		} finally {
			isProcessing = false;
			activeAction = null;
		}
	}

	async function handleClearMetadata() {
		if (isProcessing) return;
		isProcessing = true;
		activeAction = 'clearMetadata';
		try {
			await events.onClearMetadata(selectedIds);
			showClearMetadataDialog = false;
			events.onClose();
		} catch (err: unknown) {
			const msg = extractErrorMessage(err);
			error(`Failed to clear comics metadata: ${msg}`);
			toast.error(m['pages.home.toast.error.clearMetadata']());
		} finally {
			isProcessing = false;
			activeAction = null;
		}
	}

	async function handleBookmark(categoryId: number) {
		if (isProcessing) return;
		isProcessing = true;
		activeAction = 'bookmark';
		activeCategoryId = categoryId;
		try {
			await events.onBookmark(selectedIds, categoryId);
			showBookmarkMenu = false;
			events.onClose();
		} catch (err: unknown) {
			const msg = extractErrorMessage(err);
			error(`Failed to bookmark comics: ${msg}`);
			toast.error(m['pages.home.toast.error.bookmark']());
		} finally {
			isProcessing = false;
			activeAction = null;
			activeCategoryId = null;
		}
	}

	function handleHideClick() {
		showHideDialog = true;
	}

	function handleDeleteClick() {
		showDeleteDialog = true;
	}

	function handleClearMetadataClick() {
		showClearMetadataDialog = true;
	}
</script>

<AcerolaDialog
	state={{ open }}
	data={{
		title: m['pages.home.actions.title'](),
		description: m['pages.home.actions.description']({ count: selectedIds.length })
	}}
	events={{
		onOpenChange: (isOpen) => {
			if (!isOpen) events.onClose();
		}
	}}
	ui={{ contentClass: 'sm:max-w-md' }}
>
	<div class="flex w-full min-w-0 flex-col gap-3">
		<!-- Selection: number tile in the primary tone + select all / deselect. -->
		{#if events.onSelectAll && totalCount > 0}
			<div
				class="flex min-w-0 items-center justify-between gap-3 rounded-tile bg-primary-soft p-4"
			>
				<div class="flex min-w-0 flex-1 items-center gap-3 text-primary">
					<Layers size={20} class="shrink-0" />
					<div class="flex min-w-0 flex-1 flex-col">
						<span class="truncate text-sm font-black tracking-tight tabular-nums">
							{m['pages.home.actions.selection.count']({
								selected: selectedIds.length,
								total: totalCount
							})}
						</span>
						<span class="truncate text-xs font-semibold opacity-80">
							{isAllSelected
								? m['pages.home.actions.selection.all']()
								: m['pages.home.actions.selection.partial']()}
						</span>
					</div>
				</div>

				<AcerolaButton
					ui={{
						variant: isAllSelected ? 'contrast' : 'secondary',
						size: 'sm',
						class: 'h-9 shrink-0 px-3.5 text-xs',
						disabled: isProcessing
					}}
					events={{ onClick: events.onSelectAll }}
				>
					{#if isAllSelected}
						<CheckSquare size={16} class="shrink-0" />
					{:else}
						<Square size={16} class="shrink-0" />
					{/if}
					<span class="truncate">
						{isAllSelected
							? m['pages.home.selection.all.deselect']()
							: m['pages.home.selection.all.selectTotal']({ total: totalCount })}
					</span>
				</AcerolaButton>
			</div>
		{/if}

		<!-- Bookmark: list row that opens the category list. -->
		<div class="min-w-0 overflow-hidden rounded-tile bg-surface">
			<button
				type="button"
				class="flex w-full min-w-0 items-center gap-3 p-3 text-left transition-[filter] duration-200 ease-brand focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none enabled:hover:brightness-110 disabled:cursor-not-allowed disabled:opacity-60"
				disabled={isProcessing || selectedIds.length === 0}
				aria-expanded={showBookmarkMenu}
				onclick={() => (showBookmarkMenu = !showBookmarkMenu)}
			>
				<span
					class="flex size-10 shrink-0 items-center justify-center rounded-full bg-primary-soft text-primary"
				>
					<Bookmark size={18} />
				</span>
				<span class="flex min-w-0 flex-1 flex-col">
					<span class="truncate text-sm font-black tracking-tight text-foreground">
						{m['pages.home.actions.bookmark']()}
					</span>
					<span class="truncate text-xs text-muted-foreground">
						{m['pages.home.actions.collectionsHint']()}
					</span>
				</span>
				{#if bookmarks.length > 0}
					<span
						class="shrink-0 rounded-full bg-card px-2.5 py-0.5 text-[11px] font-black text-muted-foreground tabular-nums"
					>
						{bookmarks.length}
					</span>
				{/if}
				{#if showBookmarkMenu}
					<ChevronUp size={18} class="shrink-0 text-muted-foreground" />
				{:else}
					<ChevronDown size={18} class="shrink-0 text-muted-foreground" />
				{/if}
			</button>

			{#if showBookmarkMenu}
				<div transition:slide={{ duration: 220 }} class="min-w-0 px-2 pb-2">
					{#if bookmarks.length === 0}
						<p class="py-4 text-center text-sm text-muted-foreground">
							{m['pages.home.actions.categories.empty']()}
						</p>
					{:else}
						{#if bookmarks.length > 5}
							<div class="relative mb-2 w-full">
								<Search
									size={16}
									class="absolute top-1/2 left-4 shrink-0 -translate-y-1/2 text-muted-foreground"
								/>
								<input
									type="text"
									placeholder={m[
										'pages.home.actions.filterCategoriesPlaceholder'
									]()}
									aria-label={m[
										'pages.home.actions.filterCategoriesPlaceholder'
									]()}
									bind:value={searchQuery}
									class="h-11 w-full rounded-full bg-input pr-4 pl-11 text-sm text-foreground placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none"
								/>
							</div>
						{/if}

						<div class="max-h-52 w-full min-w-0 space-y-1 overflow-y-auto">
							{#if filteredBookmarks.length === 0}
								<p class="truncate py-3 text-center text-sm text-muted-foreground">
									{m['pages.home.actions.categories.noMatch']({
										query: searchQuery
									})}
								</p>
							{:else}
								{#each filteredBookmarks as category (category.id)}
									{@const hexColor =
										'#' +
										(category.color & 0xffffff).toString(16).padStart(6, '0')}
									<button
										type="button"
										class="group/item flex min-h-11 w-full min-w-0 items-center gap-3 rounded-full px-3 text-left text-sm transition-colors duration-200 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none enabled:hover:bg-card disabled:cursor-not-allowed disabled:opacity-60"
										disabled={isProcessing || selectedIds.length === 0}
										onclick={() => handleBookmark(category.id)}
									>
										<span
											class="size-3 shrink-0 rounded-full"
											style="background-color: {hexColor};"
										></span>
										<span
											class="min-w-0 flex-1 truncate font-semibold text-foreground"
											title={category.name}
										>
											{category.name}
										</span>
										{#if isProcessing && activeAction === 'bookmark' && activeCategoryId === category.id}
											<Loader2
												size={16}
												class="shrink-0 animate-spin text-primary"
											/>
										{:else}
											<Check
												size={16}
												class="shrink-0 text-primary opacity-0 transition-opacity group-hover/item:opacity-100"
											/>
										{/if}
									</button>
								{/each}
							{/if}
						</div>
					{/if}
				</div>
			{/if}
		</div>

		<!-- Actions: list rows with a round icon in the action's tone. -->
		{#snippet actionRow(
			label: string,
			tone: 'warning' | 'destructive',
			Icon: typeof EyeOff,
			busy: boolean,
			onClick: () => void
		)}
			<button
				type="button"
				class="flex w-full min-w-0 items-center gap-3 rounded-tile bg-surface p-3 text-left transition-[filter] duration-200 ease-brand focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none enabled:hover:brightness-110 disabled:cursor-not-allowed disabled:opacity-60"
				disabled={isProcessing || selectedIds.length === 0}
				onclick={onClick}
			>
				<span
					class={[
						'flex size-10 shrink-0 items-center justify-center rounded-full',
						tone === 'warning'
							? 'bg-warning-soft text-warning'
							: 'bg-destructive-soft text-destructive'
					]}
				>
					{#if busy}
						<Loader2 size={18} class="animate-spin" />
					{:else}
						<Icon size={18} />
					{/if}
				</span>
				<span
					class={[
						'min-w-0 flex-1 truncate text-sm font-black tracking-tight',
						tone === 'warning' ? 'text-warning' : 'text-destructive'
					]}
				>
					{label}
				</span>
			</button>
		{/snippet}

		{@render actionRow(
			m['pages.home.actions.hide'](),
			'warning',
			EyeOff,
			isProcessing && activeAction === 'hide',
			handleHideClick
		)}
		{@render actionRow(
			m['pages.home.actions.clearMetadata'](),
			'destructive',
			Eraser,
			isProcessing && activeAction === 'clearMetadata',
			handleClearMetadataClick
		)}
		{@render actionRow(
			m['pages.home.actions.delete'](),
			'destructive',
			Trash2,
			isProcessing && activeAction === 'delete',
			handleDeleteClick
		)}
	</div>
</AcerolaDialog>

<!-- Hide Confirmation Alert Dialog -->
<AcerolaAlertDialog
	state={{ open: showHideDialog }}
	data={{
		title: m['pages.home.actions.hideConfirm.title'](),
		description: m['pages.home.actions.hideConfirm.desc'](),
		cancelText: m['pages.home.actions.cancel'](),
		actionText: m['pages.home.actions.hideConfirm.action']()
	}}
	events={{
		onAction: handleHide,
		onCancel: () => (showHideDialog = false)
	}}
	ui={{ variant: 'destructive' }}
/>

<!-- Delete Confirmation Alert Dialog -->
<AcerolaAlertDialog
	state={{ open: showDeleteDialog }}
	data={{
		title: m['pages.home.actions.deleteConfirm.title'](),
		description: m['pages.home.actions.deleteConfirm.desc'](),
		cancelText: m['pages.home.actions.cancel'](),
		actionText: m['pages.home.actions.deleteConfirm.action']()
	}}
	events={{
		onAction: handleDelete,
		onCancel: () => (showDeleteDialog = false)
	}}
	ui={{ variant: 'destructive' }}
/>

<!-- Clear Metadata Confirmation Alert Dialog -->
<AcerolaAlertDialog
	state={{ open: showClearMetadataDialog }}
	data={{
		title: m['pages.home.actions.clearMetadataConfirm.title'](),
		description: m['pages.home.actions.clearMetadataConfirm.desc']({
			count: selectedIds.length
		}),
		cancelText: m['pages.home.actions.cancel'](),
		actionText: m['pages.home.actions.clearMetadataConfirm.action']()
	}}
	events={{
		onAction: handleClearMetadata,
		onCancel: () => (showClearMetadataDialog = false)
	}}
	ui={{ variant: 'destructive' }}
/>
