<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaScrollProgress from './acerola-scroll-progress.svelte';

	const { Story } = defineMeta({
		title: 'Primitives/AcerolaScrollProgress',
		component: AcerolaScrollProgress,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Scroll / reading progress bar fixed at the top of the screen, with smooth scrub via GSAP ScrollTrigger and a gradient that adapts to the active theme.'
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
				story: 'Reading progress bar fixed at the top.'
			}
		}
	}}
/>
