<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaModePicker from './acerola-mode-picker.svelte';

	const { Story } = defineMeta({
		title: 'Composites/AcerolaModePicker',
		component: AcerolaModePicker,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Button that toggles between light and dark mode. Shows a moon icon in dark mode and a sun in light mode.'
				}
			}
		}
	});
</script>

<Story name="Default" />
