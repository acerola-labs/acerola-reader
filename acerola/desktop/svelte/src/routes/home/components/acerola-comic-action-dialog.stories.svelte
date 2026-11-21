<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaComicActionDialog from './acerola-comic-action-dialog.svelte';
	import type { Category } from '#lib/contracts/bookmarks/bookmarks.payloads.ts';

	const mockBookmarks: Category[] = [
		{ id: 1, name: 'Favoritos', color: 0xff0000 },
		{ id: 2, name: 'Lendo', color: 0x00ff00 },
		{ id: 3, name: 'Completados', color: 0x0000ff }
	];

	const { Story } = defineMeta({
		component: AcerolaComicActionDialog,
		title: 'Pages/Home/AcerolaComicActionDialog',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Batch action dialog for selected comics: hide, delete, clear metadata and assign to a category.'
				}
			}
		}
	});
</script>

<script lang="ts">
	// AcerolaComicActionDialog is fully controlled (state.open + events.onClose) and has no
	// trigger of its own — each story needs its own local state and a real button to open it,
	// otherwise the Docs tab shows all 5 variants with the dialog open at the same time.
	let openDefault = $state(false);
	let openSingleSelection = $state(false);
	let openManySelected = $state(false);
	let openEmpty = $state(false);
	let openNoBookmarks = $state(false);
</script>

<Story name="Default" asChild>
	{#snippet children()}
		<div class="min-h-[400px] bg-surface p-8">
			<button
				type="button"
				onclick={() => (openDefault = true)}
				class="rounded-full bg-surface px-4 py-2 text-sm font-black text-foreground hover:brightness-110"
			>
				Open Dialog
			</button>
			<AcerolaComicActionDialog
				state={{ open: openDefault }}
				data={{ selectedIds: [1, 2, 3], bookmarks: mockBookmarks }}
				events={{
					onHide: async () => {},
					onDelete: async () => {},
					onClearMetadata: async () => {},
					onBookmark: async () => {},
					onClose: () => (openDefault = false)
				}}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Single Selection" asChild>
	{#snippet children()}
		<div class="min-h-[400px] bg-surface p-8">
			<button
				type="button"
				onclick={() => (openSingleSelection = true)}
				class="rounded-full bg-surface px-4 py-2 text-sm font-black text-foreground hover:brightness-110"
			>
				Open Dialog
			</button>
			<AcerolaComicActionDialog
				state={{ open: openSingleSelection }}
				data={{ selectedIds: [1], bookmarks: mockBookmarks }}
				events={{
					onHide: async () => {},
					onDelete: async () => {},
					onClearMetadata: async () => {},
					onBookmark: async () => {},
					onClose: () => (openSingleSelection = false)
				}}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Many Selected" asChild>
	{#snippet children()}
		<div class="min-h-[400px] bg-surface p-8">
			<button
				type="button"
				onclick={() => (openManySelected = true)}
				class="rounded-full bg-surface px-4 py-2 text-sm font-black text-foreground hover:brightness-110"
			>
				Open Dialog
			</button>
			<AcerolaComicActionDialog
				state={{ open: openManySelected }}
				data={{ selectedIds: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10], bookmarks: mockBookmarks }}
				events={{
					onHide: async () => {},
					onDelete: async () => {},
					onClearMetadata: async () => {},
					onBookmark: async () => {},
					onClose: () => (openManySelected = false)
				}}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Empty" asChild>
	{#snippet children()}
		<div class="min-h-[400px] bg-surface p-8">
			<button
				type="button"
				onclick={() => (openEmpty = true)}
				class="rounded-full bg-surface px-4 py-2 text-sm font-black text-foreground hover:brightness-110"
			>
				Open Dialog
			</button>
			<AcerolaComicActionDialog
				state={{ open: openEmpty }}
				data={{ selectedIds: [], bookmarks: mockBookmarks }}
				events={{
					onHide: async () => {},
					onDelete: async () => {},
					onClearMetadata: async () => {},
					onBookmark: async () => {},
					onClose: () => (openEmpty = false)
				}}
			/>
		</div>
	{/snippet}
</Story>

<Story name="No Bookmarks" asChild>
	{#snippet children()}
		<div class="min-h-[400px] bg-surface p-8">
			<button
				type="button"
				onclick={() => (openNoBookmarks = true)}
				class="rounded-full bg-surface px-4 py-2 text-sm font-black text-foreground hover:brightness-110"
			>
				Open Dialog
			</button>
			<AcerolaComicActionDialog
				state={{ open: openNoBookmarks }}
				data={{ selectedIds: [1, 2], bookmarks: [] }}
				events={{
					onHide: async () => {},
					onDelete: async () => {},
					onClearMetadata: async () => {},
					onBookmark: async () => {},
					onClose: () => (openNoBookmarks = false)
				}}
			/>
		</div>
	{/snippet}
</Story>
