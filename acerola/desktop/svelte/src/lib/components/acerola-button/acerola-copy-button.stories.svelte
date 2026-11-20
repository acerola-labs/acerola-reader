<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaCopyButton from './acerola-copy-button.svelte';

	const { Story } = defineMeta({
		component: AcerolaCopyButton,
		title: 'Primitives/AcerolaCopyButton',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Elegant copy button inspired by Cult UI, with animated feedback, check transition and disabled states.'
				}
			}
		}
	});
</script>

{#snippet template()}
	<div class="flex items-center gap-4 p-4">
		<AcerolaCopyButton value="iroh://test-connection-ticket" label="Copiar Código" />
		<AcerolaCopyButton value="https://acerola.app" variant="ghost" size="icon-sm" />
	</div>
{/snippet}

<Story name="Default" {template} />
