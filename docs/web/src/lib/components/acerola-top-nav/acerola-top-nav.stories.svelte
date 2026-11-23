<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaTopNav from './acerola-top-nav.svelte';

	const { Story } = defineMeta({
		title: 'Layout/AcerolaTopNav',
		component: AcerolaTopNav,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Fixed top header with the mobile menu, logo, docs navigation, search trigger and the language/theme controls. `onOpenSearch`/`onOpenMobileNav` are callbacks fired by their respective buttons.'
				}
			}
		},
		argTypes: {
			onOpenSearch: { description: 'Called when any search trigger is clicked.' },
			onOpenMobileNav: { description: 'Called when the (mobile) menu button is clicked.' }
		}
	});
</script>

<Story
	name="Default"
	args={{ onOpenSearch: () => {}, onOpenMobileNav: () => {} }}
	parameters={{
		docs: { description: { story: 'Full header as it appears on any docs page.' } }
	}}
/>
