<script lang="ts" module>
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaMicrosoftStoreButton from './acerola-microsoft-store-button.svelte';

	const { Story } = defineMeta({
		title: 'Primitives/AcerolaMicrosoftStoreButton',
		component: AcerolaMicrosoftStoreButton,
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Microsoft Store download badge — always a fixed dark chip (independent of the site’s light/dark theme), with the store’s official logo and a short caption above the store name, following the usual app store badge conventions.'
				}
			}
		},
		argTypes: {
			label: {
				description: 'Small caption above "Microsoft Store".',
				control: 'text'
			}
		}
	});
</script>

<Story name="Default" args={{}} />

<Story name="CustomLabel" args={{ label: 'Baixe na' }} />
