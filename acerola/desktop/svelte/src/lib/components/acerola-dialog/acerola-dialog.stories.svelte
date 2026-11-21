<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaDialog from './acerola-dialog.svelte';

	const { Story } = defineMeta({
		component: AcerolaDialog,
		title: 'Primitives/AcerolaDialog',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Generic modal dialog with title and description, the base for more specific compositions.'
				}
			}
		}
	});
</script>

<script lang="ts">
	// AcerolaDialog has no trigger of its own (state.open is fully controlled by the caller) —
	// closed by default here, with a real button in the story to open it, otherwise the Docs
	// tab shows several versions of the dialog open at the same time.
	let open = $state(false);
</script>

<Story
	name="Default"
	args={{
		data: {
			title: 'Dialog Title',
			description: 'Dialog description goes here.'
		}
	}}
	asChild
>
	{#snippet children()}
		<button
			type="button"
			onclick={() => (open = true)}
			class="rounded-full bg-surface px-4 py-2 text-sm font-black text-foreground hover:brightness-110"
		>
			Open Dialog
		</button>
		<AcerolaDialog
			state={{ open }}
			data={{
				title: 'Dialog Title',
				description: 'Dialog description goes here.'
			}}
			events={{ onOpenChange: (isOpen) => (open = isOpen) }}
		/>
	{/snippet}
</Story>
