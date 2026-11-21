<script lang="ts">
	import { onMount } from 'svelte';
	import AcerolaButton from '#lib/components/acerola-button/acerola-button.svelte';
	import AcerolaButtonIcon from '#lib/components/acerola-button/acerola-button-icon.svelte';
	import AcerolaEmptyState from '#lib/components/acerola-empty-state/acerola-empty-state.svelte';
	import { Input } from '#lib/components/ui/input/index.ts';
	import { useBookmarks } from '#lib/hooks/store/use-bookmarks.svelte.ts';
	import { m } from '#lib/paraglide/messages.js';
	import { autoAnimateList } from '#lib/utils/auto-animate.util.ts';

	import BookmarkIcon from '@lucide/svelte/icons/bookmark';
	import PlusIcon from '@lucide/svelte/icons/plus';
	import Trash2Icon from '@lucide/svelte/icons/trash-2';

	const bookmarkStore = useBookmarks();

	// Bookmark colors are user data (stored as ARGB ints, shared with Android), not theme colors.
	const CATEGORY_COLORS = [
		0xfff44336, 0xffe91e63, 0xff9c27b0, 0xff673ab7, 0xff3f51b5, 0xff2196f3, 0xff03a9f4,
		0xff00bcd4, 0xff009688, 0xff4caf50, 0xff8bc34a, 0xffcddc39, 0xffffeb3b, 0xffffc107,
		0xffff9800, 0xffff5722, 0xff795548, 0xff9e9e9e, 0xff607d8b
	];

	let newBookmarkName = $state('');
	let newBookmarkColor = $state<number>(CATEGORY_COLORS[0]);

	const toHex = (color: number) => '#' + (color & 0xffffff).toString(16).padStart(6, '0');

	async function create() {
		await bookmarkStore.createBookmark(newBookmarkName, newBookmarkColor);
		newBookmarkName = '';
	}

	onMount(() => {
		bookmarkStore.loadBookmarks();
	});
</script>

<!-- New bookmark: name + create on one line, the color swatches below. -->
<form
	class="grid gap-3 rounded-tile bg-surface p-4"
	onsubmit={(event) => {
		event.preventDefault();
		if (newBookmarkName.trim()) create();
	}}
>
	<div class="flex items-center gap-2">
		<Input
			id="bookmarkName"
			aria-label={m['pages.config.bookmarks.name']()}
			placeholder={m['pages.config.bookmarks.name']()}
			bind:value={newBookmarkName}
			class="flex-1 bg-card"
		/>
		<AcerolaButton
			ui={{
				variant: 'contrast',
				type: 'submit',
				disabled: !newBookmarkName.trim() || bookmarkStore.isLoading
			}}
		>
			<PlusIcon size={16} />
			{m['pages.config.bookmarks.create']()}
		</AcerolaButton>
	</div>

	<div class="flex flex-wrap gap-2" role="group" aria-label={m['pages.config.bookmarks.color']()}>
		{#each CATEGORY_COLORS as color (color)}
			<button
				type="button"
				class={[
					'size-7 cursor-pointer rounded-full transition-transform duration-200 ease-brand hover:scale-110 focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none',
					newBookmarkColor === color &&
						'ring-2 ring-foreground ring-offset-2 ring-offset-surface'
				]}
				style="background-color: {toHex(color)}"
				onclick={() => (newBookmarkColor = color)}
				aria-label={m['pages.config.bookmarks.colorOption']({ hex: toHex(color) })}
				aria-pressed={newBookmarkColor === color}
			></button>
		{/each}
	</div>
</form>

<!-- Existing bookmarks: one list row each. -->
<div class="grid gap-2" use:autoAnimateList>
	{#if bookmarkStore.bookmarks.length === 0}
		<AcerolaEmptyState
			data={{ title: m['pages.config.bookmarks.empty'](), icon: BookmarkIcon }}
			ui={{ class: 'py-6' }}
		/>
	{:else}
		{#each bookmarkStore.bookmarks as bookmark (bookmark.id)}
			<div class="flex min-w-0 items-center gap-4 rounded-tile bg-surface p-4">
				<span
					class="flex size-10 shrink-0 items-center justify-center rounded-full bg-card"
				>
					<span
						class="size-4 rounded-full"
						style="background-color: {toHex(bookmark.color)}"
					></span>
				</span>
				<span class="min-w-0 flex-1 truncate text-sm font-black tracking-tight">
					{bookmark.name}
				</span>
				<AcerolaButtonIcon
					events={{ onClick: () => bookmarkStore.deleteBookmark(bookmark.id) }}
					ui={{
						tone: 'destructive',
						'aria-label': m['pages.config.bookmarks.remove']({ name: bookmark.name })
					}}
				>
					<Trash2Icon size={16} />
				</AcerolaButtonIcon>
			</div>
		{/each}
	{/if}
</div>
