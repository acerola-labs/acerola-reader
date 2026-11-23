<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaThemePicker from './acerola-theme-picker.svelte';

	const { Story } = defineMeta({
		title: 'Primitives/AcerolaThemePicker',
		component: AcerolaThemePicker,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Button that toggles the theme display mode between light, dark and system, cycling in that order on each click. The state comes from the `useTheme` hook, a reactive singleton shared across the whole app.'
				}
			}
		}
	});
</script>

<Story
	name="Default"
	parameters={{
		docs: {
			description: {
				story: 'The icon reflects the current mode (sun, moon or monitor) and toggles when clicked.'
			}
		}
	}}
/>
