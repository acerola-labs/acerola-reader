<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaShinyText from './acerola-shiny-text.svelte';

	const { Story } = defineMeta({
		title: 'Primitives/AcerolaShinyText',
		component: AcerolaShinyText,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Text with a shine that sweeps continuously from left to right, as a subtle highlight effect. Respects `prefers-reduced-motion`.'
				}
			}
		},
		argTypes: {
			text: { description: 'Displayed text.', control: 'text' },
			speed: { description: 'Duration of one animation loop, in seconds.', control: 'number' },
			class: { description: 'Extra classes applied to the element.', control: 'text' }
		}
	});
</script>

<Story
	name="Default"
	args={{ text: 'Acerola Reader' }}
	parameters={{
		docs: {
			description: {
				story: 'Default speed of 6 seconds per cycle.'
			}
		}
	}}
/>

<Story
	name="Fast"
	args={{ text: 'Acerola Reader', speed: 2 }}
	parameters={{
		docs: {
			description: {
				story: 'Custom speed, useful to draw more attention to short highlights.'
			}
		}
	}}
/>
