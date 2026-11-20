<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaCard from './acerola-card.svelte';

	const { Story } = defineMeta({
		title: 'Primitives/AcerolaCard',
		component: AcerolaCard,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component: 'Wrapper of the shadcn/ui Card with a simplified interface via snippets.'
				}
			}
		},
		argTypes: {
			data: { description: 'Card content', control: 'object' },
			ui: { description: 'Card visual configuration', control: 'object' }
		}
	});
</script>

<script>
	import AcerolaButton from '#lib/components/acerola-button/acerola-button.svelte';
	import AcerolaInput from '#lib/components/acerola-input/acerola-input.svelte';
</script>

<Story name="Default" args={{ data: { title: 'Berserk' } }} />

<Story name="With description" args={{ data: { title: 'Berserk', description: 'Kentaro Miura' } }} />

<Story
	name="Size sm"
	args={{ data: { title: 'Vagabond', description: 'Takehiko Inoue' }, ui: { size: 'sm' } }}
/>

<Story name="With content and footer" asChild>
	<div class="w-sm">
		<AcerolaCard
			data={{
				title: 'Configurações da biblioteca',
				description: 'Ajuste como os quadrinhos são organizados e escaneados.'
			}}
		>
			{#snippet children()}
				<div class="flex flex-col gap-4">
					<AcerolaInput />
					<AcerolaInput ui={{ type: 'password', placeholder: 'Senha' }} />
				</div>
			{/snippet}

			{#snippet footer()}
				<div class="flex w-full gap-2">
					<AcerolaButton ui={{ variant: 'outline', class: 'flex-1' }}>Cancelar</AcerolaButton>
					<AcerolaButton ui={{ class: 'flex-1' }}>Salvar</AcerolaButton>
				</div>
			{/snippet}
		</AcerolaCard>
	</div>
</Story>
