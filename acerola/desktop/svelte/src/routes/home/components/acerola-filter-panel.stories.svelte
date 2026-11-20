<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaFilterPanel from './acerola-filter-panel.svelte';
	import type { Category } from '#lib/contracts/bookmarks/bookmarks.payloads.ts';

	const mockBookmarks: Category[] = [
		{ id: 1, name: 'Favoritos', color: 0xff0000 },
		{ id: 2, name: 'Lendo', color: 0x00ff00 },
		{ id: 3, name: 'Completados', color: 0x0000ff }
	];

	const { Story } = defineMeta({
		component: AcerolaFilterPanel,
		title: 'Pages/Home/AcerolaFilterPanel',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Library filter and sort panel: order, hidden items display, metadata source and bookmark category.'
				}
			}
		}
	});
</script>

<script lang="ts">
	// AcerolaFilterPanel is fully controlled (state.open + events.onClose) and has no trigger
	// of its own — each story needs its own local state and a real button to open it,
	// otherwise the Docs tab shows the variants with the panel open at the same time.
	let openDefault = $state(false);
	let openActiveFilters = $state(false);
	let openNoBookmarks = $state(false);
</script>

<Story name="Default" asChild>
	{#snippet children()}
		<div class="min-h-[600px] bg-surface p-8">
			<button
				type="button"
				onclick={() => (openDefault = true)}
				class="bg-surface-elevated rounded-inner border border-border px-4 py-2 text-sm font-medium text-foreground hover:opacity-80"
			>
				Open Panel
			</button>
			<AcerolaFilterPanel
				state={{ open: openDefault }}
				data={{
					sortBy: 'title',
					sortOrder: 'asc',
					showHidden: false,
					metadataSource: 'all',
					bookmarkFilter: 'all',
					bookmarks: mockBookmarks
				}}
				events={{ onApply: () => {}, onClose: () => (openDefault = false) }}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Active Filters" asChild>
	{#snippet children()}
		<div class="min-h-[600px] bg-surface p-8">
			<button
				type="button"
				onclick={() => (openActiveFilters = true)}
				class="bg-surface-elevated rounded-inner border border-border px-4 py-2 text-sm font-medium text-foreground hover:opacity-80"
			>
				Open Panel
			</button>
			<AcerolaFilterPanel
				state={{ open: openActiveFilters }}
				data={{
					sortBy: 'lastUpdated',
					sortOrder: 'desc',
					showHidden: true,
					metadataSource: 'mangadex',
					bookmarkFilter: 1,
					bookmarks: mockBookmarks
				}}
				events={{ onApply: () => {}, onClose: () => (openActiveFilters = false) }}
			/>
		</div>
	{/snippet}
</Story>

<Story name="No Bookmarks" asChild>
	{#snippet children()}
		<div class="min-h-[600px] bg-surface p-8">
			<button
				type="button"
				onclick={() => (openNoBookmarks = true)}
				class="bg-surface-elevated rounded-inner border border-border px-4 py-2 text-sm font-medium text-foreground hover:opacity-80"
			>
				Open Panel
			</button>
			<AcerolaFilterPanel
				state={{ open: openNoBookmarks }}
				data={{
					sortBy: 'title',
					sortOrder: 'asc',
					showHidden: false,
					metadataSource: 'all',
					bookmarkFilter: 'all',
					bookmarks: []
				}}
				events={{ onApply: () => {}, onClose: () => (openNoBookmarks = false) }}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Closed" asChild>
	{#snippet children()}
		<div class="min-h-[200px] bg-surface p-8 text-sm text-muted-foreground">
			Panel is closed - nothing renders when state.open is false.
			<AcerolaFilterPanel
				state={{ open: false }}
				data={{
					sortBy: 'title',
					sortOrder: 'asc',
					showHidden: false,
					metadataSource: 'all',
					bookmarkFilter: 'all',
					bookmarks: mockBookmarks
				}}
				events={{ onApply: () => {}, onClose: () => {} }}
			/>
		</div>
	{/snippet}
</Story>
