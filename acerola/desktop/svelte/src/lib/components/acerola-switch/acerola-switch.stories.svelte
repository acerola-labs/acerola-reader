<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaSwitch from './acerola-switch.svelte';

	const { Story } = defineMeta({
		title: 'Primitives/AcerolaSwitch',
		component: AcerolaSwitch,
		tags: ['autodocs'],
		parameters: {
			controls: {
				include: ['state', 'ui']
			}
		},
		argTypes: {
			state: { description: 'Switch state', control: 'object' },
			ui: { description: 'Switch visual configuration', control: 'object' }
		}
	});
</script>

<Story name="Default" asChild>
	<AcerolaSwitch />
</Story>

<Story name="Checked" asChild>
	<AcerolaSwitch state={{ checked: true }} />
</Story>

<Story name="Small" asChild>
	<AcerolaSwitch ui={{ size: 'sm' }} />
</Story>

<Story name="Disabled" asChild>
	<AcerolaSwitch ui={{ disabled: true }} />
</Story>
