<script module lang="ts">
	import { defineMeta } from '@storybook/addon-svelte-csf';
	import AcerolaPeerPicker from './acerola-peer-picker.svelte';
	import type { PairedPeerPayload } from '#lib/contracts/network/network.payloads.ts';

	const peers: PairedPeerPayload[] = [
		{ peerId: 'peer-1', addrs: [1, 2, 3], deviceName: 'Meu Celular' },
		{ peerId: 'peer-2-longid-abcdefghij', addrs: [4, 5, 6], deviceName: null }
	];

	const { Story } = defineMeta({
		component: AcerolaPeerPicker,
		title: 'Composites/AcerolaPeerPicker',
		tags: ['autodocs'],
		parameters: {
			docs: {
				description: {
					component:
						'Dialog to pick a paired network peer, used before actions that depend on a target device.'
				}
			}
		}
	});
</script>

<script lang="ts">
	// AcerolaPeerPicker is fully controlled (state.open + events.onOpenChange) and has no
	// trigger of its own — each story needs its own local state and a real button to open it,
	// otherwise the Docs tab shows the variants with the dialog open at the same time.
	let openWithPeers = $state(false);
	let openEmpty = $state(false);
</script>

<Story name="With Peers" asChild>
	{#snippet children()}
		<div class="min-h-[300px] bg-surface p-8">
			<button
				type="button"
				onclick={() => (openWithPeers = true)}
				class="rounded-full bg-surface px-4 py-2 text-sm font-black text-foreground hover:brightness-110"
			>
				Open Dialog
			</button>
			<AcerolaPeerPicker
				state={{ open: openWithPeers }}
				data={{ peers }}
				events={{ onOpenChange: (isOpen) => (openWithPeers = isOpen), onSelect: () => {} }}
			/>
		</div>
	{/snippet}
</Story>

<Story name="Empty" asChild>
	{#snippet children()}
		<div class="min-h-[300px] bg-surface p-8">
			<button
				type="button"
				onclick={() => (openEmpty = true)}
				class="rounded-full bg-surface px-4 py-2 text-sm font-black text-foreground hover:brightness-110"
			>
				Open Dialog
			</button>
			<AcerolaPeerPicker
				state={{ open: openEmpty }}
				data={{ peers: [] }}
				events={{ onOpenChange: (isOpen) => (openEmpty = isOpen), onSelect: () => {} }}
			/>
		</div>
	{/snippet}
</Story>
