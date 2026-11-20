<script module lang="ts">
	import type { PairedPeerPayload } from '#lib/contracts/network/network.payloads.ts';

	export type AcerolaPeerPickerProps = {
		state: {
			open: boolean;
		};
		data: {
			peers: PairedPeerPayload[];
			/** Local nickname (set by the user on this device) to override the default
			 *  `deviceName` — same priority as `usePeerConnection().peerLabel`. */
			nicknameFor?: (peerId: string) => string | null | undefined;
		};
		events: {
			onOpenChange: (open: boolean) => void;
			onSelect: (peer: PairedPeerPayload) => void;
		};
	};
</script>

<script lang="ts">
	import { m } from '#lib/paraglide/messages.js';
	import AcerolaDialog from '#lib/components/acerola-dialog/acerola-dialog.svelte';
	import { shortId } from '#lib/utils/connection-code.util.ts';
	import MonitorIcon from '@lucide/svelte/icons/monitor';

	let { state, data, events }: AcerolaPeerPickerProps = $props();

	// On story teardown (Storybook + vitest browser mode), this template's reactive effect
	// runs once more with `events` already undefined before the component is actually
	// destroyed — without the fallback here this leaks as an unhandled error and brings down
	// the suite even with every assert passing (same pattern as
	// acerola-network-my-device-card.svelte).
	let safeEvents = $derived(events ?? { onOpenChange: () => {}, onSelect: () => {} });
	let safeData = $derived(data ?? { peers: [] });
</script>

<AcerolaDialog
	state={{ open: state?.open ?? false }}
	data={{ title: m['pages.network.peerPicker.title']() }}
	events={{ onOpenChange: safeEvents.onOpenChange }}
	ui={{
		contentClass:
			'w-full max-w-[calc(100vw-2rem)] sm:max-w-sm border-border/60 bg-background/95 backdrop-blur-xl shadow-float p-6 rounded-tile overflow-hidden'
	}}
>
	{#if safeData.peers.length === 0}
		<p class="p-4 text-center text-sm text-muted-foreground">
			{m['pages.network.peers.empty']()}
		</p>
	{:else}
		<div class="flex flex-col gap-1 py-1">
			{#each safeData.peers as peer (peer.peerId)}
				<button
					type="button"
					class="flex items-center gap-3 rounded-inner px-3 py-2.5 text-left transition-colors hover:bg-muted"
					onclick={() => safeEvents.onSelect(peer)}
				>
					<MonitorIcon size={18} class="shrink-0 text-muted-foreground" />
					<span class="truncate text-sm font-medium text-foreground">
						{safeData.nicknameFor?.(peer.peerId) ?? peer.deviceName ?? shortId(peer.peerId)}
					</span>
				</button>
			{/each}
		</div>
	{/if}
</AcerolaDialog>
