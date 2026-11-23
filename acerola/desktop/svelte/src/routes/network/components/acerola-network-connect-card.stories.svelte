<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaNetworkConnectCard from './acerola-network-connect-card.svelte';

	const { Story } = defineMeta({
		component: AcerolaNetworkConnectCard,
		title: 'Pages/Network/AcerolaNetworkConnectCard',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component: 'Card to start a connection with a peer from a pairing code.'
				}
			}
		}
	});
</script>

<Story name="Default" asChild>
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkConnectCard
				data={{ connecting: false }}
				events={{ onConnect: async () => {} }}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Connecting" asChild>
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkConnectCard
				data={{ connecting: true }}
				events={{ onConnect: async () => {} }}
			/>
		</div>
	{/snippet}
</Story>
