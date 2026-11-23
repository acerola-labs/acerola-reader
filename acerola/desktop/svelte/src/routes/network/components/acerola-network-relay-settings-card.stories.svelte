<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaNetworkRelaySettingsCard from './acerola-network-relay-settings-card.svelte';

	const { Story } = defineMeta({
		component: AcerolaNetworkRelaySettingsCard,
		title: 'Pages/Network/AcerolaNetworkRelaySettingsCard',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Composable relay configuration: Acerola relay, Iroh Services (own account, authenticated by ticket) and a list of custom relays registered by the user.'
				}
			}
		}
	});

	const noopAsync = async () => {};
	const events = {
		onToggleAcerolaRelay: noopAsync,
		onToggleIrohPublicNetwork: noopAsync,
		onAddCustomRelayUrl: noopAsync,
		onRemoveCustomRelayUrl: noopAsync,
		onSetIrohServicesTicket: noopAsync,
		onClearIrohServicesTicket: noopAsync,
		onRestart: noopAsync
	};
</script>

<Story name="Default (Acerola Relay Only)" asChild>
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkRelaySettingsCard
				data={{
					acerolaRelayUrl: 'https://relay.acerola.app',
					useAcerolaRelay: true,
					useIrohPublicNetwork: false,
					customRelayUrls: [],
					hasIrohServicesTicket: false
				}}
				{events}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Multiple Sources Combined" asChild>
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkRelaySettingsCard
				data={{
					acerolaRelayUrl: 'https://relay.acerola.app',
					useAcerolaRelay: true,
					useIrohPublicNetwork: false,
					customRelayUrls: ['https://relay.example.com'],
					hasIrohServicesTicket: false
				}}
				{events}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Iroh Services (Exclusive, Ticket Configured)" asChild>
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkRelaySettingsCard
				data={{
					acerolaRelayUrl: 'https://relay.acerola.app',
					useAcerolaRelay: true,
					useIrohPublicNetwork: true,
					customRelayUrls: [],
					hasIrohServicesTicket: true
				}}
				{events}
			/>
		</div>
	{/snippet}
</Story>

<Story name="mDNS Only (Nothing Active)" asChild>
	{#snippet children()}
		<div class="max-w-md bg-surface p-8">
			<AcerolaNetworkRelaySettingsCard
				data={{
					acerolaRelayUrl: 'https://relay.acerola.app',
					useAcerolaRelay: false,
					useIrohPublicNetwork: false,
					customRelayUrls: [],
					hasIrohServicesTicket: false
				}}
				{events}
			/>
		</div>
	{/snippet}
</Story>
