<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaCopyCard from './acerola-copy-card.svelte';

	const { Story } = defineMeta({
		title: 'Primitives/AcerolaCopyCard',
		component: AcerolaCopyCard,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Compact card for a short value the user will likely want to copy (email, ID, endpoint), with a dedicated copy button instead of relying on selecting the text manually.'
				}
			}
		},
		argTypes: {
			label: { description: 'Label above the value.', control: 'text' },
			value: {
				description: 'Value displayed and copied to the clipboard.',
				control: 'text'
			},
			href: {
				description: 'If set, the value becomes a link (e.g. mailto:).',
				control: 'text'
			},
			copyLabel: { description: 'Accessible button text before copying.', control: 'text' },
			copiedLabel: {
				description: 'Accessible button text right after copying.',
				control: 'text'
			}
		}
	});
</script>

<Story
	name="Default"
	args={{ label: 'Contato', value: 'contact@acerola.app' }}
	parameters={{
		docs: { description: { story: 'Value displayed as plain text, with a copy button.' } }
	}}
/>

<Story
	name="WithLink"
	args={{
		label: 'Contato',
		value: 'contact@acerola.app',
		href: 'mailto:contact@acerola.app'
	}}
	parameters={{
		docs: {
			description: {
				story: 'The value is also a clickable link (e.g. `mailto:`) besides being copyable.'
			}
		}
	}}
/>
