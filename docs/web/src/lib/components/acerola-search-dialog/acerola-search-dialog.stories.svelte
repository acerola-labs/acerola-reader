<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaSearchDialog from './acerola-search-dialog.svelte';

	// `open: true` as the initial arg (instead of opening via real interaction) leaves bits-ui's
	// Dialog.Content stuck — the "presence" logic that reveals the content only runs on a real
	// closed→open transition, so starting already open breaks the preview (and, on the Docs
	// tab, the full-screen overlay blurs the whole documentation page behind it). False by
	// default; use the story's button (or Ctrl/Cmd+K) to open it.
	const { Story } = defineMeta({
		title: 'Composites/AcerolaSearchDialog',
		component: AcerolaSearchDialog,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Search command palette, opened via `bind:open` or the Ctrl/Cmd+K shortcut. The Pagefind index only exists in the production build, so in development (and in this story) the state shown is always "unavailable".'
				}
			}
		}
	});
</script>

<script lang="ts">
	// Command.Dialog renders nothing visible while `open` is false (it's just a portal) —
	// without a real trigger in the story, the Docs tab shows an empty canvas and the
	// component can't be inspected without already knowing the keyboard shortcut.
	let open = $state(false);
</script>

{#snippet template()}
	<div class="flex min-h-[240px] w-full items-center justify-center">
		<button
			type="button"
			onclick={() => (open = true)}
			class="hover:bg-surface-elevated rounded-inner border border-border bg-surface px-4 py-2 text-sm font-medium text-foreground"
		>
			Abrir busca (Ctrl/Cmd+K)
		</button>
	</div>
	<AcerolaSearchDialog bind:open />
{/snippet}

<Story
	name="Default"
	{template}
	parameters={{
		docs: {
			description: {
				story:
					'Closed by default — click the button (or Ctrl/Cmd+K) to open it and see the search unavailable state outside the production build.'
			}
		}
	}}
/>
