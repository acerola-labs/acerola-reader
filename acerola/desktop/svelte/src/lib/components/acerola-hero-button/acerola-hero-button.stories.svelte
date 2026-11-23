<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaHeroButton from './acerola-hero-button.svelte';
	import FolderIcon from '@lucide/svelte/icons/folder';
	import PlayIcon from '@lucide/svelte/icons/play';

	const { Story } = defineMeta({
		title: 'Primitives/AcerolaHeroButton',
		component: AcerolaHeroButton,
		tags: ['autodocs'],
		args: {
			data: {
				title: 'Título Padrão',
				description: 'Descrição padrão do componente'
			}
		},
		argTypes: {
			data: { description: 'Main button content', control: 'object' },
			events: { description: 'Button callbacks', control: 'object' },
			ui: { description: 'Button visual configuration', control: 'object' },
			icon: { description: 'Snippet to render an icon on the left', control: 'object' },
			action: {
				description: 'Snippet to render an action component on the right',
				control: 'object'
			}
		}
	});
</script>

{#snippet folderIcon()}
	<FolderIcon size={24} />
{/snippet}

{#snippet actionButton()}
	<div
		class="rounded-full bg-primary/10 p-2 text-primary transition-all group-hover:bg-primary group-hover:text-primary-foreground"
	>
		<PlayIcon size={20} />
	</div>
{/snippet}

<Story
	name="Complete"
	args={{
		data: {
			title: 'Pasta dos Mangás',
			description: 'C:/Users/acerola/comics'
		},
		icon: folderIcon,
		action: actionButton,
		events: { onClick: () => console.log('Clicked!') }
	}}
/>

<Story
	name="Text Only"
	args={{
		data: {
			title: 'Título de Exemplo',
			description: 'Esta descrição deve aparecer agora.'
		}
	}}
/>

<Story name="Manual Usage">
	<AcerolaHeroButton
		data={{
			title: 'Uso Manual',
			description: 'Passando props diretamente no componente'
		}}
	>
		{#snippet icon()}
			{@render folderIcon()}
		{/snippet}

		{#snippet action()}
			{@render actionButton()}
		{/snippet}
	</AcerolaHeroButton>
</Story>
