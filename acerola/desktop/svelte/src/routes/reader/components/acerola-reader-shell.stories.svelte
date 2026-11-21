<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaReaderShell from './acerola-reader-shell.svelte';

	const { Story } = defineMeta({
		title: 'Pages/Reader/AcerolaReaderShell',
		component: AcerolaReaderShell,
		tags: ['autodocs'],
		parameters: {
			// AcerolaReaderShell uses `position: fixed` anchored to the real viewport, which
			// escapes the docs page's inline canvas. Rendering in an isolated iframe makes the
			// `fixed` fill the preview area correctly.
			docs: { story: { inline: false, iframeHeight: 500 } }
		}
	});
</script>

<Story name="Default" asChild>
	<AcerolaReaderShell>
		{#snippet toolbar()}
			<div class="bg-mantle p-4 font-bold">Toolbar</div>
		{/snippet}

		{#snippet viewport()}
			<div class="flex flex-1 items-center justify-center bg-mantle/30">Viewport</div>
		{/snippet}

		{#snippet footer()}
			<div class="bg-mantle p-4">Footer</div>
		{/snippet}
	</AcerolaReaderShell>
</Story>

<Story name="With Command" asChild>
	<AcerolaReaderShell>
		{#snippet toolbar()}
			<div class="bg-mantle p-4 font-bold">Toolbar</div>
		{/snippet}

		{#snippet viewport()}
			<div class="flex flex-1 items-center justify-center bg-mantle/30">Viewport</div>
		{/snippet}

		{#snippet footer()}
			<div class="bg-mantle p-4">Footer</div>
		{/snippet}

		{#snippet command()}
			<div class="absolute inset-0 grid place-items-center bg-background/30">
				<div class="rounded-card bg-background p-6">Command palette</div>
			</div>
		{/snippet}
	</AcerolaReaderShell>
</Story>
