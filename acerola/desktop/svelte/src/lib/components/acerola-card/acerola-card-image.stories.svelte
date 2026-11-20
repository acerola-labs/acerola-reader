<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';

	import MoreVerticalIcon from '@lucide/svelte/icons/more-vertical';
	import BookOpenIcon from '@lucide/svelte/icons/book-open';

	import AcerolaCardImage from './acerola-card-image.svelte';

	const { Story } = defineMeta({
		title: 'Primitives/AcerolaCardImage',
		component: AcerolaCardImage,
		tags: ['autodocs'],

		parameters: {
			docs: {
				description: {
					component: 'Comic card with a cover image.'
				}
			}
		}
	});
</script>

<Story
	name="Without cover"
	args={{
		data: {
			title: 'Berserk'
		}
	}}
/>

<Story name="With progress" asChild>
	<AcerolaCardImage
		data={{
			title: 'Berserk',
			description: 'Descrição de teste',
			progress: 50
		}}
	>
		{#snippet floatingBadge()}
			<div
				class="absolute -top-2 left-2 flex h-6 w-6 items-center justify-center rounded-full bg-destructive text-xs font-bold text-destructive-foreground shadow-lift"
			>
				N
			</div>
		{/snippet}
	</AcerolaCardImage>
</Story>

<Story
	name="With description in the overlay"
	args={{
		data: {
			title: 'Berserk',
			description: 'Uma história sombria de espada e feitiçaria.'
		}
	}}
/>

<Story name="With footer and action" asChild>
	<AcerolaCardImage
		data={{
			title: 'Berserk'
		}}
	>
		{#snippet footer()}
			<span
				class="flex items-center gap-1 text-[10px] font-black tracking-wider text-muted-foreground uppercase"
			>
				<BookOpenIcon size={10} />
				374 Caps
			</span>
		{/snippet}

		{#snippet action()}
			<button
				type="button"
				class="p-1 text-muted-foreground transition-colors hover:text-primary"
			>
				<MoreVerticalIcon size={16} />
			</button>
		{/snippet}
	</AcerolaCardImage>
</Story>

<Story name="With custom overlay" asChild>
	<AcerolaCardImage
		data={{
			title: 'Vagabond'
		}}
	>
		{#snippet overlay()}
			<button
				type="button"
				class="w-full rounded-inner bg-primary py-2 text-xs font-black text-primary-foreground"
			>
				LER
			</button>
		{/snippet}
	</AcerolaCardImage>
</Story>
