<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaNavControls from './acerola-nav-controls.svelte';

	const { Story } = defineMeta({
		title: 'Composites/AcerolaNavControls',
		component: AcerolaNavControls,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Header control group: language toggle, theme mode selection, color palette and a link to the project’s GitHub.'
				}
			}
		}
	});
</script>

<Story
	name="Default"
	parameters={{
		docs: { description: { story: 'Default usage inside the header (top-nav) or the mobile menu.' } }
	}}
/>
