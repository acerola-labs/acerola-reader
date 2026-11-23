<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaSelect from './acerola-select.svelte';
	import type { AcerolaSelectOption } from './acerola-select.svelte';

	const { Story } = defineMeta({
		title: 'Primitives/AcerolaSelect',
		component: AcerolaSelect,
		tags: ['autodocs'],
		argTypes: {
			data: {
				description: 'Array of options ({ value, label })',
				control: 'object'
			},
			state: { description: 'Currently selected value', control: 'object' },
			ui: { description: 'Visual configuration and placeholder', control: 'object' }
		}
	});

	const defaultOptions: AcerolaSelectOption[] = [
		{ value: 'manga', label: 'Manga' },
		{ value: 'webtoon', label: 'Webtoon' },
		{ value: 'comic', label: 'Comic Book' }
	];
</script>

<Story
	name="Default"
	args={{
		data: { options: defaultOptions },
		ui: { placeholder: 'Selecione o tipo...' }
	}}
/>

<Story
	name="With preset value"
	args={{
		data: { options: defaultOptions },
		state: { value: 'manga' }
	}}
/>

<Story
	name="Custom placeholder"
	args={{
		data: { options: defaultOptions },
		ui: { placeholder: 'Escolha um formato' }
	}}
/>
